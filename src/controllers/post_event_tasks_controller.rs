// ============================================
// POST EVENT TASKS CONTROLLER (FASE 10 READY)
// ============================================
//
// Qué hace este controller:
// 1) Procesa la cola DB (1 acción a la vez) usando db_controller::task_execute.
// 2) Hace "smart polling" por ruta (lazy fetch) con contrato único:
//    - begin global load (CoreLoadKey + throttle + gating)
//    - begin scoped load (CoreLoadKey + loaded_at + throttle + gating)
// 3) Asegura que datos core (Universes/Boards) estén disponibles cuando aplica.
// 4) Evita loops: NO dispara fetch si hay DB inflight o DB queue pendiente.
//
// NOTA:
// - El “hardening” de liberar gating SIEMPRE (Ok/Err/out-of-order)
//   se resuelve en los handlers *Fetched (messages_controller / ui_controller).
//   Este archivo solo inicia cargas de manera consistente.
// ============================================

use iced::Task;

use crate::app::{AppState, Message};
use crate::controllers::db_controller;
use crate::db::Database;
use crate::project_manager::ProjectManager;

const CORE_THROTTLE_MS: u128 = 800;

// -------------------------
// Helpers (FASE 10)
// -------------------------

fn request_universes_if_needed(
    state: &mut AppState,
    db_base: &Database,
    tasks: &mut Vec<Task<Message>>,
) {
    if state.universes.is_empty() {
        let key = crate::state::CoreLoadKey::UniversesList;

        if let Some(now) = state.core_try_begin_global_load(
            key,
            state.last_universes_reload,
            CORE_THROTTLE_MS,
        ) {
            state.last_universes_reload = now;

            let db = db_base.clone();
            tasks.push(Task::perform(
                async move { db.get_all_universes().await.map_err(|e| e.to_string()) },
                Message::UniversesFetched,
            ));
        }
    }
}

fn request_boards_if_needed(
    state: &mut AppState,
    db_base: &Database,
    tasks: &mut Vec<Task<Message>>,
) {
    if state.boards_list.is_empty() {
        let key = crate::state::CoreLoadKey::BoardsList;

        if let Some(now) = state.core_try_begin_global_load(
            key,
            state.last_boards_reload,
            CORE_THROTTLE_MS,
        ) {
            state.last_boards_reload = now;

            let db = db_base.clone();
            tasks.push(Task::perform(
                async move { db.get_all_boards().await.map_err(|e| e.to_string()) },
                Message::BoardsFetched,
            ));
        }
    }
}

fn request_pm_board_if_needed(
    state: &mut AppState,
    db_base: &Database,
    tasks: &mut Vec<Task<Message>>,
    board_id: &String,
) {
    let need_fetch = state
        .pm_data
        .as_ref()
        .map(|d| d.board.id != *board_id)
        .unwrap_or(true);

    if !need_fetch {
        return;
    }

    let key = crate::state::CoreLoadKey::PmBoard {
        board_id: board_id.clone(),
    };

    let loaded_at = state.pm_board_loaded_for.get(board_id).copied();

    if state.core_try_begin_scoped_load(key, loaded_at, CORE_THROTTLE_MS) {
        let db = db_base.clone();

        // ⚠️ Evitar mover el mismo String 2 veces
        let bid_for_task = board_id.clone();
        let bid_for_msg = board_id.clone();

        tasks.push(Task::perform(
            async move { db.get_kanban_data(bid_for_task).await.map_err(|e| e.to_string()) },
            move |result| Message::PmBoardFetched {
                board_id: bid_for_msg,
                result,
            },
        ));
    }
}

