use iced::{Alignment, Length};
use iced::widget::{container, text, Column, Row, Space};

use crate::app::{AppState, Message, Route};
use crate::{pages::E, ui};

pub fn overview<'a>(_state: &'a AppState, t: ui::Tokens) -> E<'a> {
    // Header row
    let head = Row::new()
        .align_y(Alignment::Center)
        .push(ui::section_title(t, "Overview".to_string(), None))
        .push(container(Space::new()).width(Length::Fill))
        .push(ui::primary_button(
            t,
            "Open universes".to_string(),
            Message::Navigate(Route::UniverseList),
        ));

    // Welcome card
    let welcome = ui::card(
        t,
        Column::new()
            .spacing(6)
            .push(text("Welcome back.").size(18).color(t.foreground))
            .into(),
    );

    // Grids layout
    let grid_top = Row::new()
        .spacing(14)
        .push(ui::recents_card_universe(t))
        .push(ui::recents_card_forge(t))
        .width(Length::Fill);

    let grid_bottom = Row::new()
        .spacing(14)
        .push(recent_activity_list(t))
        .push(container(Space::new()).width(Length::Fill)) // Placeholder for right col
        .width(Length::Fill);

    let body = Column::new()
        .spacing(20)
        .push(head)
        .push(welcome)
        .push(grid_top)
        .push(grid_bottom)
        .width(Length::Fill);

    ui::page_padding(body.into())
}

fn recent_activity_list(t: ui::Tokens) -> iced::Element<'static, Message> {
    let title = "Recent activity";
    let items = vec![
        ("Updated 'The Silver Keep'", "2h ago", "Universe"),
        ("Created 'Iron Sword'", "5h ago", "Forge"),
        ("Modified 'Goblin Camp'", "1d ago", "PM Board"),
    ];

    let header = Row::new()
        .align_y(Alignment::Center)
        .push(text(title).size(14).color(t.foreground))
        .push(container(Space::new()).width(Length::Fill))
        .push(ui::ghost_button(
            t,
            "View all →".to_string(),
            Message::Navigate(Route::Overview),
        ));

    let mut list = Column::new().spacing(10);

    for (name, updated, tag) in items {
        let left = Column::new()
            .spacing(2)
            .push(text(name).size(12).color(t.foreground))
            .push(text(updated).size(10).color(ui::alpha(t.muted_fg, 0.7)));

        let tag_pill = container(text(tag).size(10).color(t.foreground))
            .padding([4, 8])
            .style(move |_| {
                let mut s = ui::container_style(t.active_bg, t.foreground);
                s.border.radius = 999.0.into();
                s
            });

        let row_item = Row::new()
            .align_y(Alignment::Center)
            .push(left)
            .push(container(Space::new()).width(Length::Fill))
            .push(tag_pill);

        list = list.push(row_item);
    }

    ui::card(t, Column::new().spacing(14).push(header).push(list).into())
}