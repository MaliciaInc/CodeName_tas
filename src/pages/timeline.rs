use iced::{Alignment, Color, Length, Vector, Padding};
use iced::widget::{button, container, mouse_area, text, text_input, text_editor, pick_list, Column, Row, Space};
use iced::Theme;
use iced::border;

use crate::app::{AppState, Message, TimelineMessage};
use crate::model::{TimelineEvent, Location, TimelineEra};
use crate::{pages::E, ui};

pub fn timeline<'a>(state: &'a AppState, t: ui::Tokens, universe_id: &'a str) -> E<'a> {
    let universe_name = state.universes.iter().find(|u| u.id == universe_id)
        .map(|u| u.name.as_str()).unwrap_or("Unknown");

    let header = Row::new().align_y(Alignment::Center)
        .push(
            Column::new()
                .push(text(format!("Timeline — {}", universe_name)).size(26).color(t.foreground))
                .push(text("Chronicle of events and history.").size(12).color(t.muted_fg))
        )
        .push(Space::new().width(Length::Fill))
        .push(ui::outline_button(t, "Back".to_string(), Message::BackToUniverse(universe_id.to_string())))
        .push(ui::primary_button(t, "Add Era".to_string(), Message::Timeline(TimelineMessage::EditorOpenCreateEra)));

    let mut list = Column::new().spacing(0);

    // Sort data locally for deterministic render
    // C12: ya vienen ordenados desde MessagesController
    let eras = &state.timeline_eras;
    let events = &state.timeline_events;

    if eras.is_empty() && events.is_empty() {
        list = list.push(ui::card(t, text("No history recorded yet.").size(14).color(t.muted_fg).into()));
        let content = Column::new().spacing(20).push(header).push(list);
        return ui::page_padding(content.into());
    }

    // Helper: does an event year fall inside an era?
    let in_era = |evt_year: i64, era: &TimelineEra| -> bool {
        if evt_year < era.start_year { return false; }
        match era.end_year {
            Some(end) => evt_year <= end,
            None => true,
        }
    };

    // Track which events got assigned to any era
    let mut assigned = vec![false; events.len()];

    // Render eras FIRST (so empty eras show up)
    for era in eras.iter() {
        list = list.push(era_banner_interactive(t, era));
        list = list.push(Space::new().height(Length::Fixed(12.0)));

        let mut any = false;
        for (idx, evt) in events.iter().enumerate() {
            if in_era(evt.year, era) {
                assigned[idx] = true;
                any = true;
                list = list.push(timeline_row(t, evt, false, false, &state.locations, universe_id));
            }
        }

        if !any {
            // This was the line that previously failed due to Padding conversion in your build.
            // Using explicit Padding struct avoids E0277.
            list = list.push(
                container(text("No events in this era yet.").size(12).color(t.muted_fg))
                    .padding(Padding { top: 0.0, right: 0.0, bottom: 18.0, left: 164.0 })
                    .width(Length::Fill)
            );
        }

        list = list.push(Space::new().height(Length::Fixed(26.0)));
    }

    // Render unassigned events (events that do not fall into any era)
    let any_unassigned = assigned.iter().any(|v| !*v);

    if any_unassigned {
        let phantom = TimelineEra {
            id: String::new(),
            universe_id: String::new(),
            name: String::new(),
            start_year: 0,
            end_year: None,
            description: String::new(),
            color: String::new(),
        };

        list = list.push(era_banner_interactive(t, &phantom));
        list = list.push(Space::new().height(Length::Fixed(12.0)));

        for (idx, evt) in events.iter().enumerate() {
            if !assigned[idx] {
                list = list.push(timeline_row(t, evt, false, false, &state.locations, universe_id));
            }
        }
    }

    let content = Column::new().spacing(20).push(header).push(list);
    ui::page_padding(content.into())
}

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Color::from_rgb8(r, g, b)
    } else {
        Color::from_rgb8(120, 120, 120)
    }
}

