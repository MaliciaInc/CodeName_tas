use iced::{
    widget::{button, container, mouse_area, scrollable, text, text_input, text_editor, Column, Row, Space},
    Alignment, Color, Element, Length, Vector,
    Padding, Theme,
};
use crate::app::{Message, PmMessage, PmState};
use crate::model::{BoardColumn, Card, KanbanBoardData};
use crate::ui;

pub fn pm_board<'a>(
    state: &'a crate::app::AppState,
    t: ui::Tokens,
    data: &'a Option<KanbanBoardData>,
) -> Element<'a, Message> {

    // Header
    let header = Row::new()
        .align_y(Alignment::Center)
        .push(text("Project Board").size(24).color(t.foreground))
        .push(Space::new().width(Length::Fill))
        .push(ui::primary_button(t, "Create Task".to_string(), Message::Pm(PmMessage::OpenGlobalCreate)))
        .padding(Padding { top: 0.0, right: 0.0, bottom: 20.0, left: 0.0 });

    let Some(board_data) = data else {
        return container(text("Loading Board...").color(t.muted_fg))
            .width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill).into();
    };

    let mut columns_row = Row::new().spacing(16);

    for (col, cards) in &board_data.columns {
        columns_row = columns_row.push(
            container(render_column(t, col, cards, state))
                .width(Length::Fixed(320.0)) // Un poco más ancho para mejor lectura
                .height(Length::Fill)
        );
    }

    // FIX VISUAL: Mouse Area debe envolver el contenido, no ser el contenedor principal del scroll
    // y el padding debe estar dentro del scrollable para evitar cortes visuales.
    let board_bg = mouse_area(
        container(columns_row)
            .width(Length::Shrink) // Shrink para que el scroll horizontal funcione
            .height(Length::Fill)
    );

    let content = scrollable(board_bg)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new().width(8.0).scroller_width(8.0) // Scrollbar visible pero sutil
        ))
        .width(Length::Fill)
        .height(Length::Fill);

    // Layout principal con Padding general de página
    ui::page_padding(
        Column::new()
            .push(header)
            .push(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    )
}

fn render_column<'a>(t: ui::Tokens, col: &'a BoardColumn, cards: &'a Vec<Card>, state: &'a crate::app::AppState) -> Element<'a, Message> {
    let mut cards_col = Column::new().spacing(10);

    for card in cards {
        let is_dragging_active = match &state.pm_state {
            PmState::Dragging { card_id, active, .. } => card_id == &card.id && *active,
            _ => false,
        };


        if !is_dragging_active {
            cards_col = cards_col.push(render_card(t, card, 1.0));
        } else {
            // Placeholder visual cuando se arrastra
            cards_col = cards_col.push(
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fixed(80.0))
                    .style(move |_: &Theme| {
                        let mut s = ui::container_style(ui::alpha(t.accent, 0.1), Color::TRANSPARENT);
                        s.border.radius = 6.0.into();
                        s.border.width = 1.0;
                        s.border.color = ui::alpha(t.accent, 0.3);
                        s
                    })
            );
        }
    }

    let header = Row::new().align_y(Alignment::Center)
        .push(text(&col.name).size(14).color(t.foreground).width(Length::Fill))
        .push(
            container(text(format!("{}", cards.len())).size(10).color(t.muted_fg))
                .padding([2, 6])
                .style(move |_: &Theme| {
                    let mut s = ui::container_style(t.input_border, t.foreground);
                    s.border.radius = 99.0.into(); s
                })
        );

    // FIX: Scrollable vertical dentro de la columna
    let column_content = scrollable(cards_col)
        .direction(scrollable::Direction::Vertical(scrollable::Scrollbar::new().width(4.0)))
        .height(Length::Fill);

    let body = Column::new().spacing(12)
        .push(header)
        .push(column_content);

    let content = container(body)
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &Theme| {
            let mut s = ui::container_style(ui::alpha(t.shell_a, 0.5), t.foreground);
            s.border.radius = 12.0.into(); // Más redondeado
            s.border.width = 1.0;
            s.border.color = t.border;

            if let Some(hovered) = &state.hovered_column {
                if hovered == &col.id {
                    s.background = Some(ui::alpha(t.accent, 0.03).into());
                    s.border.color = ui::alpha(t.accent, 0.4);
                }
            }
            s
        });

    mouse_area(content)
        .on_enter(Message::Pm(PmMessage::ColumnHovered(col.id.clone())))
        .into()
}

