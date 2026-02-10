use crate::app::{AppState, TimelineMessage, EventEditor, EraEditor};
use crate::model::{TimelineEvent, TimelineEra};
use crate::state::{DbAction, ToastKind};
use uuid::Uuid;
use std::time::Instant;


pub fn update(state: &mut AppState, message: TimelineMessage) {
    match message {
        TimelineMessage::Open(universe_id) => {
            state.event_editor = None;
            state.era_editor = None;
            state.route = crate::app::Route::Timeline { universe_id };
        }

        // --- EVENT ACTIONS ---
        TimelineMessage::EditorOpenCreateEvent(default_year) => state.event_editor = Some(EventEditor::create_new(default_year)),
        TimelineMessage::EditEvent(id) => if let Some(evt) = state.timeline_events.iter().find(|e| e.id == id) { state.event_editor = Some(EventEditor::from_event(evt, &state.locations)); },
        TimelineMessage::CardClicked(id) => {
            let now = Instant::now();
            if let Some((last_id, last_time)) = &state.last_timeline_click {
                if *last_id == id && now.duration_since(*last_time).as_millis() < 500 { update(state, TimelineMessage::EditEvent(id.clone())); state.last_timeline_click = None; return; }
            }
            state.last_timeline_click = Some((id, now));
        }
        TimelineMessage::EditorCancel => { state.event_editor = None; state.era_editor = None; },

        TimelineMessage::EditorSaveEvent => {
            if let Some(editor) = state.event_editor.take() {
                if !editor.title.trim().is_empty() {
                    let _year = editor.year_input.parse::<i64>().unwrap_or(0);
                    let evt = TimelineEvent {
                        id: editor.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
                        universe_id: match &state.route {
                            crate::app::Route::Timeline { universe_id } => universe_id.clone(),
                            _ => return,
                        },
                        title: editor.title,
                        description: editor.description.text(),
                        year: editor.year_input.parse().unwrap_or(0),
                        display_date: editor.display_date,
                        importance: editor.importance,
                        kind: editor.kind,
                        color: editor.color,
                        location_id: editor.location_id, // ✅ C.1: Direct ID usage
                    };
                    state.queue(DbAction::SaveEvent(evt));
                    state.show_toast("Event saved", ToastKind::Success);
                } else { state.event_editor = Some(editor); }
            }
        }
        TimelineMessage::DeleteEvent(id) => {
            state.pending_confirm = Some(crate::state::ConfirmAction::DeleteEvent(id));
        }

        // --- EVENT INPUTS ---
        TimelineMessage::TitleChanged(v) => if let Some(e) = state.event_editor.as_mut() { e.title = v },
        TimelineMessage::YearChanged(v) => if let Some(e) = state.event_editor.as_mut() { if v.chars().all(|c| c.is_ascii_digit() || c == '-') { e.year_input = v; } },
        TimelineMessage::DisplayDateChanged(v) => if let Some(e) = state.event_editor.as_mut() { e.display_date = v },
        TimelineMessage::ImportanceChanged(v) => if let Some(e) = state.event_editor.as_mut() { e.importance = v },
        TimelineMessage::KindChanged(v) => if let Some(e) = state.event_editor.as_mut() { e.kind = v },
        TimelineMessage::ColorChanged(v) => if let Some(e) = state.event_editor.as_mut() { e.color = v },
        TimelineMessage::LocationChanged(loc_id) => if let Some(e) = state.event_editor.as_mut() { e.location_id = loc_id }, // ✅ C.1: ID only
        TimelineMessage::DescriptionChanged(action) => if let Some(e) = state.event_editor.as_mut() { e.description.perform(action) },

        // --- ERA ACTIONS ---
        TimelineMessage::EditorOpenCreateEra => state.era_editor = Some(EraEditor::create_new()),
        TimelineMessage::EditEra(id) => if let Some(era) = state.timeline_eras.iter().find(|e| e.id == id) { state.era_editor = Some(EraEditor::from_era(era)); },
        TimelineMessage::EraBannerClicked(id) => update(state, TimelineMessage::EditEra(id)),
        TimelineMessage::DeleteEra(id) => {
            state.pending_confirm = Some(crate::state::ConfirmAction::DeleteEra(id));
        }

        TimelineMessage::EditorSaveEra => {
            if let Some(editor) = state.era_editor.take() {
                if !editor.name.trim().is_empty() {
                    let _start = editor.start_input.parse::<i64>().unwrap_or(0);
                    let _end = if editor.end_input.trim().is_empty() { None } else { editor.end_input.parse::<i64>().ok() };
                    let era = TimelineEra {
                        id: editor.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
                        universe_id: match &state.route {
                            crate::app::Route::Timeline { universe_id } => universe_id.clone(),
                            _ => return,
                        },
                        name: editor.name,
                        start_year: editor.start_input.parse().unwrap_or(0),
                        end_year: editor.end_input.parse().ok(),
                        description: editor.description.text(),
                        color: editor.color,
                    };
                    state.queue(DbAction::SaveEra(era));
                    state.show_toast("Era saved", ToastKind::Success);
                } else { state.era_editor = Some(editor); }
            }
        }

        // --- ERA INPUTS ---
        TimelineMessage::EraNameChanged(v) => if let Some(e) = state.era_editor.as_mut() { e.name = v },
        TimelineMessage::EraStartChanged(v) => if let Some(e) = state.era_editor.as_mut() { if v.chars().all(|c| c.is_ascii_digit() || c == '-') { e.start_input = v; } },
        TimelineMessage::EraEndChanged(v) => if let Some(e) = state.era_editor.as_mut() { if v.chars().all(|c| c.is_ascii_digit() || c == '-') { e.end_input = v; } },
        TimelineMessage::EraColorChanged(v) => if let Some(e) = state.era_editor.as_mut() { e.color = v },
        TimelineMessage::EraDescChanged(action) => if let Some(e) = state.era_editor.as_mut() { e.description.perform(action) },
    }
}