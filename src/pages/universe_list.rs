use iced::{Alignment, Length};
use iced::widget::{container, text, text_input, Column, Row};

use crate::app::{AppState, Message, Route, UniverseMessage};
use crate::model::Universe;
use crate::{ui, pages::E};

pub fn universe_list<'a>(state: &'a AppState, t: ui::Tokens) -> E<'a> {
    let title = Column::new()
        .spacing(4)
        .push(text("Universe").size(26).color(t.foreground))
        .push(text("1 universe in this workspace.").size(12).color(t.muted_fg));

    let name_input = text_input("Universe name", &state.new_universe_name)
        .on_input(|v| Message::Universe(UniverseMessage::NameChanged(v)))
        .padding(10);

    let desc_input = text_input("Short description (optional)", &state.new_universe_desc)
        .on_input(|v| Message::Universe(UniverseMessage::DescChanged(v)))
        .padding(10);

    let create_btn = ui::primary_button(t, "Create universe".to_string(), Message::Universe(UniverseMessage::Create));

    let form = Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(container(name_input).width(Length::Fixed(220.0)))
        .push(container(desc_input).width(Length::Fixed(280.0)))
        .push(create_btn);

    let header = Row::new()
        .push(container(title).width(Length::Fill))
        .push(form)
        .align_y(Alignment::Center);

    let active_header = text("Active universes").size(12).color(t.muted_fg);

    let mut active_list = Column::new().spacing(10);
    for u in state.universes.iter().filter(|u| !u.archived) {
        active_list = active_list.push(universe_card(t, u.clone()));
    }

    let archived = ui::card(
        t,
        container(text("No archived universes.").size(12).color(t.muted_fg))
            .width(Length::Fill)
            .padding([14, 14])
            .into(),
    );

    let body = Column::new()
        .spacing(14)
        .push(header)
        .push(active_header)
        .push(active_list)
        .push(text("Archived").size(12).color(t.muted_fg))
        .push(archived)
        .width(Length::Fill);

    ui::page_padding(body.into())
}

fn universe_card(t: ui::Tokens, u: Universe) -> iced::Element<'static, Message> {
    let left = Column::new()
        .spacing(4)
        .push(text(u.name.clone()).size(16).color(t.foreground))
        .push(text(u.description.clone()).size(12).color(t.muted_fg));

    let actions = Row::new()
        .spacing(10)
        .push(ui::outline_button(t, "Open".to_string(), Message::Universe(UniverseMessage::Open(u.id.clone()))))
        .push(ui::outline_button(t, "Archive".to_string(), Message::Navigate(Route::UniverseList)))
        .push(ui::danger_button(t, "Delete".to_string(), Message::Universe(UniverseMessage::Delete(u.id))));

    let body = Column::new()
        .spacing(10)
        .push(left)
        .push(actions);

    ui::card(t, body.into())
}