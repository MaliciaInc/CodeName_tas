use iced::{Color, Element, Length};
use iced::widget::{container, scrollable, text, Column, Row};

use crate::{ui, messages::Message, model::TrashEntry};
use crate::state::AppState;
use iced::widget::text_input;

pub fn trash_page(state: &AppState, t: ui::Tokens) -> Element<'_, Message> {
    let mut content = Column::new()
        .spacing(20)
        .padding(24)
        .width(Length::Fill);

    // Header
    let header = Row::new()
        .spacing(12)
        .push(
            text("Trash")
                .size(24)
                .style(move |_| iced::widget::text::Style { color: Some(t.foreground) })
        )
        .push(
            text(format!("{} items", state.trash_entries.len()))
                .size(14)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) })
        );

    content = content.push(header);

    // Search box
    let search_box: iced::widget::TextInput<'_, Message> = text_input("Search in trash...", &state.trash_search_query)
        .on_input(Message::TrashSearchChanged)
        .width(Length::Fixed(300.0))
        .padding(8);

    content = content.push(search_box);  // ← ESTA LÍNEA FALTABA

    // Buttons (Empty Trash, Clean Old Items)
    if !state.trash_entries.is_empty() {
        let buttons = Row::new()
            .spacing(8)
            .push(
                ui::danger_button(t, "Empty Trash".to_string(), Message::EmptyTrash)
            )
            .push(
                ui::ghost_button(t, "Clean Old Items (14+ days)".to_string(), Message::CleanupOldTrash)
            );

        content = content.push(buttons);
    }

    content = content.push(ui::h_divider(t));

    // Filtrar entries según búsqueda
    let filtered_entries: Vec<&TrashEntry> = state.trash_entries
        .iter()
        .filter(|entry| {
            if state.trash_search_query.is_empty() {
                true
            } else {
                entry.display_name.to_lowercase()
                    .contains(&state.trash_search_query.to_lowercase())
            }
        })
        .collect();

    // Mostrar mensaje apropiado si está vacío
    if filtered_entries.is_empty() {
        let empty_message = if state.trash_search_query.is_empty() {
            "Trash is empty"
        } else {
            "No results found"
        };

        content = content.push(
            text(empty_message)
                .size(14)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) })
        );
    } else {
        // Renderizar FILTERED entries (no todos los entries)
        for entry in filtered_entries {
            content = content.push(trash_entry_row(entry, t));
        }
    }

    scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn trash_entry_row(entry: &TrashEntry, t: ui::Tokens) -> Element<'_, Message> {
    let type_badge = container(
        text(&entry.target_type)
            .size(11)
            .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) })
    )
        .padding([2, 8])
        .style(move |_: &iced::Theme| {
            ui::container_style(ui::alpha(t.accent, 0.1), t.accent)
        });

    let mut info_col = Column::new()
        .spacing(4)
        .push(
            text(&entry.display_name)
                .size(14)
                .style(move |_| iced::widget::text::Style { color: Some(t.foreground) })
        );

    // Add metadata row
    let mut metadata_row = Row::new()
        .spacing(8)
        .push(type_badge);

    // Add display_info if present
    if let Some(info) = &entry.display_info {
        metadata_row = metadata_row.push(
            text(format!(" • {}", info))
                .size(12)
                .style(move |_| iced::widget::text::Style {
                    color: Some(Color::from_rgb(0.6, 0.6, 0.6))
                })
        );
    }

    // Add deleted_at timestamp
    metadata_row = metadata_row.push(
        text(format!(" • {}", entry.deleted_at_formatted()))
            .size(12)
            .style(move |_| iced::widget::text::Style {
                color: Some(Color::from_rgb(0.5, 0.5, 0.5))
            })
    );

    info_col = info_col.push(metadata_row);

    let buttons = Row::new()
        .spacing(8)
        .push(
            ui::primary_button(t, "Restore".to_string(), Message::RestoreFromTrash(entry.id.clone()))
        )
        .push(
            ui::danger_button(t, "Delete Forever".to_string(), Message::PermanentDelete(entry.id.clone()))
        );

    let row_content = Row::new()
        .spacing(16)
        .padding(16)
        .push(info_col)
        .push(buttons)
        .width(Length::Fill);

    container(row_content)
        .width(Length::Fill)
        .style(move |_: &iced::Theme| {
            ui::container_style(ui::alpha(t.shell_b, 0.5), t.foreground)
        })
        .into()
}