fn request_creatures_if_needed(
    state: &mut AppState,
    db_base: &Database,
    tasks: &mut Vec<Task<Message>>,
    universe_id: &String,
) {
    if state.loaded_creatures_universe.as_ref() == Some(universe_id) {
        return;
    }

    let key = crate::state::CoreLoadKey::Creatures {
        universe_id: universe_id.clone(),
    };

    let loaded_at = state.core_creatures_loaded_for.get(universe_id).copied();

    if state.core_try_begin_scoped_load(key, loaded_at, CORE_THROTTLE_MS) {
        let db = db_base.clone();

        let uid_for_task = universe_id.clone();
        let uid_for_msg = universe_id.clone();

        tasks.push(Task::perform(
            async move { db.get_creatures(uid_for_task).await.map_err(|e| e.to_string()) },
            move |result| Message::CreaturesFetched {
                universe_id: uid_for_msg.clone(),
                result,
            },
        ));
    }
}

fn request_locations_if_needed(
    state: &mut AppState,
    db_base: &Database,
    tasks: &mut Vec<Task<Message>>,
    universe_id: &String,
) {
    if state.loaded_locations_universe.as_ref() == Some(universe_id) {
        return;
    }

    let key = crate::state::CoreLoadKey::Locations {
        universe_id: universe_id.clone(),
    };

    let loaded_at = state.core_locations_loaded_for.get(universe_id).copied();

    if state.core_try_begin_scoped_load(key, loaded_at, CORE_THROTTLE_MS) {
        let db = db_base.clone();

        let uid_for_task = universe_id.clone();
        let uid_for_msg = universe_id.clone();

        tasks.push(Task::perform(
            async move { db.get_locations_flat(uid_for_task).await.map_err(|e| e.to_string()) },
            move |result| Message::LocationsFetched {
                universe_id: uid_for_msg.clone(),
                result,
            },
        ));
    }
}

fn request_timeline_if_needed(
    state: &mut AppState,
    db_base: &Database,
    tasks: &mut Vec<Task<Message>>,
    universe_id: &String,
) {
    if state.loaded_timeline_universe.as_ref() == Some(universe_id) {
        return;
    }

    let key = crate::state::CoreLoadKey::Timeline {
        universe_id: universe_id.clone(),
    };

    let loaded_at = state.core_timeline_loaded_for.get(universe_id).copied();

    if state.core_try_begin_scoped_load(key, loaded_at, CORE_THROTTLE_MS) {
        let db = db_base.clone();

        let uid_for_task = universe_id.clone();
        let uid_for_msg = universe_id.clone();

        tasks.push(Task::perform(
            async move {
                let events = db
                    .get_timeline_events(uid_for_task.clone())
                    .await
                    .map_err(|e| e.to_string())?;

                let eras = db
                    .get_timeline_eras(uid_for_task)
                    .await
                    .map_err(|e| e.to_string())?;

                Ok((events, eras))
            },
            move |result| Message::TimelineFetched {
                universe_id: uid_for_msg.clone(),
                result,
            },
        ));
    }
}

fn request_snapshots_if_needed(
    state: &mut AppState,
    db_base: &Database,
    tasks: &mut Vec<Task<Message>>,
    universe_id: &String,
) {
    if state.loaded_snapshots_universe.as_ref() == Some(universe_id) {
        return;
    }

    let key = crate::state::CoreLoadKey::Snapshots {
        universe_id: universe_id.clone(),
    };

    let loaded_at = state.core_snapshots_loaded_for.get(universe_id).copied();

    if state.core_try_begin_scoped_load(key, loaded_at, CORE_THROTTLE_MS) {
        let db = db_base.clone();

        let uid_for_task = universe_id.clone();
        let uid_for_msg = universe_id.clone();

        tasks.push(Task::perform(
            async move { db.snapshot_list(uid_for_task).await.map_err(|e| e.to_string()) },
            move |result| Message::SnapshotsFetched {
                universe_id: uid_for_msg.clone(),
                result,
            },
        ));
    }
}

// -------------------------
// Entry point
// -------------------------

