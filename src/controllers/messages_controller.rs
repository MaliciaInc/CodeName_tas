use iced::Task;

use crate::app::{AppState, Message};
use crate::controllers::{
    bestiary_controller,
    locations_controller,
    navigation_controller,
    pm_controller,
    the_forge_controller,
    timeline_controller,
    universe_controller,
};

use crate::state::ToastKind;
use crate::state::{ConfirmAction, DbAction};

pub fn update(state: &mut AppState, message: Message) -> Vec<Task<Message>> {
    let mut tasks: Vec<Task<Message>> = Vec::new();
    // ✅ FASE 8+: Navegación global con resultado explícito (Handled / Denied / NotHandled)
    match navigation_controller::try_handle(state, &message) {
        navigation_controller::NavigationResult::Handled => {
            return tasks;
        }

        navigation_controller::NavigationResult::Denied { attempted, reason } => {
            crate::logger::error(&format!(
                "❌ Navegación denegada: {} | attempted={:?} | current={:?}",
                reason, attempted, state.route
            ));

            // UX: feedback visible para evitar "click muerto"
            state.show_toast(
                format!("No se puede navegar: {}. Abrí o creá un workspace primero.", reason),
                ToastKind::Error,
            );

            // Ruta segura: Workspaces (no requiere proyecto activo)
            state.route = crate::app::Route::Workspaces;

            return tasks;
        }

        navigation_controller::NavigationResult::NotHandled => {
            // Continúa al match de módulos
        }
    }

    // Navigation is handled in navigation_controller.
    match message {

        // Delegate module-specific messages (canonical names)
        Message::Pm(msg) => pm_controller::update(state, msg),
        Message::Bestiary(msg) => bestiary_controller::update(state, msg),
        Message::Universe(msg) => universe_controller::update(state, msg),
        Message::Locations(msg) => locations_controller::update(state, msg),
        Message::Timeline(msg) => timeline_controller::update(state, msg),

        Message::TheForge(msg) => {
            if let Some(t) = the_forge_controller::update(state, msg) {
                tasks.push(t);
            }
        }
        // Global Mouse Events (Delegated to controllers that need Drag&Drop)
        Message::MouseMoved(p) => {
            pm_controller::handle_mouse_moved(state, p);
            // (Forge no necesita MouseMoved si usamos hover por mouse_area)
        }

        Message::MouseReleased => {
            pm_controller::handle_mouse_released(state);
            the_forge_controller::handle_mouse_released(state);
        }

        // ==========================================================
        // ✅ FASE 9.3 — Core fetch results: liberar gating + timestamps
        // ==========================================================

        // Universes
        Message::UniversesFetched(result) => {
            state
                .core_loading_in_progress
                .remove(&crate::state::CoreLoadKey::UniversesList);

            match result {
                Ok(v) => {
                    state.universes = v;
                }
                Err(e) => {
                    crate::logger::error(&format!("❌ Fetch Universes failed: {}", e));
                    state.show_toast(format!("Action failed: {}", e), ToastKind::Error);
                }
            }
        }

        // Boards list
        Message::BoardsFetched(result) => {
            state
                .core_loading_in_progress
                .remove(&crate::state::CoreLoadKey::BoardsList);

            match result {
                Ok(v) => {
                    state.boards_list = v;
                }
                Err(e) => {
                    crate::logger::error(&format!("❌ Fetch Boards failed: {}", e));
                    state.show_toast(format!("Action failed: {}", e), ToastKind::Error);
                }
            }
        }

        // PM board
        Message::PmBoardFetched { board_id, result } => {
            // ✅ SIEMPRE liberar gating de ese board_id, pase lo que pase (Ok/Err/out-of-order)
            state.core_loading_in_progress.remove(&crate::state::CoreLoadKey::PmBoard {
                board_id: board_id.clone(),
            });

            // Guard de ruta actual (para evitar aplicar resultados viejos)
            let is_current_route = matches!(
                &state.route,
                crate::app::Route::PmBoard { board_id: current } if current == &board_id
            );

            match result {
                Ok(v) => {
                    // ✅ Mark loaded_for (solo si aplica al board actual)
                    if is_current_route {
                        state.pm_board_loaded_for.insert(board_id.clone(), std::time::Instant::now());
                        state.pm_data = Some(v);
                    } else {
                        // Out-of-order: no aplicamos data, pero NO bloqueamos recargas futuras
                        crate::logger::warn(&format!(
                            "⚠️ Ignorando PmBoardFetched out-of-order: fetched_board_id={} route={:?}",
                            board_id, state.route
                        ));
                    }
                }
                Err(e) => {
                    crate::logger::error(&format!("❌ Fetch PmBoard failed: {}", e));

                    // UX: toast solo si estábamos en esa pantalla (evita spam por cargas viejas)
                    if is_current_route {
                        state.show_toast(format!("Action failed: {}", e), ToastKind::Error);
                    }
                }
            }
        }

        // Creatures
        Message::CreaturesFetched { universe_id, result } => {
            // ✅ SIEMPRE liberar gating de ese universe_id, pase lo que pase (Ok/Err/out-of-order)
            state.core_loading_in_progress.remove(&crate::state::CoreLoadKey::Creatures {
                universe_id: universe_id.clone(),
            });

            // Guard de ruta actual (para evitar aplicar resultados viejos)
            let still_relevant = matches!(
                &state.route,
                crate::app::Route::Bestiary { universe_id: uid } if uid == &universe_id
            );

            match result {
                Ok(v) => {
                    if still_relevant {
                        state.creatures = v;

                        // ✅ REFACTOR A.3: Rebuild index after loading
                        state.rebuild_creatures_index();

                        state.loaded_creatures_universe = Some(universe_id.clone());
                        state
                            .core_creatures_loaded_for
                            .insert(universe_id, std::time::Instant::now());
                    } else {
                        // Out-of-order: no aplicamos data, pero NO bloqueamos recargas futuras
                        crate::logger::warn(&format!(
                            "⏭️ Ignorando CreaturesFetched out-of-order (uid={})",
                            universe_id
                        ));
                    }
                }
                Err(e) => {
                    crate::logger::error(&format!(
                        "❌ Fetch creatures failed (uid={}): {}",
                        universe_id, e
                    ));

                    // UX: toast solo si estábamos en esa pantalla (evita spam por cargas viejas)
                    if still_relevant {
                        state.show_toast(format!("Action failed: {}", e), ToastKind::Error);
                    }
                }
            }
        }

        // Locations
        Message::LocationsFetched { universe_id, result } => {
            // FASE 9: limpiar gating aunque cambies de ruta antes de que responda
            state
                .core_loading_in_progress
                .remove(&crate::state::CoreLoadKey::Locations {
                    universe_id: universe_id.clone(),
                });

            let still_relevant = matches!(
                &state.route,
                crate::app::Route::Locations { universe_id: uid }
                    | crate::app::Route::Bestiary { universe_id: uid }
                    | crate::app::Route::Timeline { universe_id: uid }
                    if uid == &universe_id
            );

            match result {
                Ok(v) => {
                    if still_relevant {
                        // Orden determinístico para dropdowns: por nombre
                        let mut v = v;
                        v.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                        state.locations = v;

                        // ✅ REFACTOR A.2: Rebuild hierarchy cache after loading
                        state.rebuild_locations_cache();

                        state.loaded_locations_universe = Some(universe_id.clone());
                        state
                            .core_locations_loaded_for
                            .insert(universe_id, std::time::Instant::now());
                    } else {
                        crate::logger::warn(&format!(
                            "⏭️ Ignorando LocationsFetched out-of-order (uid={})",
                            universe_id
                        ));
                    }
                }
                Err(e) => {
                    crate::logger::error(&format!("❌ Fetch locations failed (uid={}): {}", universe_id, e));
                    if still_relevant {
                        state.show_toast(format!("Action failed: {}", e), ToastKind::Error);
                    }
                }
            }
        }

        // Timeline
        Message::TimelineFetched { universe_id, result } => {
            // FASE 9/10: limpiar gating aunque cambies de ruta antes de que responda
            state
                .core_loading_in_progress
                .remove(&crate::state::CoreLoadKey::Timeline {
                    universe_id: universe_id.clone(),
                });

            let still_relevant = matches!(
                &state.route,
                crate::app::Route::Timeline { universe_id: uid } if uid == &universe_id
            );

            match result {
                Ok((mut events, mut eras)) => {
                    if still_relevant {
                        // C12: orden determinístico una sola vez al cargar (evita clones+sort por frame en view)
                        eras.sort_by_key(|e| e.start_year);
                        events.sort_by_key(|e| e.year);

                        state.timeline_events = events;
                        state.timeline_eras = eras;

                        state.loaded_timeline_universe = Some(universe_id.clone());
                        state
                            .core_timeline_loaded_for
                            .insert(universe_id, std::time::Instant::now());
                    } else {
                        crate::logger::warn(&format!(
                            "⏭️ Ignorando TimelineFetched out-of-order (uid={})",
                            universe_id
                        ));
                    }
                }
                Err(e) => {
                    crate::logger::error(&format!(
                        "❌ Fetch timeline failed (uid={}): {}",
                        universe_id, e
                    ));
                    if still_relevant {
                        state.show_toast(format!("Action failed: {}", e), ToastKind::Error);
                    }
                }
            }
        }

        // --- THE FORGE RESULTS ---
        Message::NovelsFetched(result) => {
            crate::controllers::navigation_controller::handle_novels_fetched(state, result);
        }

        Message::ChaptersFetched(result) => {
            crate::controllers::navigation_controller::handle_chapters_fetched(state, result);
        }

        Message::ForgeChaptersFetched { novel_id, result } => {
            crate::controllers::navigation_controller::handle_forge_chapters_fetched(state, novel_id, result);
        }

        Message::ForgeScenesFetched { chapter_id, result } => {
            crate::controllers::navigation_controller::handle_forge_scenes_fetched(state, chapter_id, result);
        }

        Message::ScenesFetched => {
            // Unit variant obsoleto - la funcionalidad real está en ForgeScenesFetched
        }

        Message::SnapshotsFetched { universe_id, result } => {
            // ✅ SIEMPRE liberar gating de ese universe_id, pase lo que pase (Ok/Err/out-of-order)
            state.core_loading_in_progress.remove(&crate::state::CoreLoadKey::Snapshots {
                universe_id: universe_id.clone(),
            });
            // Guard de ruta actual (para evitar aplicar resultados viejos)
            let still_relevant = matches!(
                &state.route,
                crate::app::Route::UniverseDetail { universe_id: uid } if uid == &universe_id
            );
            match result {
                Ok(v) => {
                    if still_relevant {
                        state.snapshots = v;
                        state.loaded_snapshots_universe = Some(universe_id.clone());
                        state
                            .core_snapshots_loaded_for
                            .insert(universe_id, std::time::Instant::now());
                    }
                }

                Err(e) => {
                    crate::logger::error(&format!(
                        "❌ SnapshotsFetched failed (uid={}): {}",
                        universe_id, e
                    ));

                    // UX: toast solo si estábamos en esa pantalla (evita spam por cargas viejas)
                    if still_relevant {
                        state.show_toast(format!("Action failed: {}", e), ToastKind::Error);
                    }
                }
            }
        }
        Message::SchemaVersionFetched(Ok(v)) => state.debug_schema_version = Some(v),
        Message::IntegrityFetched(Ok(v)) => {
            state.integrity_issues = v;
            state.integrity_busy = false;
        }
        // Errors (canonical grouping) — NO incluye core fetches (ya se manejan arriba con gating release)
        Message::SchemaVersionFetched(Err(e))
        | Message::IntegrityFetched(Err(e)) => {
            crate::logger::error(&format!("❌ Fetch failed: {}", e));
            state.show_toast(format!("Action failed: {}", e), ToastKind::Error);
        }
        Message::Tick => {
            let now = std::time::Instant::now();
            state.toasts
                .retain(|t| now.duration_since(t.created_at).as_secs() < t.ttl_secs);
        }

        Message::ToastDismiss(id) => state.toasts.retain(|t| t.id != id),
        Message::ConfirmDelete => {
            if let Some(action) = state.pending_confirm.take() {
                match action {
                    ConfirmAction::DeleteUniverse(id) => {
                        if let Some(universe) = state.universes.iter().find(|u| u.id == id) {
                            let payload = serde_json::to_string(universe).unwrap_or_default();
                            state.queue(DbAction::MoveToTrash {
                                target_type: "universe".to_string(),
                                target_id: id.clone(),
                                display_name: universe.name.clone(),
                                display_info: None,
                                parent_type: None,
                                parent_id: None,
                                payload_json: payload,
                            });
                        }
                    }
                    ConfirmAction::DeleteBoard(id) => {
                        if let Some(board) = state.boards_list.iter().find(|b| b.id == id) {
                            let payload = serde_json::to_string(board).unwrap_or_default();
                            state.queue(DbAction::MoveToTrash {
                                target_type: "board".to_string(),
                                target_id: id.clone(),
                                display_name: board.name.clone(),
                                display_info: None,
                                parent_type: None,
                                parent_id: None,
                                payload_json: payload,
                            });
                        }
                    }
                    ConfirmAction::DeleteNovel(id) => {
                        if let Some(novel) = state.novels.iter().find(|n| n.id == id) {
                            let payload = serde_json::to_string(novel).unwrap_or_default();
                            let chapter_count = state.active_novel_chapters.len();
                            let display_info = if chapter_count > 0 {
                                Some(format!("{} chapters", chapter_count))
                            } else {
                                None
                            };

                            // 1) DB: ahora sí se encola porque el novel TODAVÍA existe en state
                            state.queue(DbAction::MoveToTrash {
                                target_type: "novel".to_string(),
                                target_id: id.clone(),
                                display_name: novel.title.clone(),
                                display_info,
                                parent_type: novel.universe_id.as_ref().map(|_| "universe".to_string()),
                                parent_id: novel.universe_id.clone(),
                                payload_json: payload,
                            });

                            // 2) UI: limpieza local (optimista, pero consistente)
                            state.novels.retain(|n| n.id != id);

                            if state.last_forge_novel_click
                                .as_ref()
                                .map(|(last_id, _)| last_id == &id)
                                .unwrap_or(false)
                            {
                                state.last_forge_novel_click = None;
                            }

                            if state.active_novel_id.as_deref() == Some(&id) {
                                state.active_novel_id = None;
                                state.active_novel_chapters.clear();
                                state.active_chapter_id = None;
                                state.active_chapter_scenes.clear();
                                state.active_scene_id = None;
                            }

                            if state.forge_renaming_novel_id.as_deref() == Some(&id) {
                                state.forge_renaming_novel_id = None;
                            }
                        }
                    }
                    ConfirmAction::DeleteChapter(id) => {
                        // 🔥 CLAVE: buscar en el árbol primero (hashmap), no solo en la lista activa
                        let mut found_chapter: Option<crate::model::Chapter> = None;

                        for (_novel_id, chapters) in state.chapters_by_novel_id.iter() {
                            if let Some(ch) = chapters.iter().find(|c| c.id == id) {
                                found_chapter = Some(ch.clone());
                                break;
                            }
                        }

                        // Fallback: si no estaba en el árbol, intentamos con la lista activa
                        if found_chapter.is_none() {
                            if let Some(ch) = state.active_novel_chapters.iter().find(|c| c.id == id) {
                                found_chapter = Some(ch.clone());
                            }
                        }

                        if let Some(chapter) = found_chapter {
                            let payload = serde_json::to_string(&chapter).unwrap_or_default();

                            // ✅ Conteo “real” desde el árbol (si existe), no desde la lista activa
                            let scene_count = state
                                .scenes_by_chapter_id
                                .get(&chapter.id)
                                .map(|v| v.len())
                                .unwrap_or(0);

                            let display_info = if scene_count > 0 {
                                Some(format!("{} scenes", scene_count))
                            } else {
                                None
                            };

                            // 1) DB
                            state.queue(DbAction::MoveToTrash {
                                target_type: "chapter".to_string(),
                                target_id: id.clone(),
                                display_name: chapter.title.clone(),
                                display_info,
                                parent_type: Some("novel".to_string()),
                                parent_id: Some(chapter.novel_id.clone()),
                                payload_json: payload,
                            });

                            // 2) UI cleanup — ✅ árbol (hashmap)
                            if let Some(list) = state.chapters_by_novel_id.get_mut(&chapter.novel_id) {
                                list.retain(|c| c.id != id);
                            }

                            // 3) UI cleanup — ✅ scenes del chapter (evitar zombies / resurrección visual)
                            state.scenes_by_chapter_id.remove(&id);

                            // 4) UI cleanup — ✅ panel activo
                            state.active_novel_chapters.retain(|c| c.id != id);
                            state.active_chapter_scenes.clear();

                            // 5) UI cleanup — ✅ expanded + clicks
                            state.expanded_chapters.remove(&id);

                            if state
                                .last_forge_chapter_click
                                .as_ref()
                                .map(|(last_id, _)| last_id == &id)
                                .unwrap_or(false)
                            {
                                state.last_forge_chapter_click = None;
                            }

                            // 6) Selección: si borraste el activo, limpiamos selección y editor
                            if state.active_chapter_id.as_deref() == Some(&id) {
                                state.active_chapter_id = None;
                                state.active_scene_id = None;
                                state.forge_content = iced::widget::text_editor::Content::new();
                            } else {
                                // Si había scene activa, igual la limpiamos porque ya no hay capítulo coherente en el panel
                                state.active_scene_id = None;
                                state.forge_content = iced::widget::text_editor::Content::new();
                            }

                            // 7) Rename state: cancelar si estaba en curso sobre este capítulo
                            if state.forge_renaming_chapter_id.as_deref() == Some(&id) {
                                state.forge_renaming_chapter_id = None;
                            }

                            if let Some(temp) = &state.forge_renaming_chapter_temp {
                                if temp.id == id {
                                    state.forge_renaming_chapter_temp = None;
                                }
                            }

                            // 8) “Premium feel”: invalidación mínima para que no vuelva data vieja por gating/out-of-order
                            crate::controllers::forge_data_controller::invalidate_chapters_cache(state, &chapter.novel_id);
                            crate::controllers::forge_data_controller::invalidate_scenes_cache(state, &id);
                        } else {
                            crate::logger::warn(&format!(
                                "   ⚠️ DeleteChapter confirm: no encontré chapter {} ni en árbol ni en listas activas",
                                id
                            ));
                        }
                    }
                    ConfirmAction::DeleteScene(id) => {
                        // 🔥 CLAVE: buscar en el árbol primero (hashmap), no solo en la lista activa
                        let mut found_scene: Option<crate::model::Scene> = None;
                        let mut found_chapter_id: Option<String> = None;

                        for (chapter_id, scenes) in state.scenes_by_chapter_id.iter() {
                            if let Some(scene) = scenes.iter().find(|s| s.id == id) {
                                found_scene = Some(scene.clone());
                                found_chapter_id = Some(chapter_id.clone());
                                break;
                            }
                        }

                        // Fallback: si no estaba en el árbol, intentamos con la lista activa
                        if found_scene.is_none() {
                            if let Some(scene) = state.active_chapter_scenes.iter().find(|s| s.id == id) {
                                found_scene = Some(scene.clone());
                                found_chapter_id = Some(scene.chapter_id.clone());
                            }
                        }

                        if let (Some(scene), Some(chapter_id)) = (found_scene, found_chapter_id) {
                            let payload = serde_json::to_string(&scene).unwrap_or_default();

                            // 1) DB
                            state.queue(DbAction::MoveToTrash {
                                target_type: "scene".to_string(),
                                target_id: id.clone(),
                                display_name: scene.title.clone(),
                                display_info: Some(format!("{} words", scene.word_count)),
                                parent_type: Some("chapter".to_string()),
                                parent_id: Some(scene.chapter_id.clone()),
                                payload_json: payload,
                            });

                            // 2) UI cleanup — ✅ árbol (hashmap)
                            if let Some(list) = state.scenes_by_chapter_id.get_mut(&chapter_id) {
                                list.retain(|s| s.id != id);
                            }
                            // Si queda vacío, opcionalmente removemos la key
                            if state
                                .scenes_by_chapter_id
                                .get(&chapter_id)
                                .map(|v| v.is_empty())
                                .unwrap_or(false)
                            {
                                state.scenes_by_chapter_id.remove(&chapter_id);
                            }

                            // 3) UI cleanup — ✅ panel activo (si aplica)
                            state.active_chapter_scenes.retain(|s| s.id != id);

                            if state
                                .last_forge_scene_click
                                .as_ref()
                                .map(|(last_id, _)| last_id == &id)
                                .unwrap_or(false)
                            {
                                state.last_forge_scene_click = None;
                            }

                            if state.active_scene_id.as_deref() == Some(&id) {
                                state.active_scene_id = None;
                            }

                            if state.forge_renaming_scene_id.as_deref() == Some(&id) {
                                state.forge_renaming_scene_id = None;
                            }

                            // Si estabas “viendo” esa scene, limpiamos editor
                            if state.active_scene_id.is_none() {
                                state.forge_content = iced::widget::text_editor::Content::new();
                                crate::controllers::the_forge_controller::cancel_debounce(state);
                            }
                        } else {
                            crate::logger::warn(&format!(
                                "   ⚠️ ConfirmDeleteScene: scene {} no encontrada ni en hashmap ni en lista activa",
                                id
                            ));
                        }
                    }

                    ConfirmAction::DeleteCreature(id) => {
                        if let Some(creature) = state.creatures.iter().find(|c| c.id == id) {
                            let payload = serde_json::to_string(creature).unwrap_or_default();
                            state.queue(DbAction::MoveToTrash {
                                target_type: "creature".to_string(),
                                target_id: id.clone(),
                                display_name: creature.name.clone(),
                                display_info: Some(creature.kind.clone()),
                                parent_type: Some("universe".to_string()),
                                parent_id: state.loaded_creatures_universe.clone(),
                                payload_json: payload,
                            });
                        }
                    }

                    ConfirmAction::DeleteLocation(id) => {
                        if let Some(location) = state.locations.iter().find(|l| l.id == id) {
                            let payload = serde_json::to_string(location).unwrap_or_default();
                            state.queue(DbAction::MoveToTrash {
                                target_type: "location".to_string(),
                                target_id: id.clone(),
                                display_name: location.name.clone(),
                                display_info: Some(location.kind.clone()),
                                parent_type: Some("universe".to_string()),
                                parent_id: Some(location.universe_id.clone()),
                                payload_json: payload,
                            });
                        }
                    }

                    ConfirmAction::DeleteEvent(id) => {
                        if let Some(event) = state.timeline_events.iter().find(|e| e.id == id) {
                            let payload = serde_json::to_string(event).unwrap_or_default();
                            state.queue(DbAction::MoveToTrash {
                                target_type: "event".to_string(),
                                target_id: id.clone(),
                                display_name: event.title.clone(),
                                display_info: Some(event.display_date.clone()),
                                parent_type: Some("universe".to_string()),
                                parent_id: Some(event.universe_id.clone()),
                                payload_json: payload,
                            });
                        }
                    }

                    ConfirmAction::DeleteEra(id) => {
                        if let Some(era) = state.timeline_eras.iter().find(|e| e.id == id) {
                            let payload = serde_json::to_string(era).unwrap_or_default();
                            state.queue(DbAction::MoveToTrash {
                                target_type: "era".to_string(),
                                target_id: id.clone(),
                                display_name: era.name.clone(),
                                display_info: None,
                                parent_type: Some("universe".to_string()),
                                parent_id: Some(era.universe_id.clone()),
                                payload_json: payload,
                            });
                        }
                    }
                }
            }
        }

        Message::TrashFetched(Ok(entries)) => {
            state.trash_entries = entries;
            state.trash_loaded = true;
        }

        Message::TrashFetched(Err(e)) => {
            state.show_toast(format!("Failed to load trash: {}", e), ToastKind::Error);
        }

        Message::RestoreFromTrash(entry_id) => {
            state.queue(DbAction::RestoreFromTrash(entry_id));
        }

        Message::PermanentDelete(entry_id) => {
            state.queue(DbAction::PermanentDelete(entry_id));
        }

        Message::EmptyTrash => {
            state.queue(DbAction::EmptyTrash);
        }

        Message::CancelConfirm => {
            state.pending_confirm = None;
        }

        Message::TrashSearchChanged(query) => {
            state.trash_search_query = query;
        }

        Message::ToggleTrashSelection(id) => {
            if state.trash_selected.contains(&id) {
                state.trash_selected.remove(&id);
            } else {
                state.trash_selected.insert(id);
            }
        }

        Message::SelectAllTrash => {
            state.trash_selected = state.trash_entries.iter()
                .map(|e| e.id.clone())
                .collect();
        }

        Message::DeselectAllTrash => {
            state.trash_selected.clear();
        }

        Message::RestoreSelected => {
            let ids: Vec<String> = state.trash_selected.drain().collect();
            for id in ids {
                state.queue(DbAction::RestoreFromTrash(id));
            }
        }

        Message::DeleteSelectedForever => {
            let ids: Vec<String> = state.trash_selected.drain().collect();
            for id in ids {
                state.queue(DbAction::PermanentDelete(id));
            }
        }

        Message::CleanupOldTrash => {
            state.queue(DbAction::CleanupOldTrash);
        }

        _ => {}
    }

    tasks
}
