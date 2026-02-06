use iced::widget::text;
use crate::app::AppState;
use crate::{ui, pages::E};

pub fn workspaces_stub<'a>(_state: &'a AppState, t: ui::Tokens) -> E<'a> {
    ui::page_padding(ui::card(
        t,
        text("Workspaces (stub)").size(14).color(t.muted_fg).into(),
    ))
}

pub fn timeline_stub<'a>(_state: &'a AppState, t: ui::Tokens, _universe_id: &'a str) -> E<'a> {
    ui::page_padding(ui::card(
        t,
        text("Timeline (stub)").size(14).color(t.muted_fg).into(),
    ))
}

pub fn forge_stub<'a>(_state: &'a AppState, t: ui::Tokens) -> E<'a> {
    ui::page_padding(ui::card(
        t,
        text("The Forge (stub)").size(14).color(t.muted_fg).into(),
    ))
}

// PM STUB ELIMINADO AQUÍ

pub fn assets_stub<'a>(_state: &'a AppState, t: ui::Tokens) -> E<'a> {
    ui::page_padding(ui::card(
        t,
        text("Assets (stub)").size(14).color(t.muted_fg).into(),
    ))
}

pub fn account_stub<'a>(_state: &'a AppState, t: ui::Tokens) -> E<'a> {
    ui::page_padding(ui::card(
        t,
        text("Account (stub)").size(14).color(t.muted_fg).into(),
    ))
}