pub fn post_event_tasks(state: &mut AppState, db: &Option<Database>) -> Vec<Task<Message>> {
    let mut tasks = Vec::new();
    let Some(db_base) = db else { return tasks };

    // Keep projects fresh en contextos tipo launcher
    if state.projects.is_empty() {
        tasks.push(Task::perform(
            async { ProjectManager::load_projects() },
            Message::ProjectsLoaded,
        ));
    }

    // ========================================
    // 1) Procesar DB queue (UNA por tick)
    // ========================================
    // Regla PRO: primero DB, luego fetch.
    // Así evitamos "fetch antes del insert".
    if state.db_inflight.is_none() {
        if let Some(action) = state.db_queue.pop_front() {
            state.db_inflight = Some(action.clone());
            let db = db_base.clone();
            tasks.push(db_controller::task_execute(db, action));
        }
    }

    // ========================================
    // Trash fetch (solo cuando aplica)
    // ========================================
    if state.route == crate::app::Route::Trash
        && !state.trash_loaded
        && state.db_inflight.is_none()
    {
        let db = db_base.clone();
        tasks.push(Task::perform(
            async move { db.get_trash_entries().await.map_err(|e| e.to_string()) },
            Message::TrashFetched,
        ));
    }

    // ========================================
    // 2) Lazy fetch por ruta (FASE 10 contract)
    // ========================================
    // Regla PRO: si hay DB inflight o DB queue pendiente, no dispares fetch.
    if state.db_inflight.is_none() && state.db_queue.is_empty() {
        let route = state.route.clone();

        // Prefetch global mínimo (si aplica)
        // - universes: siempre que esté vacío
        request_universes_if_needed(state, db_base, &mut tasks);

        // - boards: solo si la UX lo necesita (Overview / PM List / PM Board)
        let wants_boards = matches!(
            route,
            crate::app::Route::Overview
                | crate::app::Route::PmList
                | crate::app::Route::PmBoard { .. }
        );
        if wants_boards {
            request_boards_if_needed(state, db_base, &mut tasks);
        }

        match route {
            crate::app::Route::UniverseList => {
                // Nada extra: universes ya se pide arriba si hace falta
            }

            crate::app::Route::Overview => {
                // Nada extra: universes + boards ya se pide arriba
            }

            crate::app::Route::PmList => {
                // Nada extra: boards ya se pide arriba
            }

            crate::app::Route::PmBoard { board_id } => {
                request_pm_board_if_needed(state, db_base, &mut tasks, &board_id);
            }

            crate::app::Route::Bestiary { universe_id } => {
                request_creatures_if_needed(state, db_base, &mut tasks, &universe_id);
                request_locations_if_needed(state, db_base, &mut tasks, &universe_id);
            }

            crate::app::Route::Locations { universe_id } => {
                request_locations_if_needed(state, db_base, &mut tasks, &universe_id);
            }

            crate::app::Route::Timeline { universe_id } => {
                request_timeline_if_needed(state, db_base, &mut tasks, &universe_id);
                // Locations también se usa como “dropdown cache” del editor
                request_locations_if_needed(state, db_base, &mut tasks, &universe_id);
            }

            crate::app::Route::Forge => {
                // NOTE: Forge lazy-loading se maneja en navigation_controller::load_forge_data_if_needed.
                // Mantenerlo fuera de post_event evita duplicados.
            }

            crate::app::Route::UniverseDetail { universe_id } => {
                // schema version check (debug overlay)
                if state.debug_overlay_open && state.debug_schema_version.is_none() {
                    let db = db_base.clone();
                    tasks.push(Task::perform(
                        async move { db.get_schema_version().await.map_err(|e| e.to_string()) },
                        Message::SchemaVersionFetched,
                    ));
                }

                // snapshots list fetching (por universo)
                request_snapshots_if_needed(state, &db_base, &mut tasks, &universe_id);

                // integrity issues fetching (Validate Universe)
                if state.integrity_busy {
                    let db = db_base.clone();
                    tasks.push(Task::perform(
                        async move { db.validate_integrity().await.map_err(|e| e.to_string()) },
                        Message::IntegrityFetched,
                    ));
                }
            }

            _ => {}
        }
    }

    tasks
}
