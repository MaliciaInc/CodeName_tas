use iced::{mouse, Event};

use crate::app::{AppState, Message};

/// Translates low-level global events into app Messages.
/// Keeps ui_controller clean and isolates input concerns.
pub fn translate_global_event(state: &AppState, message: &Message) -> Option<Message> {
    // Only relevant when a project is open.
    if state.active_project.is_none() {
        return None;
    }

    match message {
        Message::GlobalEvent(Event::Mouse(mouse::Event::CursorMoved { position })) => {
            Some(Message::MouseMoved(*position))
        }

        Message::GlobalEvent(Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))) => {
            Some(Message::MouseReleased)
        }

        _ => None,
    }
}
