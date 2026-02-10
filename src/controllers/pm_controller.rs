use iced::widget::text_editor;
use iced::Point;
use std::time::Instant;
use uuid::Uuid;

use crate::app::{AppState, PmMessage, PmState};
use crate::state::ConfirmAction;
use crate::state::{DbAction, ToastKind};

// Helper a nivel de módulo: evita duplicación al abrir el editor de creación.
// Nota de rendimiento: esto NO corre por frame; solo en acciones de UI (click/atajos).
// Helpers a nivel de módulo: evita duplicación al abrir editores (Create / Edit).
// Nota de rendimiento: esto NO corre por frame; solo en acciones de UI (click/atajos).

fn open_create_editor(state: &mut AppState, col_id: &str) {
    state.pm_state = PmState::Editing {
        card_id: None,
        column_id: col_id.to_string(),
        title: String::new(),
        description: text_editor::Content::new(),
        priority: "Medium".to_string(),
    };
}

// Edit cuando YA TENÉS Strings (evita to_string extra; mueve ownership)
fn open_edit_editor_owned(
    state: &mut AppState,
    card_id: String,
    column_id: String,
    title: String,
    description: String,
    priority: String,
) {
    let content = text_editor::Content::with_text(&description);
    state.pm_state = PmState::Editing {
        card_id: Some(card_id),
        column_id,
        title,
        description: content,
        priority,
    };
}
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

            // 🔥 Rebuild del pool de IDs internados para evitar heap churn en view/drag/hover
            if let Some(pm) = &state.pm_data {
                state.pm_ids.rebuild_from_pm(pm);

                // Selección inicial: To Do (por id o por nombre), o la primera columna
                if let Some(col) = pm.columns.iter().find(|c| c.id == "col-todo") {
                    state.hovered_column = Some(state.pm_ids.get(col.id.as_str()));
                } else if let Some(col) = pm
                    .columns
                    .iter()
                    .find(|c| c.name.trim().eq_ignore_ascii_case("to do"))
                {
                    state.hovered_column = Some(state.pm_ids.get(col.id.as_str()));
                } else if let Some(first) = pm.columns.first() {
                    state.hovered_column = Some(state.pm_ids.get(first.id.as_str()));
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
            // card_id ahora es PmId (Arc<str>)
            const DOUBLE_CLICK_MS: u128 = 350;
            let now = Instant::now();

            let is_double = match state.last_pm_click.as_ref() {
                Some((last_id, last_at))
                if last_id.as_ref() == card_id.as_ref()
                    && now.duration_since(*last_at).as_millis() <= DOUBLE_CLICK_MS =>
                    {
                        true
                    }
                _ => false,
            };

            if is_double {
                state.last_pm_click = None;

                // ---------------------------------------------------------
                // 1) Extraemos lo que necesitamos mientras el borrow vive
                // ---------------------------------------------------------
                let payload = state
                    .pm_data
                    .as_ref()
                    .and_then(|data| data.get_card(card_id.as_ref()))
                    .map(|card| {
                        (
                            card.id.clone(),
                            card.column_id.clone(),
                            card.title.clone(),
                            card.description.clone(),
                            card.priority.clone(),
                        )
                    });

                // ---------------------------------------------------------
                // 2) Aquí el borrow inmutable de pm_data YA TERMINÓ
                // ---------------------------------------------------------

                if let Some((cid, col, title, desc, prio)) = payload {
                    open_edit_editor_owned(state, cid, col, title, desc, prio);
                }

                return;
            }

            // Primer click: registramos para posible double click (Arc clone = barato)
            state.last_pm_click = Some((card_id.clone(), now));

            // Iniciar dragging con IDs internados (Arc<str>) para evitar clones de String
            if let Some(data) = state.pm_data.as_ref() {
                if let Some(card) = data.get_card(card_id.as_ref()) {
                    state.pm_state = PmState::Dragging {
                        card_id: state.pm_ids.get(card.id.as_str()),
                        card_title: card.title.clone(),
                        original_col: state.pm_ids.get(card.column_id.as_str()),
                        drag_start: iced::Point::new(0.0, 0.0),
                        current_cursor: iced::Point::new(0.0, 0.0),
                        active: false,
                        last_cursor_emit: Instant::now(),
                    };
                }
            }
        }

        PmMessage::ColumnHovered(cid) => state.hovered_column = Some(cid),
        PmMessage::CardHovered(cid) => state.hovered_card = Some(cid),

        PmMessage::OpenCreate(cid) => {
            open_create_editor(state, cid.as_ref());
        }

        PmMessage::OpenGlobalCreate => {
            // Abrimos editor en una columna “razonable”
            let mut target_col_id = String::new();

            if let Some(data) = &state.pm_data {
                // 0) Prefer hovered_column si es válida
                if let Some(hc) = &state.hovered_column {
                    if data.columns.iter().any(|c| c.id.as_str() == hc.as_ref()) {
                        target_col_id = hc.as_ref().to_string();
                    }
                }

                if target_col_id.is_empty() {
                    // 1) Try To Do by id
                    if let Some(col) = data.columns.iter().find(|c| c.id == "col-todo") {
                        target_col_id = col.id.clone();
                    }
                    // 2) Try To Do by name
                    else if let Some(col) = data
                        .columns
                        .iter()
                        .find(|c| c.name.trim().eq_ignore_ascii_case("to do"))
                    {
                        target_col_id = col.id.clone();
                    }
                    // 3) Fallback to first
                    else if let Some(first) = data.columns.first() {
                        target_col_id = first.id.clone();
                    }
                }
            }

            if !target_col_id.is_empty() {
                open_create_editor(state, target_col_id.as_str());
            } else {
                state.show_toast("No columns available to create task", ToastKind::Error);
            }
        }

        PmMessage::OpenEdit(c) => {
            open_edit_editor_owned(state, c.id, c.column_id, c.title, c.description, c.priority);
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
                    // ✅ OPTIMIZED: Use get_column_cards O(1) method
                    let next_pos = state
                        .pm_data
                        .as_ref()
                        .map(|data| {
                            let cards = data.get_column_cards(column_id);
                            cards.len() as i64
                        })
                        .unwrap_or(0);

                    let _card = crate::model::Card {
                        id: card_id
                            .clone()
                            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
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
            if let PmState::Editing {
                card_id: Some(id), ..
            } = &state.pm_state
            {
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
        // Primera muestra real: fija el origen al cursor para evitar activar el drag “de una”
        // por el valor inicial (0,0).
        if !*active && drag_start.x == 0.0 && drag_start.y == 0.0 {
            *drag_start = p;
            *current_cursor = p;
            return;
        }

        *current_cursor = p;

        // No activar drag hasta pasar un umbral (10px)
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

    // Nota: evitamos returns tempranos para GARANTIZAR reset del drag state.
    let mut should_move = false;

    {
        if let PmState::Dragging {
            card_id,
            original_col,
            active,
            ..
        } = &state.pm_state
        {
            if *active {
                if let Some(target_col) = &state.hovered_column {
                    // Si terminó en la misma columna, no hacemos nada (pero igual reseteamos el estado).
                    if target_col.as_ref() != original_col.as_ref() {
                        should_move = true;

                        if let Some(data) = &state.pm_data {
                            // ✅ HOT-PATH: lookup O(1) por columna
                            let cards = data.get_column_cards(target_col.as_ref());

                            let mut new_pos: i64 = 1000;
                            let mut found_neighbor = false;
                            let mut needs_rebalance = false;

                            if let Some(hover_id) = &state.hovered_card {
                                if let Some(idx) = cards
                                    .iter()
                                    .position(|c| c.id.as_str() == hover_id.as_ref())
                                {
                                    found_neighbor = true;

                                    // Insertamos “antes” del hovered card.
                                    let neighbor_pos = cards[idx].position;

                                    let prev_pos = if idx > 0 { cards[idx - 1].position } else { 0 };
                                    let gap = neighbor_pos - prev_pos;

                                    if gap > 1 {
                                        new_pos = neighbor_pos - 1;
                                    } else {
                                        // No hay espacio: ponemos provisional y pedimos rebalance.
                                        new_pos = neighbor_pos + 1;
                                        needs_rebalance = true;
                                    }
                                }
                            }

                            if !found_neighbor {
                                new_pos = cards.last().map(|c| c.position + 1000).unwrap_or(1000);
                            }

                            // ✅ BORDE DB: aquí sí convertimos PmId -> String (1 vez).
                            actions_to_queue.push(DbAction::MoveCard(
                                card_id.as_ref().to_string(),
                                target_col.as_ref().to_string(),
                                new_pos,
                            ));

                            if needs_rebalance {
                                actions_to_queue.push(DbAction::RebalanceColumn(
                                    target_col.as_ref().to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Encolado fuera del borrow de state
    if should_move {
        for action in actions_to_queue {
            state.queue(action);
        }
    }

    // ✅ SIEMPRE reseteamos el drag state (aunque sea misma columna o no haya hovered_column)
    state.pm_state = PmState::Idle;
}
