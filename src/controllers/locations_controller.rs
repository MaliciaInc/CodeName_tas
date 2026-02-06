use crate::app::{AppState, LocationsMessage, LocationEditor};
use crate::model::Location;
use crate::state::{DbAction, ToastKind};
use uuid::Uuid;
use std::time::Instant;

pub fn update(state: &mut AppState, message: LocationsMessage) {
    match message {
        LocationsMessage::Open(universe_id) => {
            state.location_editor = None;
            state.route = crate::app::Route::Locations { universe_id };
        }
        LocationsMessage::EditorOpenCreate(parent_id) => {
            state.location_editor = Some(LocationEditor::create_new(parent_id));
        }
        LocationsMessage::ToggleExpand(id) => {
            if state.expanded_locations.contains(&id) { state.expanded_locations.remove(&id); } else { state.expanded_locations.insert(id); }
        }
        LocationsMessage::Select(id) => {
            state.selected_location = Some(id.clone());
            let now = Instant::now();
            if let Some((last_id, last_time)) = &state.last_location_click {
                if *last_id == id && now.duration_since(*last_time).as_millis() < 500 {
                    update(state, LocationsMessage::CardDoubleClicked(id)); return;
                }
            }
            state.last_location_click = Some((id, now));
        }
        LocationsMessage::CardDoubleClicked(id) => {
            if let Some(loc) = state.locations.iter().find(|l| l.id == id) {
                state.location_editor = Some(LocationEditor::from_location(loc));
            }
            state.last_location_click = None;
        }
        LocationsMessage::CardClicked(_) => {},
        LocationsMessage::EditorCancel => state.location_editor = None,

        LocationsMessage::EditorSave => {
            if let Some(editor) = state.location_editor.take() {
                if !editor.name.trim().is_empty() {
                    let universe_id = match &state.route { crate::app::Route::Locations { universe_id } => universe_id.clone(), _ => return };
                    let loc = Location {
                        id: editor.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
                        universe_id: universe_id.clone(),
                        parent_id: editor.parent_id,
                        name: editor.name,
                        description: editor.description.text(),
                        kind: editor.kind,
                    };
                    if let Some(pid) = &loc.parent_id { state.expanded_locations.insert(pid.clone()); }

                    state.queue(DbAction::SaveLocation(loc));
                    state.show_toast("Location saved", ToastKind::Success);
                } else {
                    state.location_editor = Some(editor);
                    state.show_toast("Name cannot be empty", ToastKind::Error);
                }
            }
        }

        LocationsMessage::Delete(id) => {
            state.pending_confirm = Some(crate::state::ConfirmAction::DeleteLocation(id));
        },

        LocationsMessage::NameChanged(v) => if let Some(e) = state.location_editor.as_mut() { e.name = v },
        LocationsMessage::KindChanged(v) => if let Some(e) = state.location_editor.as_mut() { e.kind = v },
        LocationsMessage::DescriptionChanged(action) => if let Some(e) = state.location_editor.as_mut() { e.description.perform(action) },
    }
}