// Banner de Era Interactivo y Redondeado
fn era_banner_interactive<'a>(t: ui::Tokens, era: &'a TimelineEra) -> iced::Element<'static, Message> {
    let color_str = era.color.clone();
    let name_str = era.name.clone();
    let era_id = era.id.clone();
    let start_year = era.start_year;

    let color = hex_to_color(&color_str);
    let bg = ui::alpha(color, 0.15);
    let border_col = ui::alpha(color, 0.5);

    let end_text = if let Some(end) = era.end_year { format!("{}", end) } else { "Present".to_string() };
    let range = format!("{}  —  {}", era.start_year, end_text);

    let range_pill = container(text(range).size(11).color(color))
        .padding([2, 8])
        .style(move |_: &Theme| {
            let mut s = ui::container_style(ui::alpha(color, 0.1), Color::TRANSPARENT);
            s.border.radius = 12.0.into();
            s
        });

    let header_content = Row::new().align_y(Alignment::Center)
        .push(text(name_str).size(18).color(t.foreground))
        .push(Space::new().width(Length::Fixed(10.0)))
        .push(range_pill)
        .push(Space::new().width(Length::Fill))
        .push(ui::ghost_button(t, "+ Event".to_string(), Message::Timeline(TimelineMessage::EditorOpenCreateEvent(Some(start_year)))))
        .push(ui::ghost_button(t, "Edit".to_string(), Message::Timeline(TimelineMessage::EditEra(era_id.clone()))))
        .push(ui::danger_button(t, "×".to_string(), Message::Timeline(TimelineMessage::DeleteEra(era_id.clone()))));

    let card = container(header_content)
        .width(Length::Fill)
        .padding([12, 16])
        .style(move |_: &Theme| {
            let mut s = ui::container_style(bg, t.foreground);
            s.border.width = 1.0;
            s.border.color = border_col;
            s.border.radius = 12.0.into();
            s
        });

    mouse_area(card)
        .on_press(Message::Timeline(TimelineMessage::EraBannerClicked(era_id)))
        .into()
}

fn timeline_row<'a>(
    t: ui::Tokens,
    evt: &'a TimelineEvent,
    _is_first: bool,
    _is_last: bool,
    locations: &'a [Location],
    universe_id: &'a str
) -> iced::Element<'static, Message> {

    // Fallback: if display_date is empty, show year
    let display_date = if evt.display_date.trim().is_empty() {
        evt.year.to_string()
    } else {
        evt.display_date.clone()
    };

    let title = evt.title.clone();
    let description = evt.description.clone();
    let importance = evt.importance.clone();
    let event_id = evt.id.clone();
    let uid = universe_id.to_string();
    let kind = evt.kind.clone();
    let accent_color = hex_to_color(&evt.color);

    let date_col = container(
        text(display_date)
            .size(13)
            .color(accent_color)
            .align_x(iced::alignment::Horizontal::Right)
            .width(Length::Fill)
    )
        .width(Length::Fixed(140.0))
        .padding(Padding { top: 18.0, right: 16.0, bottom: 0.0, left: 0.0 });

    let is_major = importance == "Major";
    let dot_size = if is_major { 14.0 } else { 10.0 };
    let dot_top_margin = if is_major { 18.0 } else { 20.0 };

    let dot = container(Space::new())
        .width(Length::Fixed(dot_size))
        .height(Length::Fixed(dot_size))
        .style(move |_: &Theme| {
            let mut s = ui::container_style(accent_color, Color::TRANSPARENT);
            s.border.radius = 999.0.into();
            if is_major {
                s.shadow = iced::Shadow { color: accent_color, offset: Vector::new(0.0, 0.0), blur_radius: 6.0 };
            }
            s
        });

    let safe_line_col = Column::new()
        .align_x(Alignment::Center)
        .width(Length::Fixed(24.0))
        .push(Space::new().height(Length::Fixed(dot_top_margin)))
        .push(dot)
        .push(container(Space::new())
            .width(Length::Fixed(2.0))
            .height(Length::Fill)
            .style(move |_: &Theme| ui::container_style(ui::alpha(t.muted_fg, 0.2), Color::TRANSPARENT))
        );

    let loc_info: iced::Element<'static, Message> = if let Some(lid) = &evt.location_id {
        let lid_clone = lid.clone();
        let name = locations.iter().find(|l| l.id == *lid).map(|l| l.name.clone()).unwrap_or("Unknown".to_string());
        button(text(format!("📍 {}", name)).size(12).color(t.muted_fg))
            .padding(0).style(crate::ui::ghost_button_style(t))
            .on_press(Message::GoToLocation(uid, lid_clone))
            .into()
    } else {
        Space::new().width(Length::Shrink).height(Length::Shrink).into()
    };

    let kind_badge = container(text(kind).size(10).color(accent_color))
        .padding([2, 6])
        .style(move |_: &Theme| {
            let mut s = ui::container_style(ui::alpha(accent_color, 0.1), accent_color);
            s.border.radius = 4.0.into();
            s
        });

    let card_content = Column::new().spacing(6)
        .push(Row::new().align_y(Alignment::Center).spacing(8)
            .push(text(title).size(16).color(t.foreground))
            .push(kind_badge)
            .push(Space::new().width(Length::Fill))
            .push(loc_info)
        )
        .push(text(description).size(13).color(ui::alpha(t.muted_fg, 0.8)));

    let card_inner = container(card_content)
        .padding(16)
        .width(Length::Fill)
        .style(move |_: &Theme| {
            let mut s = ui::container_style(t.card, t.foreground);
            s.border.color = ui::alpha(accent_color, 0.5);
            s.border.width = 1.0;
            s.border.radius = 8.0.into();
            s
        });

    let card_interactive = mouse_area(card_inner)
        .on_press(Message::Timeline(TimelineMessage::CardClicked(event_id.clone())));

    let actions_row = Row::new()
        .spacing(8)
        .push(Space::new().width(Length::Fill))
        .push(ui::ghost_button(t, "Edit".to_string(), Message::Timeline(TimelineMessage::EditEvent(event_id.clone()))))
        .push(ui::danger_button(t, "Delete".to_string(), Message::Timeline(TimelineMessage::DeleteEvent(event_id))));

    let content_col = Column::new()
        .spacing(8)
        .push(card_interactive)
        .push(actions_row)
        .padding(Padding { top: 0.0, right: 0.0, bottom: 24.0, left: 0.0 });

    Row::new()
        .align_y(Alignment::Start)
        .push(date_col)
        .push(safe_line_col)
        .push(content_col)
        .into()
}

