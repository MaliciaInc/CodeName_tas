use iced::{Alignment, Color, Length, Vector};
use iced::widget::{button, container, mouse_area, text, text_input, text_editor, Column, Row, Space};
use iced::Theme;
use crate::app::{AppState, Message, LocationsMessage};
use crate::model::Location;
use crate::{pages::E, ui};

pub fn locations<'a>(state: &'a AppState, t: ui::Tokens, universe_id: &'a str) -> E<'a> {
    let universe_name = state.universes.iter().find(|u| u.id == universe_id)
        .map(|u| u.name.as_str()).unwrap_or("Unknown");

    let header = Row::new().align_y(Alignment::Center)
        .push(
            Column::new()
                .push(text(format!("Geography — {}", universe_name)).size(26).color(t.foreground))
                .push(text("Manage locations, maps, and points of interest.").size(12).color(t.muted_fg))
        )
        .push(Space::new().width(Length::Fill))
        .push(ui::outline_button(t, "Back".to_string(), Message::BackToUniverse(universe_id.to_string())))
        .push(ui::primary_button(t, "Add New Location".to_string(), Message::Locations(LocationsMessage::EditorOpenCreate(None))));

    // ✅ REFACTOR A.2: Use cached children_map instead of rebuilding O(n) every render
    let tree_items = build_visual_tree_optimized(state, &state.expanded_locations);

    let mut list = Column::new().spacing(4);

    if tree_items.is_empty() {
        list = list.push(ui::card(t, text("No locations found.").size(14).color(t.muted_fg).into()));
    } else {
        for (loc, depth) in tree_items {
            // ✅ REFACTOR A.2: Use cached map for O(1) check
            let has_children = !state.get_location_children(&Some(loc.id.clone())).is_empty();
            let is_expanded = state.expanded_locations.contains(&loc.id);
            let is_selected = state.selected_location.as_ref() == Some(&loc.id);

            list = list.push(location_node(t, loc, depth, has_children, is_expanded, is_selected));
        }
    }

    let content = Column::new().spacing(20).push(header).push(list);
    ui::page_padding(content.into())
}


// ✅ REFACTOR A.2: Use cached map from AppState
fn build_visual_tree_optimized<'a>(
    state: &'a crate::app::AppState,
    expanded: &std::collections::HashSet<String>
) -> Vec<(&'a Location, usize)> {
    let mut result = Vec::new();
    process_node_optimized(state, &None, 0, &mut result, expanded);
    result
}

// ✅ REFACTOR A.2: Use O(1) lookups via state.locations_children_map
fn process_node_optimized<'a>(
    state: &'a crate::app::AppState,
    parent_id: &Option<String>,
    depth: usize,
    result: &mut Vec<(&'a Location, usize)>,
    expanded: &std::collections::HashSet<String>
) {
    // O(1) lookup using cached map
    let child_ids = state.get_location_children(parent_id);

    if child_ids.is_empty() {
        return;
    }

    // Get actual Location objects and sort by name
    let mut children: Vec<&Location> = child_ids
        .iter()
        .filter_map(|id| state.locations.iter().find(|l| &l.id == *id))
        .collect();

    children.sort_by_key(|l| &l.name);

    for child in children {
        result.push((child, depth));
        if expanded.contains(&child.id) {
            process_node_optimized(state, &Some(child.id.clone()), depth + 1, result, expanded);
        }
    }
}

