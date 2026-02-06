use iced::Task;
use crate::app::{AppState, Message, Route};
use crate::db::Database;
use crate::model::{Chapter, Novel, Scene};
use crate::state::ToastKind;

// ✅ NUEVO
use crate::controllers::forge_data_controller;

// --- NAVIGATION ROUTING ---

// Resultado explícito de navegación: evita "silencios" y centraliza UX en el dispatcher.
pub enum NavigationResult {
    NotHandled,
    Handled,
    Denied { attempted: Route, reason: String },
}

pub fn try_handle(state: &mut AppState, message: &Message) -> NavigationResult {
    match message {
        Message::Navigate(route) => {
            crate::logger::info(&format!("🧭 NAVIGATE called: {:?}", route));

            // Bloquear navegación si no hay proyecto activo, pero con resultado explícito.
            if state.active_project.is_none() {
                crate::logger::error("❌ Navigation BLOCKED: No active project!");
                crate::logger::error(&format!("   Current route: {:?}", state.route));
                crate::logger::error(&format!("   Attempted route: {:?}", route));

                return NavigationResult::Denied {
                    attempted: route.clone(),
                    reason: "No hay proyecto activo".to_string(),
                };
            }

            if let Some(project) = state.active_project.as_ref() {
                crate::logger::info(&format!("   Active project: {}", project.name));
            } else {
                // Defensive (debería ser inalcanzable si active_project.is_none() arriba)
                crate::logger::error("❌ Navigation BLOCKED: No active project! (unexpected)");
                crate::logger::error(&format!("   Current route: {:?}", state.route));
                crate::logger::error(&format!("   Attempted route: {:?}", route));

                return NavigationResult::Denied {
                    attempted: route.clone(),
                    reason: "No hay proyecto activo (estado inesperado)".to_string(),
                };
            }

            crate::logger::info(&format!("   From: {:?}", state.route));
            crate::logger::info(&format!("   To: {:?}", route));

            state.route = route.clone();

            crate::logger::info("   ✅ Route changed successfully");
            NavigationResult::Handled
        }

        Message::BackToUniverses => {
            crate::logger::info("🧭 Back to Universes");
            state.route = Route::UniverseList;
            NavigationResult::Handled
        }

        Message::BackToUniverse(id) => {
            crate::logger::info(&format!("🧭 Back to Universe: {}", id));
            state.route = crate::app::Route::UniverseDetail {
                universe_id: id.clone(),
            };
            NavigationResult::Handled
        }

        Message::OpenTimeline(id) => {
            crate::logger::info(&format!("🧭 Open Timeline: {}", id));
            state.route = crate::app::Route::Timeline {
                universe_id: id.clone(),
            };
            NavigationResult::Handled
        }

        Message::GoToLocation(universe_id, location_id) => {
            crate::logger::info(&format!(
                "🧭 Go to Location: {} in {}",
                location_id, universe_id
            ));

            state.route = crate::app::Route::Locations {
                universe_id: universe_id.clone(),
            };
            state.selected_location = Some(location_id.clone());

            // Auto-expand tree to ensure the selected location becomes visible.
            let mut current_search = Some(location_id.clone());
            let mut safeguard = 0;

            while let Some(curr_id) = current_search {
                if safeguard > 50 {
                    break;
                }
                safeguard += 1;

                if let Some(loc) = state.locations.iter().find(|l| l.id == curr_id) {
                    if let Some(parent_id) = &loc.parent_id {
                        state.expanded_locations.insert(parent_id.clone());
                        current_search = Some(parent_id.clone());
                    } else {
                        current_search = None;
                    }
                } else {
                    current_search = None;
                }
            }

            NavigationResult::Handled
        }

        _ => NavigationResult::NotHandled,
    }
}

// --- THE FORGE DATA LOADING ---

pub fn load_forge_data_if_needed(state: &mut AppState, db: &Database) -> Vec<Task<Message>> {
    // ✅ DELEGADO al nuevo forge_data_controller (devuelve Vec<Task>)
    crate::controllers::forge_data_controller::load_novels_if_needed(state, db)
}

// --- FETCH HANDLERS ---

