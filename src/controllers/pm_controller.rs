use iced::widget::text_editor;
use iced::Point;
use std::time::Instant;
use uuid::Uuid;

use crate::app::{AppState, PmMessage, PmState};
use crate::model::Card;
use crate::state::{DbAction, ToastKind};
use crate::state::ConfirmAction;

pub fn update(state: &mut AppState, message: PmMessage) {
    match message {
        PmMessage::BoardNameChanged(name) => state.new_board_name = name,

        PmMessage::CreateBoard => {
            if !state.new_board_name.trim().is_empty() {
                // DbAction::CreateBoard es struct variant en tu repo actual.
                let id = format!("board-{}", Uuid::new_v4());
                let name = state.new_board_name.trim().to_string();

                state.queue(DbAction::CreateBoard { id, name });
                state.new_board_name.clear();
                state.show_toast("Creating board...", ToastKind::Info);
            }
        }

        PmMessage::DeleteBoard(id) => {
            state.pending_confirm = Some(ConfirmAction::DeleteBoard(id));
        }

        PmMessage::OpenBoard(id) => state.route = crate::app::Route::PmBoard { board_id: id },

        PmMessage::BoardLoaded(data) => {
            state.pm_data = Some(data);

            // ✅ Mejora #1: escoger columna default inmediatamente
            // Preferimos "col-todo" (id), luego name "to do", luego first.
            if let Some(pm) = &state.pm_data {
                if let Some((col, _)) = pm.columns.iter().find(|(c, _)| c.id == "col-todo") {
                    state.hovered_column = Some(col.id.clone());
                } else if let Some((col, _)) = pm
                    .columns
                    .iter()
                    .find(|(c, _)| c.name.trim().eq_ignore_ascii_case("to do"))
                {
                    state.hovered_column = Some(col.id.clone());
                } else if let Some((first, _)) = pm.columns.first() {
                    state.hovered_column = Some(first.id.clone());
                } else {
                    state.hovered_column = None;
                }
            } else {
                state.hovered_column = None;
            }

            // Limpieza defensiva
            state.hovered_card = None;
            state.pm_state = PmState::Idle;
        }

        PmMessage::DragStart(card_id) => {
            let now = Instant::now();
            let mut is_double_click = false;

            if let Some((last_id, last_time)) = &state.last_pm_click {
                if *last_id == card_id && now.duration_since(*last_time).as_millis() < 500 {
                    is_double_click = true;
                }
            }
            state.last_pm_click = Some((card_id.clone(), now));

            // Buscar el card real en pm_data (sin clonar Card completo en el mensaje)
            let found = state.pm_data.as_ref().and_then(|data| {
                data.columns
                    .iter()
                    .flat_map(|(_col, cards)| cards.iter())
                    .find(|c| c.id == card_id)
            });

            if let Some(c) = found {
                if is_double_click {
                    state.pm_state = PmState::Editing {
                        card_id: Some(c.id.clone()),
                        column_id: c.column_id.clone(),
                        title: c.title.clone(),
                        description: text_editor::Content::with_text(&c.description),
                        priority: c.priority.clone(),
                    };
                } else {
                    state.pm_state = PmState::Dragging {
                        card_id: c.id.clone(),
                        original_col: c.column_id.clone(),
                        drag_start: Point::new(0.0, 0.0),
                        current_cursor: Point::new(0.0, 0.0),
                        active: false,
                    };
                }
            }
        }

        PmMessage::ColumnHovered(cid) => state.hovered_column = Some(cid),
        PmMessage::CardHovered(cid) => state.hovered_card = Some(cid),

        PmMessage::OpenCreate(cid) => {
            state.pm_state = PmState::Editing {
                card_id: None,
                column_id: cid,
                title: String::new(),
                description: text_editor::Content::new(),
                priority: "Medium".to_string(),
            }
        }

        PmMessage::OpenGlobalCreate => {
            // ✅ Mejora #2: preferir hovered_column si existe y es válida
            let mut target_col_id = String::new();

            if let Some(data) = &state.pm_data {
                // 0) Prefer hovered_column si es válida
                if let Some(hc) = &state.hovered_column {
                    if data.columns.iter().any(|(c, _)| &c.id == hc) {
                        target_col_id = hc.clone();
                    }
                }

                if target_col_id.is_empty() {
                    // 1) Try To Do by id
                    if let Some((col, _)) = data.columns.iter().find(|(c, _)| c.id == "col-todo") {
                        target_col_id = col.id.clone();
                    }
                    // 2) Try To Do by name
                    else if let Some((col, _)) = data
                        .columns
                        .iter()
                        .find(|(c, _)| c.name.trim().eq_ignore_ascii_case("to do"))
                    {
                        target_col_id = col.id.clone();
                    }
                    // 3) Fallback to first
                    else if let Some((first, _)) = data.columns.first() {
                        target_col_id = first.id.clone();
                    }
                }
            }

            if !target_col_id.is_empty() {
                state.pm_state = PmState::Editing {
                    card_id: None,
                    column_id: target_col_id,
                    title: String::new(),
                    description: text_editor::Content::new(),
                    priority: "Medium".to_string(),
                };
            } else {
                state.show_toast("No columns available to create task", ToastKind::Error);
            }
        }

        PmMessage::OpenEdit(c) => {
            state.pm_state = PmState::Editing {
                card_id: Some(c.id),
                column_id: c.column_id,
                title: c.title,
                description: text_editor::Content::with_text(&c.description),
                priority: c.priority,
            }
        }

        PmMessage::TitleChanged(v) => {
            if let PmState::Editing { title, .. } = &mut state.pm_state {
                *title = v;
            }
        }

        PmMessage::DescChanged(action) => {
            if let PmState::Editing { description, .. } = &mut state.pm_state {
                description.perform(action);
            }
        }

        PmMessage::PriorityChanged(v) => {
            if let PmState::Editing { priority, .. } = &mut state.pm_state {
                *priority = v;
            }
        }

        PmMessage::Cancel => state.pm_state = PmState::Idle,

        PmMessage::Save => {
            if let PmState::Editing {
                card_id,
                column_id,
                title,
                description,
                priority,
            } = &state.pm_state
            {
                if !title.trim().is_empty() && !column_id.is_empty() {
                    // Calcular la siguiente posición en la columna
                    let next_pos = state.pm_data
                        .as_ref()
                        .and_then(|data| {
                            data.columns.iter()
                                .find(|(col, _)| &col.id == column_id)
                                .map(|(_, cards)| cards.len() as i64)
                        })
                        .unwrap_or(0);

                    let _card = Card {
                        id: card_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                        column_id: column_id.clone(),
                        title: title.clone(),
                        description: description.text(),
                        position: next_pos,
                        priority: priority.clone(),
                    };

                    state.queue(DbAction::SaveCard(_card));
                }
                state.pm_state = crate::app::PmState::Idle;
        }
         }

        // ✅ En tu repo, el delete desde editor es PmMessage::Delete
        PmMessage::Delete => {
            if let PmState::Editing { card_id: Some(id), .. } = &state.pm_state {
                state.queue(DbAction::DeleteCard(id.clone()));
            }
            state.pm_state = PmState::Idle;
        }
    }
}