// Modal de Evento
pub fn render_event_modal<'a>(t: ui::Tokens, editor: &'a crate::app::EventEditor, locations: &'a [Location]) -> E<'a> {
    let title = if editor.id.is_some() { "Edit Event" } else { "Add Event" };

    let title_input = text_input("Event Title", &editor.title)
        .on_input(|v| Message::Timeline(TimelineMessage::TitleChanged(v)))
        .padding(10).style(ui::input_style(t));

    let year_input = text_input("Sort Year", &editor.year_input)
        .on_input(|v| Message::Timeline(TimelineMessage::YearChanged(v)))
        .padding(10).style(ui::input_style(t)).width(Length::Fixed(100.0));

    let display_date_input = text_input("Display Date", &editor.display_date)
        .on_input(|v| Message::Timeline(TimelineMessage::DisplayDateChanged(v)))
        .padding(10).style(ui::input_style(t));

    let kind_input = text_input("Type (e.g. Battle)", &editor.kind)
        .on_input(|v| Message::Timeline(TimelineMessage::KindChanged(v)))
        .padding(10).style(ui::input_style(t)).width(Length::Fixed(150.0));

    let color_input = text_input("Color (#Hex)", &editor.color)
        .on_input(|v| Message::Timeline(TimelineMessage::ColorChanged(v)))
        .padding(10).style(ui::input_style(t)).width(Length::Fixed(100.0));

    let loc_picker = pick_list(
        locations,
        editor.location.clone(),
        |loc| Message::Timeline(TimelineMessage::LocationChanged(Some(loc)))
    ).placeholder("Location...").width(Length::Fill).padding(10);

    let imp_picker = pick_list(
        vec!["Major", "Minor", "Normal"],
        Some(editor.importance.as_str()),
        |v| Message::Timeline(TimelineMessage::ImportanceChanged(v.to_string()))
    ).width(Length::Fixed(100.0)).padding(10);

    let desc_input = text_editor(&editor.description)
        .on_action(|v| Message::Timeline(TimelineMessage::DescriptionChanged(v)))
        .padding(10).height(Length::Fixed(120.0)).style(ui::text_editor_style(t));

    let form = Column::new().spacing(16)
        .push(text(title).size(20).color(t.foreground))
        .push(Column::new().spacing(6).push(text("Title").size(12).color(t.muted_fg)).push(title_input))
        .push(Row::new().spacing(10)
            .push(Column::new().spacing(6).push(text("Year").size(12).color(t.muted_fg)).push(year_input))
            .push(Column::new().spacing(6).push(text("Importance").size(12).color(t.muted_fg)).push(imp_picker))
            .push(Column::new().spacing(6).push(text("Display Date").size(12).color(t.muted_fg)).push(display_date_input))
        )
        .push(Row::new().spacing(10)
            .push(Column::new().spacing(6).push(text("Type").size(12).color(t.muted_fg)).push(kind_input))
            .push(Column::new().spacing(6).push(text("Color").size(12).color(t.muted_fg)).push(color_input))
            .push(Column::new().spacing(6).push(text("Location").size(12).color(t.muted_fg)).push(loc_picker))
        )
        .push(Column::new().spacing(6).push(text("Description").size(12).color(t.muted_fg)).push(desc_input))
        .push(Row::new().spacing(10)
            .push(ui::primary_button(t, "Save".to_string(), Message::Timeline(TimelineMessage::EditorSaveEvent)))
            .push(ui::ghost_button(t, "Cancel".to_string(), Message::Timeline(TimelineMessage::EditorCancel)))
        );

    container(container(form).width(Length::Fixed(650.0)).padding(24).style(move |_: &Theme| {
        let mut s = ui::container_style(t.popover, t.foreground);
        s.border = border::Border { color: t.border, width: 1.0, radius: 12.0.into() };
        s.shadow = iced::Shadow { color: Color::BLACK, offset: Vector::new(0.0, 10.0), blur_radius: 40.0 };
        s
    })).width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill)
        .style(move |_: &Theme| ui::container_style(Color::from_rgba8(0,0,0, 0.7), t.foreground)).into()
}

