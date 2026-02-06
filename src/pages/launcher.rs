use iced::{Alignment, Length};
use iced::widget::{button, container, text, text_input, Column, Row, Space}; // Removed mouse_area
use iced::Theme;

use crate::app::{AppState, Message, WorkspaceMessage, APP_NAME};
use crate::model::Project;
use crate::{pages::E, ui};

pub fn launcher_view<'a>(state: &'a AppState, t: ui::Tokens) -> E<'a> {
    let brand = Column::new().align_x(Alignment::Center).spacing(8)
        .push(text(APP_NAME).size(42).color(t.foreground))
        .push(text("Select a workspace to begin architecture.").size(16).color(t.muted_fg));

    // CREATE NEW SECTION
    let create_card = if state.is_creating_project {
        let input = text_input("Workspace Name...", &state.new_project_name)
            .on_input(|v| Message::Workspace(WorkspaceMessage::NameChanged(v)))
            .padding(12)
            .style(ui::input_style(t))
            .on_submit(Message::Workspace(WorkspaceMessage::CreateConfirm));

        Column::new().spacing(12)
            .push(text("Create New Workspace").size(14).color(t.accent))
            .push(input)
            .push(Row::new().spacing(10)
                .push(ui::primary_button(t, "Create".to_string(), Message::Workspace(WorkspaceMessage::CreateConfirm)))
                .push(ui::ghost_button(t, "Cancel".to_string(), Message::Workspace(WorkspaceMessage::CreateCancel)))
            )
    } else {
        Column::new().push(
            ui::primary_button(t, "+ Create New Workspace".to_string(), Message::Workspace(WorkspaceMessage::CreateStart))
        )
    };

    let create_container = container(create_card).width(Length::Fixed(400.0)).padding(20).style(move |_: &Theme| {
        let mut s = ui::container_style(t.card, t.foreground);
        s.border.color = t.border;
        s.border.width = 1.0;
        s.border.radius = 12.0.into();
        s
    });

    // LIST
    let mut list = Column::new().spacing(12).width(Length::Fixed(400.0));
    if state.projects.is_empty() {
        list = list.push(text("No recent workspaces.").size(14).color(t.muted_fg));
    } else {
        list = list.push(text("Recent Workspaces").size(12).color(t.muted_fg));
        for p in &state.projects {
            list = list.push(project_card_premium(t, p));
        }
    }

    let scroll_grid = iced::widget::scrollable(
        Column::new().spacing(30).align_x(Alignment::Center)
            .push(create_container)
            .push(list)
    ).height(Length::Fill).direction(iced::widget::scrollable::Direction::Vertical(iced::widget::scrollable::Scrollbar::new()));

    let content = Column::new().align_x(Alignment::Center).spacing(60)
        .push(brand)
        .push(scroll_grid);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_: &Theme| ui::shell_style(t))
        .into()
}

fn project_card_premium<'a>(t: ui::Tokens, p: &Project) -> iced::Element<'static, Message> {
    let name = p.name.clone();
    let id = p.id.clone();

    // Contenido limpio + Botón borrar
    let content = Column::new().spacing(8)
        .push(text(name).size(18).color(t.foreground))
        .push(Space::new().height(Length::Fill))
        .push(
            Row::new().push(Space::new().width(Length::Fill))
                .push(
                    button(text("×").size(16).color(t.muted_fg))
                        .padding(4)
                        .style(ui::ghost_button_style(t))
                        .on_press(Message::Workspace(WorkspaceMessage::Delete(id.clone())))
                )
        );

    // El botón es el contenedor principal
    button(container(content).width(Length::Fill).height(Length::Fixed(80.0)))
        .padding(16)
        .style(ui::premium_card_style(t))
        .on_press(Message::Workspace(WorkspaceMessage::Open(id)))
        .width(Length::Fill)
        .into()
}