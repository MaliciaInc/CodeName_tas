use iced::{mouse, Event};
use std::time::Instant;

use crate::app::{AppState, Message, PmState};

/// Throttle del cursor durante drag activo.
/// 16ms ≈ 60Hz (suficiente para “sentirse” fluido y reduce spam de eventos).
const DRAG_CURSOR_THROTTLE_MS: u128 = 16;

/// Translates low-level global events into app Messages.
/// Mantiene ui_controller limpio y encapsula concerns de input.
///
/// Nota: ahora recibe &mut AppState porque aplicamos throttle con estado del drag.
pub fn translate_global_event(state: &mut AppState, message: &Message) -> Option<Message> {
    // Only relevant when a project is open.
    if state.active_project.is_none() {
        return None;
    }

    match message {
        Message::GlobalEvent(Event::Mouse(mouse::Event::CursorMoved { position })) => {
            // ✅ CB-3: Si estamos en drag activo, reducimos la frecuencia de MouseMoved.
            if let PmState::Dragging {
                active: true,
                last_cursor_emit,
                ..
            } = &mut state.pm_state
            {
                let now = Instant::now();
                if now.duration_since(*last_cursor_emit).as_millis() < DRAG_CURSOR_THROTTLE_MS {
                    return None;
                }
                *last_cursor_emit = now;
            }

            Some(Message::MouseMoved(*position))
        }

        Message::GlobalEvent(Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))) => {
            Some(Message::MouseReleased)
        }

        _ => None,
    }
}
