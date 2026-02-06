use iced::{Alignment, Color, Length, Vector};
use iced::widget::{button, container, mouse_area, text, text_input, text_editor, pick_list, Column, Row};
use iced::Theme;
use crate::app::{AppState, Message, BestiaryMessage};
use crate::model::{Creature, Location};
use crate::{pages::E, ui};

pub fn bestiary<'a>(state: &'a AppState, t: ui::Tokens, universe_id: &'a str) -> E<'a> {
    let universe_name = state.universes.iter().find(|u| u.id == universe_id).map(|u| u.name.as_str()).unwrap_or(universe_id);

    // C12: no alocar Vecs por render; iteramos una sola vez por sección
    let (active_grid, active_count) = creatures_grid_counted(
        t,
        state.creatures.iter().enumerate().filter(|(_, c)| !c.archived),
        &state.locations,
        universe_id,
    );

    let (archived_grid, archived_count) = creatures_grid_counted(
        t,
        state.creatures.iter().enumerate().filter(|(_, c)| c.archived),
        &state.locations,
        universe_id,
    );

    let header_left = Column::new().spacing(4)
        .push(text(format!("Bestiary — {}", universe_name)).size(26).color(t.foreground))
        .push(text("Creatures, entities and beings that inhabit this universe.").size(12).color(t.muted_fg));

    let header_right = Row::new().spacing(10)
        .push(ui::outline_button(t, "Back to universe".to_string(), Message::BackToUniverse(universe_id.to_string())))
        .push(ui::outline_button(t, "All universes".to_string(), Message::BackToUniverses))
        .push(ui::primary_button(t, "Create creature".to_string(), Message::Bestiary(BestiaryMessage::EditorOpenCreate)));

    let header = Row::new().align_y(Alignment::Center)
        .push(container(header_left).width(Length::Fill))
        .push(header_right);

    // Active Section
    // Active Section
    let active_header = text(format!("Active creatures ({})", active_count))
        .size(12)
        .color(t.muted_fg);

    // Archived Section
    let archived_header = text(format!("Archived creatures ({})", archived_count))
        .size(12)
        .color(t.muted_fg);

    let body = Column::new().spacing(14)
        .push(header)
        .push(active_header)
        .push(active_grid)
        .push(ui::h_divider(t))
        .push(archived_header)
        .push(archived_grid)
        .width(Length::Fill);

    ui::page_padding(body.into())
}

// ... (render_creature_modal y danger_pill se mantienen IGUALES)

pub fn render_creature_modal<'a>(t: ui::Tokens, editor: &'a crate::app::CreatureEditor, locations: &'a [Location]) -> E<'a> {
    let is_new = editor.index.is_none();
    let title = if is_new { "Create Creature" } else { "Edit Creature" };

    let name_input = text_input("Name", &editor.name).on_input(|v| Message::Bestiary(BestiaryMessage::NameChanged(v))).padding(10).style(ui::input_style(t));
    let kind_input = text_input("Kind / tags", &editor.kind).on_input(|v| Message::Bestiary(BestiaryMessage::KindChanged(v))).padding(10).style(ui::input_style(t));
    let habitat_input = text_input("Habitat Description", &editor.habitat).on_input(|v| Message::Bestiary(BestiaryMessage::HabitatChanged(v))).padding(10).style(ui::input_style(t));

    let location_picker = Column::new().spacing(6).push(text("Home Location (Optional)").size(12).color(t.muted_fg)).push(pick_list(locations, editor.home_location.clone(), |loc| Message::Bestiary(BestiaryMessage::LocationChanged(Some(loc)))).placeholder("Select location...").width(Length::Fill).padding(10).style(move |_, status| { let base = iced::widget::pick_list::Style { text_color: t.foreground, placeholder_color: t.muted_fg, handle_color: t.muted_fg, background: iced::Background::Color(t.input_border), border: iced::Border { color: t.border, width: 1.0, radius: 6.0.into() } }; match status { iced::widget::pick_list::Status::Opened { .. } => iced::widget::pick_list::Style { border: iced::Border { color: t.accent, ..base.border }, ..base }, _ => base } }));

    let desc_input = text_editor(&editor.description).on_action(|v| Message::Bestiary(BestiaryMessage::DescriptionChanged(v))).padding(10).height(Length::Fixed(150.0)).style(ui::text_editor_style(t));

    let danger_pills = Row::new().spacing(8).push(text("Danger Level:").size(12).color(t.muted_fg)).push(danger_pill(t, "Low", &editor.danger)).push(danger_pill(t, "Medium", &editor.danger)).push(danger_pill(t, "High", &editor.danger)).push(danger_pill(t, "Extreme", &editor.danger));

    let actions = Row::new().spacing(10).align_y(Alignment::Center).push(ui::primary_button(t, "Save Creature".to_string(), Message::Bestiary(BestiaryMessage::EditorSave))).push(ui::ghost_button(t, "Cancel".to_string(), Message::Bestiary(BestiaryMessage::EditorCancel)));

    let form = Column::new().spacing(16).push(text(title).size(20).color(t.foreground)).push(Column::new().spacing(6).push(text("Name").size(12).color(t.muted_fg)).push(name_input)).push(Row::new().spacing(10).push(Column::new().spacing(6).push(text("Kind").size(12).color(t.muted_fg)).push(kind_input).width(Length::FillPortion(1))).push(location_picker.width(Length::FillPortion(1)))).push(Column::new().spacing(6).push(text("Habitat Details").size(12).color(t.muted_fg)).push(habitat_input)).push(Column::new().spacing(6).push(text("Description").size(12).color(t.muted_fg)).push(desc_input)).push(danger_pills).push(actions);

    container(container(form).width(Length::Fixed(550.0)).padding(24).style(move |_: &Theme| { let mut s = ui::container_style(t.popover, t.foreground); s.border.color = t.border; s.border.width = 1.0; s.border.radius = 12.0.into(); s.shadow = iced::Shadow { color: Color::BLACK, offset: Vector::new(0.0, 10.0), blur_radius: 40.0 }; s })).width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill).style(move |_: &Theme| ui::container_style(Color::from_rgba8(0,0,0, 0.7), t.foreground)).into()
}

