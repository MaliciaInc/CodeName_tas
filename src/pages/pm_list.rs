use iced::{Alignment, Length};
use iced::widget::{container, text, text_input, Column, Row};

use crate::app::{AppState, Message, PmMessage};
use crate::model::Board;
use crate::{ui, pages::E};

pub fn pm_list<'a>(state: &'a AppState, t: ui::Tokens) -> E<'a> {
    let title = Column::new()
        .spacing(4)
        .push(text("Project Boards").size(26).color(t.foreground))
        .push(text("Manage your projects and tasks.").size(12).color(t.muted_fg));

    let name_input = text_input("New board name...", &state.new_board_name)
        .on_input(|v| Message::Pm(PmMessage::BoardNameChanged(v)))
        .padding(10)
        .style(ui::input_style(t));

    let create_btn = ui::primary_button(t, "Create Board".to_string(), Message::Pm(PmMessage::CreateBoard));

    let form = Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(container(name_input).width(Length::Fixed(300.0)))
        .push(create_btn);

    let header = Row::new()
        .spacing(20)
        .align_y(Alignment::Center)
        .push(title)
        .push(container(iced::widget::Space::new()).width(Length::Fill))
        .push(form)
        .width(Length::Fill);

    let mut board_list = Column::new().spacing(10);

    if state.boards_list.is_empty() {
        board_list = board_list.push(
            text("No boards found. Create one to get started.")
                .size(14)
                .color(t.muted_fg)
        );
    } else {
        for board in &state.boards_list {
            board_list = board_list.push(board_card(t, board));
        }
    }

    let body = Column::new()
        .spacing(20)
        .push(header)
        .push(ui::h_divider(t))
        .push(board_list)
        .width(Length::Fill);

    ui::page_padding(body.into())
}

fn board_card(t: ui::Tokens, board: &Board) -> iced::Element<'static, Message> {
    let b_name = board.name.clone();
    let b_id = board.id.clone();

    let left = Column::new()
        .spacing(4)
        .push(text(b_name).size(16).color(t.foreground))
        .push(text("Kanban Board").size(12).color(t.muted_fg));

    let actions = Row::new()
        .spacing(10)
        .push(ui::outline_button(t, "Open".to_string(), Message::Pm(PmMessage::OpenBoard(b_id.clone()))))
        .push(ui::danger_button(t, "Delete".to_string(), Message::Pm(PmMessage::DeleteBoard(b_id))));

    let body = Row::new()
        .align_y(Alignment::Center)
        .push(container(left).width(Length::Fill))
        .push(actions);

    ui::card(t, body.into())
}