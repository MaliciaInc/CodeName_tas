// src/controllers/action_done_controller.rs

use iced::widget::text_editor;

use crate::app::AppState;
use crate::state::{DbAction, DemoResetScope, ToastKind};

fn invalidate_trash(state: &mut AppState) {
    state.trash_entries.clear();
    state.trash_loaded = false;
}

fn clear_forge_ui_state(state: &mut AppState) {
    state.novels.clear();
    state.active_novel_id = None;
    state.active_novel_chapters.clear();
    state.active_chapter_id = None;
    state.active_chapter_scenes.clear();
    state.active_scene_id = None;
    state.forge_content = text_editor::Content::new();
}

fn clear_universe_scoped_caches_if_match(state: &mut AppState, universe_id: &String) {
    // --- Creatures (Bestiary) ---
    if state.loaded_creatures_universe.as_ref() == Some(universe_id) {
        state.loaded_creatures_universe = None;
        state.creatures.clear();
        state.creatures_index.clear(); // ✅ REFACTOR A.3: Clear index too
    }
    // ✅ FASE 10: abrir compuerta completa (loaded_for + in_progress)
    state.core_creatures_loaded_for.remove(universe_id);
    state.core_loading_in_progress.remove(&crate::state::CoreLoadKey::Creatures {
        universe_id: universe_id.clone(),
    });

    // --- Locations ---
    if state.loaded_locations_universe.as_ref() == Some(universe_id) {
        state.loaded_locations_universe = None;
        state.locations.clear();
    }
    state.core_locations_loaded_for.remove(universe_id);
    state.core_loading_in_progress.remove(&crate::state::CoreLoadKey::Locations {
        universe_id: universe_id.clone(),
    });

    // --- Timeline ---
    if state.loaded_timeline_universe.as_ref() == Some(universe_id) {
        state.loaded_timeline_universe = None;
        state.timeline_events.clear();
        state.timeline_eras.clear();
    }
    state.core_timeline_loaded_for.remove(universe_id);
    state.core_loading_in_progress.remove(&crate::state::CoreLoadKey::Timeline {
        universe_id: universe_id.clone(),
    });

    // --- Snapshots ---
    if state.loaded_snapshots_universe.as_ref() == Some(universe_id) {
        state.loaded_snapshots_universe = None;
        state.snapshots.clear();
    }
    state.core_snapshots_loaded_for.remove(universe_id);
    state.core_loading_in_progress.remove(&crate::state::CoreLoadKey::Snapshots {
        universe_id: universe_id.clone(),
    });

    // --- Forge (UI state) ---
    if state.loaded_forge_universe.as_ref() == Some(universe_id) {
        state.loaded_forge_universe = None;
        clear_forge_ui_state(state);
    }
}

fn handle_deleted_universe(state: &mut AppState, deleted_uid: String) {
    // Force refresh of universes list
    state.universes.clear();

    // If we deleted the currently-open UniverseDetail, navigate out.
    if matches!(
        &state.route,
        crate::app::Route::UniverseDetail { universe_id }
            if universe_id == &deleted_uid
    ) {
        state.route = crate::app::Route::UniverseList;
    }

    // Defensive cleanup of scoped caches
    clear_universe_scoped_caches_if_match(state, &deleted_uid);
}

fn handle_deleted_board(state: &mut AppState, deleted_id: String) {
    // Force refresh of boards list
    state.boards_list.clear();

    // If we deleted the currently-open board, navigate out and clear pm_data.
    if matches!(
        &state.route,
        crate::app::Route::PmBoard { board_id }
            if board_id == &deleted_id
    ) {
        state.route = crate::app::Route::PmList;
    }

    if let Some(data) = &state.pm_data {
        if data.board.id == deleted_id {
            state.pm_data = None;
            state.pm_state = crate::app::PmState::Idle;
            state.hovered_column = None;
            state.hovered_card = None;
        }
    }
}