fn danger_pill<'a>(t: ui::Tokens, label: &'a str, current: &str) -> E<'a> {
    let is_selected = current == label;
    let mut btn = button(text(label).size(12).color(if is_selected { t.background } else { t.muted_fg })).padding([6, 12]).on_press(Message::Bestiary(BestiaryMessage::DangerChanged(label.to_string())));
    if is_selected { btn = btn.style(ui::primary_button_style(t)); } else { btn = btn.style(ui::ghost_button_style(t)); }
    btn.into()
}

// FIX: Firma cambiada a Vec<&'a Creature> para evitar E0515
fn creatures_grid_counted<'a, I>(
    t: ui::Tokens,
    creatures: I,
    locations: &'a [Location],
    universe_id: &'a str,
) -> (E<'a>, usize)
where
    I: IntoIterator<Item = (usize, &'a Creature)>,
{
    let mut col: Column<'a, Message> = Column::new().spacing(14).width(Length::Fill);
    let mut row: Row<'a, Message> = Row::new().spacing(14).width(Length::Fill);

    let mut in_row: usize = 0;
    let mut total: usize = 0;

    for (idx, c) in creatures.into_iter() {
        row = row.push(container(creature_card(t, idx, c, locations, universe_id)).width(Length::Fill));
        in_row += 1;
        total += 1;

        if in_row == 3 {
            col = col.push(row);
            row = Row::new().spacing(14).width(Length::Fill);
            in_row = 0;
        }
    }

    if in_row != 0 {
        col = col.push(row);
    }

    (col.into(), total)
}

fn creature_card<'a>(t: ui::Tokens, index: usize, c: &'a Creature, locations: &'a [Location], universe_id: &'a str) -> E<'a> {
    let location_info = if let Some(lid) = &c.home_location_id {
        let name = locations.iter().find(|l| l.id == *lid).map(|l| l.name.as_str()).unwrap_or("Unknown");
        button(text(format!("📍 {}", name)).size(12).color(t.accent)).padding(0).style(crate::ui::ghost_button_style(t)).on_press(Message::GoToLocation(universe_id.to_string(), lid.clone()))
    } else {
        button(text(format!("Habitat: {}", c.habitat)).size(12).color(t.muted_fg)).padding(0).style(crate::ui::ghost_button_style(t))
    };

    let actions = if c.archived {
        Row::new().spacing(10)
            .push(ui::outline_button(t, "Restore".to_string(), Message::Bestiary(BestiaryMessage::Restore(c.id.clone()))))
            .push(ui::danger_button(t, "Delete Forever".to_string(), Message::Bestiary(BestiaryMessage::Delete(c.id.clone()))))
    } else {
        Row::new().spacing(10)
            .push(ui::outline_button(t, "Archive".to_string(), Message::Bestiary(BestiaryMessage::Archive(c.id.clone()))))
            .push(ui::danger_button(t, "Delete".to_string(), Message::Bestiary(BestiaryMessage::Delete(c.id.clone()))))
    };

    let body = Column::new().spacing(6)
        .push(text(&c.name).size(16).color(t.foreground))
        .push(text(&c.kind).size(12).color(t.muted_fg))
        .push(location_info)
        .push(text(&c.description).size(12).color(t.muted_fg))
        .push(text(format!("Danger: {}", c.danger)).size(12).color(t.foreground))
        .push(actions)
        .push(text("Double-click card to edit creature.").size(10).color(Color::from_rgba8(0xA1, 0xA1, 0xA1, 0.55)));

    let card = ui::card(t, body.into());
    mouse_area(card).on_press(Message::Bestiary(BestiaryMessage::CardClicked(index))).into()
}