fn render_card<'a>(t: ui::Tokens, card: &'a Card, alpha_mul: f32) -> Element<'a, Message> {
    let title_color = ui::alpha(t.foreground, alpha_mul);
    let bg_color = if alpha_mul < 1.0 { ui::alpha(t.card, 0.2) } else { t.card };

    let priority_color = match card.priority.as_str() {
        "High" => Color::from_rgb8(239, 68, 68), // Red 500
        "Low" => Color::from_rgb8(34, 197, 94),  // Green 500
        _ => Color::from_rgb8(234, 179, 8),      // Yellow 500
    };

    let content = Column::new().spacing(8)
        .push(
            Row::new().align_y(Alignment::Start)
                .push(text(&card.title).size(14).color(title_color).width(Length::Fill))
                .push(
                    container(Space::new())
                        .width(Length::Fixed(8.0))
                        .height(Length::Fixed(8.0))
                        .style(move |_: &Theme| {
                            let mut s = ui::container_style(priority_color, Color::TRANSPARENT);
                            s.border.radius = 99.0.into(); s
                        })
                )
        );

    // Opcional: Mostrar preview de descripción si no está vacía
    let has_desc = !card.description.trim().is_empty();
    let final_content = if has_desc {
        content.push(text(&card.description).size(12).color(ui::alpha(t.muted_fg, alpha_mul * 0.8)).shaping(iced::widget::text::Shaping::Advanced))
    } else {
        content
    };

    let card_box = container(final_content).padding(12).width(Length::Fill).style(move |_: &Theme| {
        let mut s = ui::container_style(bg_color, title_color);
        s.border.width = 1.0;
        s.border.color = ui::alpha(t.border, alpha_mul);
        s.border.radius = 8.0.into();
        if alpha_mul >= 1.0 {
            s.shadow = iced::Shadow { color: Color::from_rgba8(0,0,0,0.1), offset: Vector::new(0.0, 2.0), blur_radius: 4.0 };
        }
        s
    });

    mouse_area(card_box)
        .on_press(Message::Pm(PmMessage::DragStart(card.id.clone())))
        .on_enter(Message::Pm(PmMessage::CardHovered(card.id.clone())))
        .into()
}

// render_modal se mantiene igual, ya está en app.rs en tu estructura, pero asegúrate de que use PmMessage::Save
pub fn render_modal<'a>(t: ui::Tokens, title: &'a str, desc: &'a text_editor::Content, priority: &'a str, is_new: bool) -> Element<'a, Message> {
    // ... (Tu código existente para render_modal en ui.rs o donde lo tengas es correcto, solo asegúrate que emite PmMessage::Save)
    // REPLICA EXACTA DE TU pm_board.rs ORIGINAL PERO LIMPIA
    let header_text = if is_new { "Create Task" } else { "Edit Task" };

    let p_btn = |label: &str, val: &str| {
        let is_selected = priority == val;
        let mut btn = button(
            text(label.to_string())
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(if is_selected { t.background } else { t.muted_fg }) })
        )
            .padding([6, 12])
            .on_press(Message::Pm(PmMessage::PriorityChanged(val.to_string())));

        if is_selected {
            btn = btn.style(ui::primary_button_style(t));
        } else {
            btn = btn.style(ui::ghost_button_style(t));
        }
        btn
    };

    let priority_row = Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(text("Priority:").size(12).style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }))
        .push(p_btn("Low", "Low"))
        .push(p_btn("Medium", "Medium"))
        .push(p_btn("High", "High"));

    let form = Column::new()
        .spacing(16)
        .push(text(header_text).size(20).style(move |_| iced::widget::text::Style { color: Some(t.foreground) }))
        .push(
            Column::new()
                .spacing(6)
                .push(text("Title").size(12).style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }))
                .push(
                    text_input("", title)
                        .on_input(|v| Message::Pm(PmMessage::TitleChanged(v)))
                        .padding(10)
                        .style(ui::input_style(t))
                )
        )
        .push(
            Column::new()
                .spacing(6)
                .push(text("Description").size(12).style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }))
                .push(
                    text_editor(desc)
                        .on_action(|v| Message::Pm(PmMessage::DescChanged(v)))
                        .padding(10)
                        .height(Length::Fixed(150.0))
                        .style(ui::text_editor_style(t))
                )
        )
        .push(priority_row)
        .push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(ui::primary_button(t, "Save Task".to_string(), Message::Pm(PmMessage::Save)))
                .push(ui::ghost_button(t, "Cancel".to_string(), Message::Pm(PmMessage::Cancel)))
                .push(Space::new().width(Length::Fill))
                .push(if !is_new { ui::danger_button(t, "Delete".to_string(), Message::Pm(PmMessage::Delete)) } else { Space::new().into() })
        );

    container(
        container(form)
            .width(Length::Fixed(500.0))
            .padding(24)
            .style(move |_: &Theme| {
                let mut s = ui::container_style(t.popover, t.foreground);
                s.border.color = t.border;
                s.border.width = 1.0;
                s.border.radius = 12.0.into();
                s.shadow = iced::Shadow {
                    color: Color::BLACK,
                    offset: Vector::new(0.0, 10.0),
                    blur_radius: 40.0,
                };
                s
            })
    )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_: &Theme| ui::container_style(Color::from_rgba8(0, 0, 0, 0.70), t.foreground))
        .into()
}