pub fn handle_forge_chapters_fetched(
    state: &mut AppState,
    novel_id: String,
    result: Result<Vec<Chapter>, String>,
) {
    match result {
        Ok(chapters) => {
            crate::controllers::forge_data_controller::mark_chapters_load_finished(state, novel_id.clone());

            crate::logger::info(&format!(
                "✅ navigation_controller: Loaded {} chapters (novel {})",
                chapters.len(),
                novel_id
            ));

            let mut by_id: std::collections::HashMap<String, Chapter> = std::collections::HashMap::new();

            for c in chapters {
                by_id.insert(c.id.clone(), c);
            }

            if let Some(local_list) = state.chapters_by_novel_id.remove(&novel_id) {
                for c in local_list {
                    by_id.insert(c.id.clone(), c);
                }
            }

            let mut merged: Vec<Chapter> = by_id.into_values().collect();
            merged.sort_by(|a, b| a.id.cmp(&b.id));

            let chapter_ids: Vec<String> = merged.iter().map(|c| c.id.clone()).collect();
            state.chapters_by_novel_id.insert(novel_id.clone(), merged.clone());

            if state.active_novel_id.as_ref() == Some(&novel_id) {
                state.active_novel_chapters = merged;

                for id in &chapter_ids {
                    state.expanded_chapters.insert(id.clone());
                }

                if state.active_chapter_id.is_none() {
                    if let Some(first_id) = chapter_ids.first() {
                        state.active_chapter_id = Some(first_id.clone());
                        state.active_chapter_scenes.clear();
                        state.active_scene_id = None;

                        crate::logger::info(&format!(
                            "🧭 navigation_controller: Default active chapter set to {} (auto-expanded)",
                            first_id
                        ));
                    }
                }
            } else {
                state.debug_record_ignored(format!(
                    "ForgeChaptersFetched out-of-order: novel_id={} active_novel_id={:?} (cache only)",
                    novel_id, state.active_novel_id
                ));

                crate::logger::info(&format!(
                    "🧠 navigation_controller: Chapters cached for novel {} (not active anymore)",
                    novel_id
                ));
            }

            // ✅ FASE 12: contrato de navegación (safe fallback)
            ensure_forge_safe_fallback(state);
        }
        Err(e) => {
            crate::controllers::forge_data_controller::mark_chapters_load_finished(state, novel_id.clone());

            crate::logger::error(&format!(
                "❌ navigation_controller: Failed to load chapters for novel {}: {}",
                novel_id, e
            ));

            // ✅ FASE 14 (premium): evitar toast spam si el usuario ya cambió de novel
            if state.active_novel_id.as_ref() == Some(&novel_id) {
                state.show_toast(format!("Failed to load chapters: {}", e), ToastKind::Error);
            } else {
                state.debug_record_ignored(format!(
                    "Chapters load error suppressed (not active): novel_id={} active_novel_id={:?} err={}",
                    novel_id, state.active_novel_id, e
                ));
            }
        }
    }
}

pub fn handle_novels_fetched(state: &mut AppState, result: Result<Vec<Novel>, String>) {
    match result {
        Ok(novels) => {
            // ✅ fin de carga (PRO)
            forge_data_controller::mark_novels_load_finished(state);

            crate::logger::info(&format!(
                "✅ navigation_controller: Loaded {} novels",
                novels.len()
            ));

            // Keep a copy of ids for expansion + default selection
            let novel_ids: Vec<String> = novels.iter().map(|n| n.id.clone()).collect();

            state.novels = novels;

            // ✅ Expand ALL novels by default (better visibility)
            for id in &novel_ids {
                state.expanded_novels.insert(id.clone());
            }

            // ✅ Optional: if nothing is selected, select the first novel by default
            if state.active_novel_id.is_none() {
                if let Some(first_id) = novel_ids.first() {
                    state.active_novel_id = Some(first_id.clone());

                    state.active_novel_chapters.clear();
                    state.active_chapter_id = None;
                    state.active_chapter_scenes.clear();
                    state.active_scene_id = None;

                    crate::logger::info(&format!(
                        "🧭 navigation_controller: Default active novel set to {} (auto-expanded)",
                        first_id
                    ));
                }
            }

            // ✅ FASE 12: contrato de navegación (safe fallback)
            ensure_forge_safe_fallback(state);
        }
        Err(e) => {
            forge_data_controller::mark_novels_load_finished(state);
            crate::logger::error(&format!(
                "❌ navigation_controller: Failed to load novels: {}",
                e
            ));
            state.show_toast(format!("Failed to load novels: {}", e), ToastKind::Error);
        }
    }
}

