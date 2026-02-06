use iced::{Element, Point, Theme};

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

#[derive(Clone, Debug, PartialEq)]
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

impl Route {
    #[allow(dead_code)]
    pub fn header_title(&self) -> &'static str {
        ""
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum PmState {
    Idle,
    Dragging {
        card_id: String,
        original_col: String,
        drag_start: Point,
        current_cursor: Point,
        active: bool,
    },
    Editing {
        card_id: Option<String>,
        column_id: String,
        title: String,
        description: iced::widget::text_editor::Content,
        priority: String,
    },
}

impl Default for PmState {
    fn default() -> Self {
        Self::Idle
    }
}

// --- VIEW FACADE (Router-only behavior) ---
pub fn view(state: &AppState) -> Element<'_, Message> {
    ui_shell::view(state)
}
