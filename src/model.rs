use sqlx::FromRow;
use std::fmt;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

// --- PROJECT IDENTITY ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectKind {
    Universe,
    Novel,
    Board,
}

impl Default for ProjectKind {
    fn default() -> Self {
        Self::Universe
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub last_opened: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Project {
    pub fn get_kind(&self) -> ProjectKind {
        if self.path.ends_with(".novel") {
            ProjectKind::Novel
        } else if self.path.ends_with(".pmboard") {
            ProjectKind::Board
        } else {
            ProjectKind::Universe
        }
    }
}

// --- UNIVERSE & BESTIARY ---
#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct Universe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub archived: bool,
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct Creature {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub habitat: String,
    pub description: String,
    pub danger: String,
    pub home_location_id: Option<String>,
    #[sqlx(default)]
    pub archived: bool,
}

// --- LOCATIONS ---
#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub universe_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub description: String,
    pub kind: String,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

// --- TIMELINE ---
#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct TimelineEra {
    pub id: String,
    pub universe_id: String,
    pub name: String,
    pub start_year: i64,
    pub end_year: Option<i64>,
    pub description: String,
    pub color: String,
}

#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub universe_id: String,
    pub title: String,
    pub description: String,
    pub year: i64,
    pub display_date: String,
    pub importance: String,
    pub kind: String,
    pub color: String,
    pub location_id: Option<String>,
}

// --- PM TOOLS (KANBAN) ---
#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct Board {
    pub id: String,
    pub name: String,
    #[allow(dead_code)]
    pub kind: String,
}

#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct BoardColumn {
    pub id: String,
    #[allow(dead_code)]
    pub board_id: String,
    pub name: String,
    pub position: i32,
}

#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub column_id: String,
    pub title: String,
    pub description: String,
    pub position: i64,
    #[sqlx(default)]
    pub priority: String,
}

#[derive(Debug, Clone)]
pub struct KanbanBoardData {
    pub board: Board,
    pub columns: Vec<(BoardColumn, Vec<Card>)>,
}

// --- THE FORGE (NARRATIVE) ---
#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub chapter_id: String,
    pub title: String,
    #[sqlx(default)]
    pub body: String,
    pub position: i64,
    #[sqlx(default)]
    pub status: String,
    #[sqlx(default)]
    pub word_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct Novel {
    pub id: String,
    pub universe_id: Option<String>,
    pub title: String,
    #[sqlx(default)]
    pub synopsis: String,
    #[sqlx(default)]
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub novel_id: String,
    pub title: String,
    pub position: i64,
    #[sqlx(default)]
    pub synopsis: String,
    #[sqlx(default)]
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- SNAPSHOTS ---
#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct UniverseSnapshot {
    pub id: String,
    pub universe_id: String,
    pub name: String,
    pub created_at: i64,   // epoch seconds (sqlite unixepoch)
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseSnapshotPayload {
    pub universe: Universe,
    pub creatures: Vec<Creature>,
    pub locations: Vec<Location>,
    pub timeline_eras: Vec<TimelineEra>,
    pub timeline_events: Vec<TimelineEvent>,
    pub pm_cards: Vec<Card>,
}

// --- TRASH SYSTEM ---
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TrashEntry {
    pub id: String,
    pub deleted_at: DateTime<Utc>,
    pub target_type: String,
    pub target_id: String,
    pub parent_type: Option<String>,
    pub parent_id: Option<String>,
    pub display_name: String,
    pub display_info: Option<String>,
    pub payload_json: String,
}

impl TrashEntry {
    pub fn deleted_at_formatted(&self) -> String {
        self.deleted_at.format("%Y-%m-%d %H:%M").to_string()
    }
}

// --- AUDIT LOG ---
#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub ts: i64, // unixepoch seconds
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    #[sqlx(default)]
    pub details_json: String,
}

// --- RELATIONSHIPS ---
#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct RelationshipType {
    pub id: String,
    pub name: String,
    #[sqlx(default)]
    pub description: String,
    pub directed: i64, // 0/1 (sqlite)
}

#[derive(Debug, Clone, FromRow, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub relationship_type_id: String,
    pub from_type: String,
    pub from_id: String,
    pub to_type: String,
    pub to_id: String,
    #[sqlx(default)]
    pub note: String,
    pub created_at: i64, // unixepoch seconds
}