pub fn handle_chapters_fetched(state: &mut AppState, result: Result<Vec<Chapter>, String>) {
    crate::logger::info("📥 handle_chapters_fetched LLAMADO");

    match result {
        Ok(chapters) => {
            crate::logger::info(&format!("   📦 Recibidos {} chapters de la DB", chapters.len()));

            // ✅ FASE 2/4 (PRO): fin de carga + timestamp de "loaded_for"
            if let Some(novel_id) = state.active_novel_id.clone() {
                crate::controllers::forge_data_controller::mark_chapters_load_finished(state, novel_id);
            }

            crate::logger::info(&format!(
                "✅ navigation_controller: Loaded {} chapters",
                chapters.len()
            ));

            // ✅ FASE 4.2 (PRO): merge "LOCAL MANDA" (no pisar lo local con fetch tardío)
            let mut by_id: std::collections::HashMap<String, Chapter> = std::collections::HashMap::new();

            // 1) DB primero
            crate::logger::info("   🔄 Paso 1: Insertando chapters de DB...");
            for c in chapters {
                crate::logger::info(&format!("      DB: '{}' -> '{}'", c.id, c.title));
                by_id.insert(c.id.clone(), c);
            }

            // 2) Local encima (local gana)
            crate::logger::info("   🔄 Paso 2: Merge con active_novel_chapters...");
            for c in state.active_novel_chapters.drain(..) {
                crate::logger::info(&format!("      active: '{}' -> '{}'", c.id, c.title));
                by_id.insert(c.id.clone(), c);
            }

            let mut merged: Vec<Chapter> = by_id.into_values().collect();

            // Orden determinista para evitar "shuffle" visual
            merged.sort_by(|a, b| a.id.cmp(&b.id));

            let chapter_ids: Vec<String> = merged.iter().map(|c| c.id.clone()).collect();

            crate::logger::info("   ✅ Merge completado, resultado:");
            for c in &merged {
                crate::logger::info(&format!("      result: '{}' -> '{}'", c.id, c.title));
            }

            state.active_novel_chapters = merged.clone();

            // 🔥 FIX: También actualizar el árbol con el merge final
            if let Some(novel_id) = &state.active_novel_id {
                state.chapters_by_novel_id.insert(novel_id.clone(), merged);
                crate::logger::info(&format!(
                    "   ✅ Updated chapters_by_novel_id[{}] with {} chapters",
                    novel_id, chapter_ids.len()
                ));
            }

            // ✅ Expand ALL chapters by default
            for id in &chapter_ids {
                state.expanded_chapters.insert(id.clone());
            }

            // ✅ Optional: default active chapter -> helps scenes lazy-load chain
            if state.active_chapter_id.is_none() {
                if let Some(first_id) = chapter_ids.first() {
                    state.active_chapter_id = Some(first_id.clone());

                    // Force scenes to load for the first chapter (loader fetches only if empty)
                    state.active_chapter_scenes.clear();

                    // Reset downstream selection
                    state.active_scene_id = None;

                    crate::logger::info(&format!(
                        "🧭 navigation_controller: Default active chapter set to {} (auto-expanded)",
                        first_id
                    ));
                }
            }
        }
        Err(e) => {
            // ✅ FASE 4.4 (PRO): no dejar el gating pegado si falla el fetch
            if let Some(novel_id) = state.active_novel_id.clone() {
                crate::controllers::forge_data_controller::mark_chapters_load_finished(state, novel_id);
            }

            crate::logger::error(&format!(
                "❌ navigation_controller: Failed to load chapters: {}",
                e
            ));
            state.show_toast(format!("Failed to load chapters: {}", e), ToastKind::Error);
        }
    }
}

pub fn handle_forge_scenes_fetched(
    state: &mut AppState,
    chapter_id: String,
    result: Result<Vec<Scene>, String>,
) {
    // ✅ Siempre cerramos gating para ESTE chapter_id (aunque llegue out-of-order)
    // y siempre cacheamos por chapter_id para que el árbol nunca se mezcle.
    handle_scenes_fetched(state, chapter_id, result);
}