// Modal de Era
pub fn render_era_modal<'a>(t: ui::Tokens, editor: &'a crate::app::EraEditor) -> E<'a> {
    let title = if editor.id.is_some() { "Edit Era" } else { "New Era" };

    let name_input = text_input("Era Name", &editor.name)
        .on_input(|v| Message::Timeline(TimelineMessage::EraNameChanged(v)))
        .padding(10).style(ui::input_style(t));

    let start_input = text_input("Start Year", &editor.start_input)
        .on_input(|v| Message::Timeline(TimelineMessage::EraStartChanged(v)))
        .padding(10).style(ui::input_style(t)).width(Length::Fixed(120.0));

    let end_input = text_input("End Year (Empty=Present)", &editor.end_input)
        .on_input(|v| Message::Timeline(TimelineMessage::EraEndChanged(v)))
        .padding(10).style(ui::input_style(t)).width(Length::Fixed(180.0));

    let color_input = text_input("Color (#Hex)", &editor.color)
        .on_input(|v| Message::Timeline(TimelineMessage::EraColorChanged(v)))
        .padding(10).style(ui::input_style(t)).width(Length::Fixed(120.0));

    let desc_input = text_editor(&editor.description)
        .on_action(|v| Message::Timeline(TimelineMessage::EraDescChanged(v)))
        .padding(10).height(Length::Fixed(100.0)).style(ui::text_editor_style(t));

    let form = Column::new().spacing(16)
        .push(text(title).size(20).color(t.foreground))
        .push(Column::new().spacing(6).push(text("Name").size(12).color(t.muted_fg)).push(name_input))
        .push(Row::new().spacing(10)
            .push(Column::new().spacing(6).push(text("Start Year").size(12).color(t.muted_fg)).push(start_input))
            .push(Column::new().spacing(6).push(text("End Year").size(12).color(t.muted_fg)).push(end_input))
            .push(Column::new().spacing(6).push(text("Color").size(12).color(t.muted_fg)).push(color_input))
        )
        .push(Column::new().spacing(6).push(text("Description").size(12).color(t.muted_fg)).push(desc_input))
        .push(Row::new().spacing(10)
            .push(ui::primary_button(t, "Save Era".to_string(), Message::Timeline(TimelineMessage::EditorSaveEra)))
            .push(ui::ghost_button(t, "Cancel".to_string(), Message::Timeline(TimelineMessage::EditorCancel)))
        );

    container(container(form).width(Length::Fixed(600.0)).padding(24).style(move |_: &Theme| {
        let mut s = ui::container_style(t.popover, t.foreground);
        s.border = border::Border { color: t.border, width: 1.0, radius: 12.0.into() };
        s.shadow = iced::Shadow { color: Color::BLACK, offset: Vector::new(0.0, 10.0), blur_radius: 40.0 };
        s
    })).width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill)
        .style(move |_: &Theme| ui::container_style(Color::from_rgba8(0,0,0, 0.7), t.foreground)).into()
}
