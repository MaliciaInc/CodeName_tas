use iced::{Element, Point, Theme};
use std::time::Instant;
use std::sync::Arc;

// IMPORTANT: Keep app.rs thin. View composition lives in ui_shell.rs.
#[path = "ui_shell.rs"]
mod ui_shell;

// --- RE-EXPORTS (Facade) ---
pub use crate::state::AppState;
pub use crate::messages::{
    Message,
    PmMessage,
    BestiaryMessage,
    UniverseMessage,
    LocationsMessage,
    TimelineMessage,
    WorkspaceMessage,
};
pub use crate::editors::{CreatureEditor, LocationEditor, EventEditor, EraEditor};

pub const APP_NAME: &str = "Titan Architect Studio";
pub const APP_ACRONYM: &str = "TAS";
pub fn app_theme(_state: &AppState) -> Theme {
    Theme::Dark
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Overview,
    Workspaces,

    UniverseList,
    UniverseDetail { universe_id: String },
    Bestiary { universe_id: String },
    Locations { universe_id: String },
    Timeline { universe_id: String },

    PmList,
    PmBoard { board_id: String },

    Forge,

    Assets,
    Account,

    Trash,
}

impl Default for Route {
    fn default() -> Self {
        Self::Overview
    }
}
// IDs internados para el loop caliente (drag/hover/render).
pub type PmId = Arc<str>;

#[derive(Debug, Clone)]
pub enum PmState {
    Idle,
    Dragging {
        card_id: PmId,
        card_title: String,        // ✅ Cache del título (evita búsquedas en view)
        original_col: PmId,
        drag_start: Point,
        current_cursor: Point,
        active: bool,

        // ✅ Throttle: marca de tiempo del último “cursor update” emitido
        // para reducir mensajes y reconstrucciones durante drag.
        last_cursor_emit: Instant,
    },
    Editing {
        card_id: Option<String>,
        column_id: String,
        title: String,
        description: iced::widget::text_editor::Content,
        priority: String,
    },
}


// --- VIEW FACADE (Router-only behavior) ---
pub fn view(state: &AppState) -> Element<'_, Message> {
    ui_shell::view(state)
}