fn invalidate_after_restore_from_trash(state: &mut AppState) {
    // Refresh trash list
    invalidate_trash(state);

    // Invalidar caches para que se recarguen
    state.universes.clear();
    state.boards_list.clear();
    state.novels.clear();

    // --- Core caches (flags + data) ---
    state.loaded_creatures_universe = None;
    state.creatures.clear();
    state.creatures_index.clear(); // ✅ REFACTOR A.3

    state.loaded_locations_universe = None;
    state.locations.clear();

    state.loaded_timeline_universe = None;
    state.timeline_events.clear();
    state.timeline_eras.clear();

    state.loaded_snapshots_universe = None;
    state.snapshots.clear();

    // ✅ FASE 10: abrir compuertas globales (loaded_for + in_progress)
    state.core_creatures_loaded_for.clear();
    state.core_locations_loaded_for.clear();
    state.core_timeline_loaded_for.clear();
    state.core_snapshots_loaded_for.clear();
    state.core_loading_in_progress.clear();

    // --- Forge caches/UI ---
    state.loaded_forge_universe = None;
    state.active_novel_chapters.clear();
    state.active_chapter_scenes.clear();
}

fn apply_global_invalidate_legacy(state: &mut AppState) {
    // Preserve old behavior: if not handled explicitly, do global invalidate.
    state.data_dirty = true;

    // Flags legacy
    state.loaded_creatures_universe = None;
    state.loaded_locations_universe = None;
    state.loaded_timeline_universe = None;
    state.loaded_snapshots_universe = None;

    // ✅ FASE 10: abrir compuertas (loaded_for + in_progress)
    state.core_creatures_loaded_for.clear();
    state.core_locations_loaded_for.clear();
    state.core_timeline_loaded_for.clear();
    state.core_snapshots_loaded_for.clear();
    state.core_loading_in_progress.clear();

    // PM data
    state.pm_data = None;
}
pub fn handle_action_done(state: &mut AppState, result: &Result<(), String>) {
    // O(1) y cero clones: tomamos la acción inflight y dejamos None de una vez.
    let inflight = state.db_inflight.take();

    match result {
        Ok(_) => {
            let mut do_global_invalidate = true;

            if let Some(ref action) = inflight {
                // Si la acción fue MoveToTrash, invalidar cache de trash SIEMPRE
                if matches!(action, DbAction::MoveToTrash { .. }) {
                    invalidate_trash(state);
                }

                match action {
                    // =========================================================
                    // UNIVERSES LIST
                    // =========================================================
                    DbAction::CreateUniverse { .. } => {
                        do_global_invalidate = false;
                        state.universes.clear();
                    }

                    DbAction::MoveToTrash { target_type, target_id, .. } if *target_type == "universe" => {
                        do_global_invalidate = false;
                        handle_deleted_universe(state, target_id.clone());
                    }

                    // =========================================================
                    // PM BOARDS LIST
                    // =========================================================
                    DbAction::CreateBoard { .. } => {
                        do_global_invalidate = false;
                        state.boards_list.clear();
                    }

                    DbAction::MoveToTrash { target_type, target_id, .. } if target_type == "board" => {
                        do_global_invalidate = false;
                        handle_deleted_board(state, target_id.clone());
                    }

                    // =========================================================
                    // THE FORGE (NOVEL/CHAPTER/SCENE) - PRO (cache inteligente)
                    // =========================================================
                    DbAction::CreateNovel(_, _, _) => {
                        do_global_invalidate = false;
                        crate::controllers::forge_data_controller::invalidate_novels_cache(state);
                    }

                    DbAction::CreateChapter(_, novel_id, _) => {
                        do_global_invalidate = false;
                        // Evitar to_string() + refs temporales: aquí sí ocupamos un String propio.
                        crate::controllers::forge_data_controller::invalidate_chapters_cache(state, novel_id);
                    }

                    DbAction::ReorderChapter(_, _) => {
                        do_global_invalidate = false;
                    }

                    DbAction::CreateScene(_, chapter_id, _) => {
                        do_global_invalidate = false;
                        crate::controllers::forge_data_controller::invalidate_scenes_cache(state, chapter_id);
                    }

                    DbAction::ReorderScene(_, _) => {
                        do_global_invalidate = false;
                    }

                    DbAction::MoveToTrash { target_type, target_id, parent_type, parent_id, .. }
                    if target_type == "novel" => {
                        do_global_invalidate = false;

                        crate::controllers::forge_data_controller::invalidate_novels_cache(state);

                        if state.active_novel_id.as_ref() == Some(target_id) {
                            state.active_novel_id = None;
                            state.active_novel_chapters.clear();
                            state.active_chapter_id = None;
                            state.active_chapter_scenes.clear();
                            state.active_scene_id = None;
                            state.forge_content = text_editor::Content::new();
                        }

                        state.expanded_novels.remove(target_id);
                    }

                    DbAction::MoveToTrash { target_type, target_id, parent_type, parent_id, .. }
                    if target_type == "chapter" => {
                        do_global_invalidate = false;

                        if parent_type.as_deref() == Some("novel") {
                            if let Some(pid) = parent_id.as_ref() {
                                crate::controllers::forge_data_controller::invalidate_chapters_cache(state, pid);
                            }
                        }

                        if state.active_chapter_id.as_ref() == Some(target_id) {
                            state.active_chapter_id = None;
                            state.active_chapter_scenes.clear();
                            state.active_scene_id = None;
                            state.forge_content = text_editor::Content::new();
                        }

                        state.expanded_chapters.remove(target_id);
                    }

                    DbAction::MoveToTrash { target_type, target_id, parent_type, parent_id, .. }
                    if target_type == "scene" => {
                        do_global_invalidate = false;

                        if parent_type.as_deref() == Some("chapter") {
                            if let Some(pid) = parent_id.as_ref() {
                                crate::controllers::forge_data_controller::invalidate_scenes_cache(state, pid);
                            }
                        }

                        if state.active_scene_id.as_ref() == Some(target_id) {
                            state.active_scene_id = None;
                            state.forge_content = text_editor::Content::new();
                        }
                    }

                    DbAction::UpdateNovel(novel) => {
                        do_global_invalidate = false;
                        crate::controllers::forge_data_controller::invalidate_novels_cache(state);

                        if state.active_novel_id.as_deref() == Some(&novel.id) {
                            crate::logger::info("✅ Novel rename confirmado por DB");
                        }
                    }

                    DbAction::UpdateChapter(chapter) => {
                        do_global_invalidate = false;
                        crate::controllers::forge_data_controller::invalidate_chapters_cache(state, &chapter.novel_id);
                        crate::logger::info("✅ Chapter rename confirmado por DB");
                    }

                    DbAction::UpdateScene(scene) => {
                        do_global_invalidate = false;
                        crate::controllers::forge_data_controller::invalidate_scenes_cache(state, &scene.chapter_id);
                        crate::logger::info("✅ Scene rename confirmado por DB");
                    }

                    // =========================================================
                    // TRASH OPERATIONS
                    // =========================================================
                    DbAction::RestoreFromTrash(_) => {
                        do_global_invalidate = false;
                        invalidate_after_restore_from_trash(state);
                    }

                    DbAction::PermanentDelete(_) => {
                        do_global_invalidate = false;
                        invalidate_trash(state);
                    }

                    DbAction::EmptyTrash => {
                        invalidate_trash(state);
                        state.show_toast("Trash emptied", ToastKind::Success);
                    }

                    // =========================================================
                    // BESTIARY / LOCATIONS: invalidate caches on successful writes
                    // =========================================================
                    DbAction::SaveCreature(_, universe_id) => {
                        do_global_invalidate = false;

                        state.loaded_creatures_universe = None;
                        state.creatures.clear();
                        state.creatures_index.clear(); // ✅ REFACTOR A.3

                        // Evitar construir CoreLoadKey + clone de universe_id:
                        // removemos por predicado (sin allocs).
                        state.core_creatures_loaded_for.remove(universe_id);
                        state.core_loading_in_progress.retain(|k| {
                            !matches!(
                                k,
                                crate::state::CoreLoadKey::Creatures { universe_id: uid }
                                    if uid == universe_id
                            )
                        });
                    }

                    DbAction::ArchiveCreature(_, _) => {
                        do_global_invalidate = false;

                        state.loaded_creatures_universe = None;
                        state.creatures.clear();
                        state.creatures_index.clear(); // ✅ REFACTOR A.3

                        state.core_creatures_loaded_for.clear();
                        state.core_loading_in_progress.retain(|k| {
                            !matches!(k, crate::state::CoreLoadKey::Creatures { .. })
                        });
                    }

                    DbAction::SaveLocation(l) => {
                        do_global_invalidate = false;

                        state.loaded_locations_universe = None;
                        state.locations.clear();

                        state.core_locations_loaded_for.remove(&l.universe_id);
                        state.core_loading_in_progress.retain(|k| {
                            !matches!(
                                k,
                                crate::state::CoreLoadKey::Locations { universe_id: uid }
                                    if uid == &l.universe_id
                            )
                        });
                    }

                    // --- DEFAULT: keep old behavior ---
                    _ => {}
                }
            }

            if do_global_invalidate {
                apply_global_invalidate_legacy(state);
            }

            // Post-success toasts (ONLY when DB confirmed Ok)
            if let Some(action) = inflight {
                match action {
                    DbAction::ResetDemoDataScoped(_, scope) => {
                        let msg = match scope {
                            DemoResetScope::All => "Demo reset complete: Bestiary(7), Locations(7), Timeline(5 eras/15 events), PM Tools(6 cards)",
                            DemoResetScope::Timeline => "Timeline reset complete: 5 eras / 15 events",
                            DemoResetScope::Locations => "Locations reset complete: 7 locations",
                            DemoResetScope::Bestiary => "Bestiary reset complete: 7 creatures",
                            DemoResetScope::PmTools => "PM Tools reset complete: 6 cards",
                        };
                        state.show_toast(msg, ToastKind::Success);
                    }

                    DbAction::InjectDemoData(_) => {
                        state.show_toast("Demo data injected", ToastKind::Success);
                    }

                    DbAction::MoveToTrash { display_name, .. } => {
                        state.show_toast(format!("'{}' moved to trash", display_name), ToastKind::Success);
                    }

                    DbAction::RestoreFromTrash(_) => {
                        state.show_toast("Item restored from trash", ToastKind::Success);
                    }

                    DbAction::SaveCreature(c, _) => {
                        state.show_toast(format!("Creature '{}' saved", c.name), ToastKind::Success);
                    }

                    DbAction::ArchiveCreature(id, archived) => {
                        // Evitamos clonar name a un String intermedio. Formateamos directo con &str.
                        let name = state
                            .creatures
                            .iter()
                            .find(|x| x.id == *id)
                            .map(|x| x.name.as_str())
                            .unwrap_or("Creature");

                        if archived {
                            state.show_toast(format!("Creature '{}' archived", name), ToastKind::Success);
                        } else {
                            state.show_toast(format!("Creature '{}' restored", name), ToastKind::Success);
                        }
                    }

                    DbAction::SaveLocation(l) => {
                        state.show_toast(format!("Location '{}' saved", l.name), ToastKind::Success);
                    }

                    _ => {}
                }
            }
        }

        Err(e) => {
            let msg = if e.contains("disabled in this project") {
                format!(
                    "❌ Feature disabled: {}",
                    e.replace("Capability '", "")
                        .replace("' is disabled in this project", "")
                )
            } else if e.contains("capability") || e.contains("Capability") {
                format!("⚠️ Permission error: {}", e)
            } else {
                format!("Action failed: {}", e)
            };

            state.show_toast(msg, ToastKind::Error);
        }
    }
}
