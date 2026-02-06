use iced::{Alignment, Length};
use iced::widget::{container, mouse_area, text, text_input, Column, Row, Space};
use iced::Theme;

use crate::app::{AppState, Message, WorkspaceMessage};
use crate::model::Project;
use crate::{pages::E, ui};

pub fn workspaces_page<'a>(state: &'a AppState, t: ui::Tokens) -> E<'a> {
    let header = Row::new().align_y(Alignment::Center)
        .push(
            Column::new().spacing(4)
                .push(text("Workspaces").size(26).color(t.foreground))
                .push(text("Switch or manage your worlds.").size(12).color(t.muted_fg))
        )
        .push(Space::new().width(Length::Fill))
        .push(ui::danger_button(t, "Close Current Workspace".to_string(), Message::Workspace(WorkspaceMessage::CloseProject)));

    let create_section = if state.is_creating_project {
        let input = text_input("New Workspace Name...", &state.new_project_name)
            .on_input(|v| Message::Workspace(WorkspaceMessage::NameChanged(v)))
            .padding(10)
            .style(ui::input_style(t));

        Row::new().spacing(10)
            .push(container(input).width(Length::Fixed(250.0)))
            .push(ui::primary_button(t, "Create".to_string(), Message::Workspace(WorkspaceMessage::CreateConfirm)))
            .push(ui::ghost_button(t, "Cancel".to_string(), Message::Workspace(WorkspaceMessage::CreateCancel)))
    } else {
        Row::new().push(ui::primary_button(t, "+ New Workspace".to_string(), Message::Workspace(WorkspaceMessage::CreateStart)))
    };

    let mut grid = Column::new().spacing(16);
    let mut row = Row::new().spacing(16);
    let mut count = 0;

    for project in &state.projects {
        row = row.push(project_card_internal(t, project, &state.active_project));
        count += 1;
        if count >= 3 {
            grid = grid.push(row);
            row = Row::new().spacing(16);
            count = 0;
        }
    }
    if count > 0 {
        grid = grid.push(row);
    }

    let body = Column::new().spacing(24)
        .push(header)
        .push(create_section)
        .push(ui::h_divider(t))
        .push(grid);

    ui::page_padding(body.into())
}

fn project_card_internal<'a>(t: ui::Tokens, p: &Project, active: &Option<Project>) -> iced::Element<'static, Message> {
    let is_active = active.as_ref().map(|ap| ap.id == p.id).unwrap_or(false);
    let name = p.name.clone();
    let id = p.id.clone();

    let status_text = if is_active { "Active Session" } else { "Switch to" };
    let status_color = if is_active { t.accent } else { t.muted_fg };

    // FIX: Pill centrado usando center_x/y
    let status_pill = container(text(status_text).size(11).color(status_color))
        .padding([4, 10])
        .center_y(Length::Fill) // Centrado Vertical
        .center_x(Length::Fill) // Centrado Horizontal
        .height(Length::Fixed(24.0)) // Altura fija para alineación perfecta
        .style(move |_: &Theme| {
            let mut s = ui::container_style(ui::alpha(status_color, 0.1), status_color);
            s.border.radius = 99.0.into();
            s
        });

    let content = Column::new().spacing(8)
        .push(text(name).size(16).color(t.foreground).width(Length::Fill))
        .push(Space::new().height(Length::Fill))
        .push(
            Row::new().align_y(Alignment::Center)
                .push(Space::new().width(Length::Fill))
                .push(status_pill)
        );

    let container = container(content)
        .width(Length::Fixed(300.0))
        .height(Length::Fixed(120.0))
        .padding(16)
        .style(move |_: &Theme| {
            let mut s = ui::container_style(t.card, t.foreground);
            s.border.color = if is_active { t.accent } else { t.border };
            s.border.width = if is_active { 2.0 } else { 1.0 };
            s.border.radius = 12.0.into();
            s
        });

    if is_active {
        container.into()
    } else {
        mouse_area(container)
            .on_press(Message::Workspace(WorkspaceMessage::Open(id)))
            .into()
    }
}