// --- PUBLIC HELPERS FOR ROOT CONTROLLER (Global Mouse Events) ---
// messages_controller.rs espera estas funciones exactamente.

pub fn handle_mouse_moved(state: &mut AppState, p: Point) {
    if let PmState::Dragging {
        current_cursor,
        drag_start,
        active,
        ..
    } = &mut state.pm_state
    {
        *current_cursor = p;
        if !*active {
            let dx = p.x - drag_start.x;
            let dy = p.y - drag_start.y;
            if (dx * dx + dy * dy).sqrt() > 10.0 {
                *active = true;
            }
        }
    }
}

pub fn handle_mouse_released(state: &mut AppState) {
    let mut actions_to_queue: Vec<DbAction> = Vec::new();

    {
        if let PmState::Dragging { card_id, original_col, active, .. } = &state.pm_state {
            if *active {
                if let Some(target_col) = &state.hovered_column {
                    // Si terminó en la misma columna, no hacemos nada.
                    if target_col == original_col {
                        return;
                    }
                    if let Some(data) = &state.pm_data {
                        if let Some((_, cards)) =
                            data.columns.iter().find(|(col, _)| col.id == *target_col)
                        {
                            let mut new_pos: i64 = 0;
                            let mut found_neighbor = false;
                            let mut needs_rebalance = false;

                            if let Some(hover_id) = &state.hovered_card {
                                if let Some(idx) = cards.iter().position(|c| c.id == *hover_id) {
                                    let neighbor_pos = cards[idx].position;

                                    if idx > 0 {
                                        let prev_pos = cards[idx - 1].position;
                                        new_pos = (prev_pos + neighbor_pos) / 2;

                                        if (neighbor_pos - prev_pos).abs() < 10 {
                                            needs_rebalance = true;
                                        }
                                    } else {
                                        new_pos = neighbor_pos / 2;
                                        if neighbor_pos < 10 {
                                            needs_rebalance = true;
                                        }
                                    }

                                    found_neighbor = true;
                                }
                            }

                            if !found_neighbor {
                                new_pos = if let Some(last) = cards.last() {
                                    last.position + 1000
                                } else {
                                    1000
                                };
                            }

                            actions_to_queue.push(DbAction::MoveCard(
                                card_id.clone(),
                                target_col.clone(),
                                new_pos,
                            ));

                            if needs_rebalance {
                                actions_to_queue.push(DbAction::RebalanceColumn(target_col.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    for action in actions_to_queue {
        state.queue(action);
    }

    state.pm_state = PmState::Idle;
    state.hovered_column = None;
    state.hovered_card = None;
}