// =======================================================
// FASE 12 — CONTRATO DE NAVEGACIÓN (SAFE FALLBACK)
// “Nunca más pantallas zombis”
// =======================================================
fn ensure_forge_safe_fallback(state: &mut AppState) {
    if !matches!(state.route, Route::Forge) {
        return;
    }

    // 1) NOVEL: si el activo no existe, caer a uno válido (o None)
    let active_novel_exists = state
        .active_novel_id
        .as_ref()
        .map(|id| state.novels.iter().any(|n| &n.id == id))
        .unwrap_or(false);

    if !active_novel_exists {
        state.active_novel_id = state.novels.first().map(|n| n.id.clone());

        // limpiar downstream duro (evita “ruta zombi” interna)
        state.active_novel_chapters.clear();
        state.active_chapter_id = None;
        state.active_chapter_scenes.clear();
        state.active_scene_id = None;

        state.forge_content = iced::widget::text_editor::Content::new();
        crate::controllers::the_forge_controller::cancel_debounce(state);
    }

    // 2) NOVEL UI: expandir el activo y, si tengo cache, sincronizar vista activa
    if let Some(novel_id) = state.active_novel_id.clone() {
        state.expanded_novels.insert(novel_id.clone());

        // Si la vista activa quedó vacía pero ya tenemos cache por novel, lo usamos
        if state.active_novel_chapters.is_empty() {
            if let Some(cached) = state.chapters_by_novel_id.get(&novel_id) {
                state.active_novel_chapters = cached.clone();
            }
        }

        // 3) CHAPTER: validar que el activo exista bajo el novel activo
        let active_chapter_exists = state
            .active_chapter_id
            .as_ref()
            .map(|cid| state.active_novel_chapters.iter().any(|c| &c.id == cid))
            .unwrap_or(false);

        if !active_chapter_exists {
            state.active_chapter_id = state.active_novel_chapters.first().map(|c| c.id.clone());

            // limpiar scenes/editor (porque ya no hay chapter válido)
            state.active_chapter_scenes.clear();
            state.active_scene_id = None;

            state.forge_content = iced::widget::text_editor::Content::new();
            crate::controllers::the_forge_controller::cancel_debounce(state);
        }
    }

    // 4) CHAPTER UI: expandir el activo y, si tengo cache, sincronizar scenes
    if let Some(chapter_id) = state.active_chapter_id.clone() {
        state.expanded_chapters.insert(chapter_id.clone());

        // Si la vista activa quedó vacía pero ya tenemos cache por chapter, lo usamos
        if state.active_chapter_scenes.is_empty() {
            if let Some(cached) = state.scenes_by_chapter_id.get(&chapter_id) {
                state.active_chapter_scenes = cached.clone();
            }
        }

        // 5) SCENE: validar que la activa exista bajo el chapter activo
        let active_scene_exists = state
            .active_scene_id
            .as_ref()
            .map(|sid| state.active_chapter_scenes.iter().any(|s| &s.id == sid))
            .unwrap_or(false);

        if !active_scene_exists {
            state.active_scene_id = state.active_chapter_scenes.first().map(|s| s.id.clone());

            // Si no hay escena válida, el editor no puede quedar “vivo”
            if state.active_scene_id.is_none() {
                state.forge_content = iced::widget::text_editor::Content::new();
                crate::controllers::the_forge_controller::cancel_debounce(state);
            }
        }
    }
}

pub fn handle_scenes_fetched(
    state: &mut AppState,
    chapter_id: String,
    result: Result<Vec<Scene>, String>,
) {
    match result {
        Ok(scenes) => {
            crate::controllers::forge_data_controller::mark_scenes_load_finished(state, chapter_id.clone());

            crate::logger::info(&format!(
                "✅ navigation_controller: Loaded {} scenes (chapter {})",
                scenes.len(),
                chapter_id
            ));

            let mut by_id: std::collections::HashMap<String, Scene> = std::collections::HashMap::new();

            for s in scenes {
                by_id.insert(s.id.clone(), s);
            }

            if let Some(local_list) = state.scenes_by_chapter_id.remove(&chapter_id) {
                for s in local_list {
                    by_id.insert(s.id.clone(), s);
                }
            }

            let mut merged: Vec<Scene> = by_id.into_values().collect();
            merged.sort_by(|a, b| a.id.cmp(&b.id));

            state.scenes_by_chapter_id.insert(chapter_id.clone(), merged.clone());

            if state.active_chapter_id.as_ref() == Some(&chapter_id) {
                state.active_chapter_scenes = merged;

                if state.active_scene_id.is_none() {
                    if let Some(first) = state.active_chapter_scenes.first() {
                        state.active_scene_id = Some(first.id.clone());
                    }
                }
            }

            // ✅ FASE 12: contrato de navegación (safe fallback)
            ensure_forge_safe_fallback(state);
        }
        Err(e) => {
            crate::controllers::forge_data_controller::mark_scenes_load_finished(state, chapter_id.clone());

            crate::logger::error(&format!(
                "❌ navigation_controller: Failed to load scenes for chapter {}: {}",
                chapter_id, e
            ));

            if state.active_chapter_id.as_ref() == Some(&chapter_id) {
                state.show_toast(format!("Failed to load scenes: {}", e), ToastKind::Error);
            }
        }
    }
}