use iced::widget::text_editor;
use crate::model::{Creature, Location, TimelineEvent, TimelineEra};

// ============================================
// REFACTOR C.1: Editors store IDs instead of full structs
// ============================================

#[derive(Debug, Clone)]
pub struct CreatureEditor {
    pub index: Option<usize>,
    pub id: Option<String>,
    pub name: String,
    pub kind: String,
    pub habitat: String,
    pub description: text_editor::Content,
    pub danger: String,
    pub home_location_id: Option<String>, // ✅ C.1: ID only instead of Option<Location>
}

impl CreatureEditor {
    pub fn create_new() -> Self {
        Self {
            index: None,
            id: None,
            name: String::new(),
            kind: String::new(),
            habitat: String::new(),
            description: text_editor::Content::new(),
            danger: "Medium".to_string(),
            home_location_id: None, // ✅ C.1
        }
    }

    // ✅ C.1: No need to search Location, just store ID directly
    pub fn from_creature(index: usize, c: &Creature, _all_locations: &[Location]) -> Self {
        Self {
            index: Some(index),
            id: Some(c.id.clone()),
            name: c.name.clone(),
            kind: c.kind.clone(),
            habitat: c.habitat.clone(),
            description: text_editor::Content::with_text(&c.description),
            danger: c.danger.clone(),
            home_location_id: c.home_location_id.clone(), // ✅ C.1: Direct copy
        }
    }

    pub fn into_creature(self) -> Creature {
        Creature {
            id: self.id.clone().unwrap_or_default(),
            name: self.name.clone(),
            kind: self.kind.clone(),
            habitat: self.habitat.clone(),
            description: self.description.text(),
            danger: self.danger.clone(),
            home_location_id: self.home_location_id, // ✅ C.1: Direct usage
            archived: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocationEditor {
    pub id: Option<String>,
    pub parent_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub description: text_editor::Content,
}

impl LocationEditor {
    pub fn create_new(parent_id: Option<String>) -> Self {
        Self {
            id: None,
            parent_id,
            name: String::new(),
            kind: "Place".to_string(),
            description: text_editor::Content::new(),
        }
    }

    pub fn from_location(l: &Location) -> Self {
        Self {
            id: Some(l.id.clone()),
            parent_id: l.parent_id.clone(),
            name: l.name.clone(),
            kind: l.kind.clone(),
            description: text_editor::Content::with_text(&l.description),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventEditor {
    pub id: Option<String>,
    pub title: String,
    pub year_input: String,
    pub display_date: String,
    pub importance: String,
    pub kind: String,
    pub color: String,
    pub location_id: Option<String>, // ✅ C.1: ID only instead of Option<Location>
    pub description: text_editor::Content,
}

impl EventEditor {
    pub fn create_new(default_year: Option<i64>) -> Self {
        Self {
            id: None,
            title: String::new(),
            year_input: default_year.unwrap_or(0).to_string(),
            display_date: String::new(),
            importance: "Normal".to_string(),
            kind: "General".to_string(),
            color: "#A1A1AA".to_string(),
            location_id: None, // ✅ C.1
            description: text_editor::Content::new(),
        }
    }

    // ✅ C.1: No need to search Location, just store ID directly
    pub fn from_event(e: &TimelineEvent, _all_locations: &[Location]) -> Self {
        Self {
            id: Some(e.id.clone()),
            title: e.title.clone(),
            year_input: e.year.to_string(),
            display_date: e.display_date.clone(),
            importance: e.importance.clone(),
            kind: e.kind.clone(),
            color: e.color.clone(),
            location_id: e.location_id.clone(), // ✅ C.1: Direct copy
            description: text_editor::Content::with_text(&e.description),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EraEditor {
    pub id: Option<String>,
    pub name: String,
    pub start_input: String,
    pub end_input: String,
    pub color: String,
    pub description: text_editor::Content,
}

impl EraEditor {
    pub fn create_new() -> Self {
        Self {
            id: None,
            name: String::new(),
            start_input: "0".to_string(),
            end_input: "".to_string(),
            color: "#6366F1".to_string(),
            description: text_editor::Content::new(),
        }
    }

    pub fn from_era(e: &TimelineEra) -> Self {
        Self {
            id: Some(e.id.clone()),
            name: e.name.clone(),
            start_input: e.start_year.to_string(),
            end_input: e.end_year.map(|y| y.to_string()).unwrap_or_default(),
            color: e.color.clone(),
            description: text_editor::Content::with_text(&e.description),
        }
    }
}