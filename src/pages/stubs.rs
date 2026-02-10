use iced::widget::text;
use crate::app::AppState;
use crate::{ui, pages::E};


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