fn location_node<'a>(
    t: ui::Tokens,
    loc: &'a Location,
    depth: usize,
    has_children: bool,
    is_expanded: bool,
    is_selected: bool
) -> E<'a> {
    // Solo clonamos lo que NECESITA ownership para mensajes
    let id = loc.id.clone();

    let toggle_icon = if has_children {
        let icon_text = if is_expanded { "-" } else { "+" };
        button(text(icon_text).size(14).color(t.muted_fg))
            .padding([2, 8])
            .style(crate::ui::ghost_button_style(t))
            .on_press(Message::Locations(LocationsMessage::ToggleExpand(id.clone())))
    } else {
        button(text(" ").size(14))
            .padding([2, 8])
            .style(crate::ui::ghost_button_style(t))
    };

    // ✅ Ya no clonamos kind/name: usamos referencias (requiere lifetime 'a, por eso el cambio)
    let kind_pill = container(text(&loc.kind).size(10).color(t.foreground))
        .padding([2, 6])
        .style(move |_| {
            let mut s = ui::container_style(t.active_bg, t.foreground);
            s.border.radius = 999.0.into();
            s
        });

    let main_info = Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(text(&loc.name).size(16).color(t.foreground))
        .push(kind_pill);

    let actions = Row::new()
        .spacing(4)
        .push(ui::ghost_button(
            t,
            "+ Sub".to_string(),
            Message::Locations(LocationsMessage::EditorOpenCreate(Some(id.clone()))),
        ))
        .push(ui::danger_button(
            t,
            "×".to_string(),
            Message::Locations(LocationsMessage::Delete(id.clone())),
        ));

    let card_content = Row::new()
        .align_y(Alignment::Center)
        .spacing(8)
        .push(toggle_icon)
        .push(main_info)
        .push(Space::new().width(Length::Fill))
        .push(actions);

    let bg_color = if is_selected {
        crate::ui::alpha(t.foreground, 0.08)
    } else if depth == 0 {
        t.card
    } else {
        crate::ui::alpha(t.card, 0.6)
    };

    let border_color = if is_selected {
        crate::ui::alpha(t.muted_fg, 0.3)
    } else {
        t.border
    };

    let shadow = if depth == 0 {
        iced::Shadow {
            color: Color::BLACK,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 8.0,
        }
    } else {
        iced::Shadow::default()
    };

    let card_container = container(card_content)
        .padding([8, 12])
        .width(Length::Fill)
        .style(move |_: &Theme| {
            let mut s = ui::container_style(bg_color, t.foreground);
            s.border.color = border_color;
            s.border.width = 1.0;
            s.border.radius = 6.0.into();
            s.shadow = shadow;
            s
        });

    let indent_size = 28.0 * (depth as f32);

    let row = Row::new()
        .push(Space::new().width(Length::Fixed(indent_size)))
        .push(card_container);

    mouse_area(row)
        .on_press(Message::Locations(LocationsMessage::CardClicked(id)))
        .into()
}


pub fn render_location_modal<'a>(t: ui::Tokens, editor: &'a crate::app::LocationEditor) -> E<'a> {
    let title = if editor.id.is_some() { "Edit Location" } else if editor.parent_id.is_some() { "Add Sub-Location" } else { "Add New Location" };

    let name_input = text_input("Location Name", &editor.name)
        .on_input(|v| Message::Locations(LocationsMessage::NameChanged(v)))
        .padding(10).style(ui::input_style(t));

    let kind_input = text_input("Type (e.g. City, Region)", &editor.kind)
        .on_input(|v| Message::Locations(LocationsMessage::KindChanged(v)))
        .padding(10).style(ui::input_style(t));

    let desc_input = text_editor(&editor.description)
        .on_action(|v| Message::Locations(LocationsMessage::DescriptionChanged(v)))
        .padding(10).height(Length::Fixed(150.0)).style(ui::text_editor_style(t));

    let form = Column::new().spacing(16)
        .push(text(title).size(20).color(t.foreground))
        .push(Column::new().spacing(6).push(text("Name").size(12).color(t.muted_fg)).push(name_input))
        .push(Column::new().spacing(6).push(text("Type").size(12).color(t.muted_fg)).push(kind_input))
        .push(Column::new().spacing(6).push(text("Description").size(12).color(t.muted_fg)).push(desc_input))
        .push(Row::new().spacing(10).push(ui::primary_button(t, "Save".to_string(), Message::Locations(LocationsMessage::EditorSave))).push(ui::ghost_button(t, "Cancel".to_string(), Message::Locations(LocationsMessage::EditorCancel))));

    container(container(form).width(Length::Fixed(550.0)).padding(24).style(move |_: &Theme| {
        let mut s = ui::container_style(t.popover, t.foreground);
        s.border.color = t.border; s.border.width = 1.0; s.border.radius = 12.0.into();
        s.shadow = iced::Shadow { color: Color::BLACK, offset: Vector::new(0.0, 10.0), blur_radius: 40.0 };
        s
    })).width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill)
        .style(move |_: &Theme| ui::container_style(Color::from_rgba8(0,0,0, 0.7), t.foreground)).into()
}