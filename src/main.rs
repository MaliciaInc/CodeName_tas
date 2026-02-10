#![windows_subsystem = "windows"]

mod app;
pub mod db;
mod db_seed;
mod logger;
mod model;
mod pages;
mod ui;
mod controllers;
mod project_manager;
mod messages;
mod state;
mod editors;
mod guards;

// ✅ Draft Recovery (Forge) - módulo a nivel de crate root
mod forge_draft;

pub fn main() -> iced::Result {
    controllers::ui_controller::run()
}
