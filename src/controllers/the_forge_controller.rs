// ============================================
// CONTROLLER: THE FORGE V2 - FIXED
// ============================================

use std::time::{Duration, Instant};
use iced::{Task, widget::{operation, text_editor, Id}};
use crate::app::{AppState, Message};
use crate::messages::TheForgeMessage;
use crate::state::DbAction;
use crate::state::ConfirmAction;

const AUTOSAVE_DELAY_MS: u64 = 800;

// ============================================
// HELPERS: lookup robusto (árbol manda)
// ============================================

fn find_chapter_anywhere(state: &AppState, chapter_id: &str) -> Option<crate::model::Chapter> {
    // 1) Fast-path: si hay novel activa, probamos ahí primero
    if let Some(novel_id) = state.active_novel_id.as_deref() {
        if let Some(list) = state.chapters_by_novel_id.get(novel_id) {
            if let Some(ch) = list.iter().find(|c| c.id == chapter_id) {
                let mut fixed = ch.clone();

                // ✅ PREMIUM: garantizar parent correcto para commits (rename/delete/etc)
                if fixed.novel_id.trim().is_empty() {
                    fixed.novel_id = novel_id.to_string();
                }

                return Some(fixed);
            }
        }
    }

    // 2) Escaneo del árbol completo (hashmap)
    for (novel_id, list) in state.chapters_by_novel_id.iter() {
        if let Some(ch) = list.iter().find(|c| c.id == chapter_id) {
            let mut fixed = ch.clone();

            // ✅ PREMIUM: el bucket sabe el parent real, el struct puede venir incompleto
            if fixed.novel_id.trim().is_empty() {
                fixed.novel_id = novel_id.clone();
            }

            return Some(fixed);
        }
    }

    // 3) Fallback: lista activa (por si el árbol aún no lo tiene)
    state
        .active_novel_chapters
        .iter()
        .find(|c| c.id == chapter_id)
        .cloned()
}

fn find_scene_anywhere(state: &AppState, scene_id: &str) -> Option<crate::model::Scene> {
    // 1) Fast-path: si hay chapter activo, probamos ahí primero
    if let Some(chapter_id) = state.active_chapter_id.as_deref() {
        if let Some(list) = state.scenes_by_chapter_id.get(chapter_id) {
            if let Some(sc) = list.iter().find(|s| s.id == scene_id) {
                return Some(sc.clone());
            }
        }
    }

    // 2) Escaneo del árbol completo (hashmap)
    for (_chapter_id, list) in state.scenes_by_chapter_id.iter() {
        if let Some(sc) = list.iter().find(|s| s.id == scene_id) {
            return Some(sc.clone());
        }
    }

    // 3) Fallback: lista activa
    state
        .active_chapter_scenes
        .iter()
        .find(|s| s.id == scene_id)
        .cloned()
}

pub fn update(state: &mut AppState, message: TheForgeMessage) -> Option<Task<Message>> {
    crate::logger::info(&format!("🎨 FORGE: {:?}", message));

    match message {
        // --- NAVIGATION ---
        TheForgeMessage::Open(universe_id) => {
            crate::logger::info("   📍 Handling Forge Open...");

            // ✅ PASO 1: Cambiar la ruta
            state.route = crate::app::Route::Forge;
            crate::logger::info("   ✅ Route changed to Forge");

            // ✅ PASO 2: Auto-seleccionar universe si no se especificó
            let target_universe = universe_id.or_else(|| {
                state.loaded_forge_universe.clone().or_else(|| {
                    state.universes.first().map(|u| {
                        crate::logger::info(&format!("   🌌 Auto-selecting universe: {} ({})", u.name, u.id));
                        u.id.clone()
                    })
                })
            });

            crate::logger::info(&format!("   🎯 Target universe: {:?}", target_universe));

            // ✅ PASO 3: Setear loaded_forge_universe (CRÍTICO)
            let should_load = state.loaded_forge_universe != target_universe;

            if should_load {
                crate::logger::info(&format!("   📝 Setting loaded_forge_universe to: {:?}", target_universe));
                state.loaded_forge_universe = target_universe;

                // Limpiar estado anterior
                state.novels.clear();
                state.active_novel_id = None;
                state.active_novel_chapters.clear();
                state.active_chapter_id = None;
                state.active_chapter_scenes.clear();
                state.active_scene_id = None;
                state.forge_content = text_editor::Content::new();

                crate::logger::info("   🧹 Forge state cleared");
            } else {
                crate::logger::info("   ℹ️ Universe already loaded");
            }

            // ✅ VERIFICACIÓN FINAL
            if state.loaded_forge_universe.is_some() {
                crate::logger::info(&format!(
                    "   ✅ SUCCESS: Forge ready with universe: {:?}",
                    state.loaded_forge_universe
                ));

                // ✅ PASO 4 (PRO): disparar carga inicial de Novels (sin DB en state)
                return Some(Task::done(Message::ForgeRequestLoadNovels));
            } else {
                crate::logger::warn("   ⚠️ WARNING: No universe available for Forge!");
            }

            None
        }

        TheForgeMessage::UniverseChanged(new_id) => {
            if state.loaded_forge_universe.as_ref() != Some(&new_id) {
                state.loaded_forge_universe = Some(new_id);
                reset_forge_state(state);
            }
            None
        }
        TheForgeMessage::CreateNovel => {
            // ✅ DEBOUNCING: Prevenir clicks múltiples
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(state.last_create_novel_time).as_millis();

            if elapsed < 1000 {
                // Menos de 1 segundo desde la última creación - ignorar
                crate::logger::warn(&format!("   ⚠️ CreateNovel ignored (debouncing: {}ms)", elapsed));
                return None;
            }

            state.last_create_novel_time = now;
            crate::logger::info("   📝 Creating novel...");

            let is_standalone_novel_project = matches!(
                state.active_project.as_ref().map(|p| p.get_kind()),
                Some(crate::model::ProjectKind::Novel)
            );

            let uid = state.loaded_forge_universe.clone();

            if uid.is_some() || is_standalone_novel_project {
                // ✅ Crear novel localmente PRIMERO
                let new_novel = crate::model::Novel {
                    id: format!("novel-{}", uuid::Uuid::new_v4()),
                    universe_id: uid.clone(),
                    title: "Novel".to_string(),
                    synopsis: String::new(),
                    status: "draft".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                crate::logger::info(&format!("   🎯 Adding novel {} locally", new_novel.id));

                // ✅ Agregar a la lista local INMEDIATAMENTE
                state.novels.push(new_novel.clone());

                // ✅ Expandir y seleccionar automáticamente
                state.expanded_novels.insert(new_novel.id.clone());
                state.active_novel_id = Some(new_novel.id.clone());

                // ✅ Importante: NO colapsar el árbol global (eso se siente “bug”/antinatural).
                // Solo reseteamos el contexto activo del editor, porque el usuario “se movió” al novel nuevo.
                state.active_novel_chapters.clear();

                state.active_chapter_id = None;
                state.active_chapter_scenes.clear();

                state.active_scene_id = None;
                state.forge_content = text_editor::Content::new();
                cancel_debounce(state);

                // ❌ ANTES: state.expanded_chapters.clear();
                // ✅ AHORA: mantenemos expanded_chapters para no “minimizar” escenas/chapters del novel existente.

                // ✅ Reset de rename (evita que el input/estado de rename “sangre” al nuevo árbol)
                state.forge_renaming_novel_id = None;
                state.forge_renaming_chapter_id = None;
                state.forge_renaming_scene_id = None;

                state.forge_renaming_novel_temp = None;
                state.forge_renaming_chapter_temp = None;
                state.forge_renaming_scene_temp = None;

                // ✅ Reset tracking de double-click (evita falsos dobles al crear)
                state.last_forge_novel_click = None;
                state.last_forge_chapter_click = None;
                state.last_forge_scene_click = None;

                // ✅ Enviar a DB en background
                crate::logger::info(&format!("   💾 Syncing novel {} to DB", new_novel.id));
                state.queue(DbAction::CreateNovel(
                    new_novel.id.clone(),
                    uid,
                    "Novel".to_string(),
                ));
            } else {
                crate::logger::error("   ❌ ERROR: Cannot create novel - no universe loaded!");
                state.show_toast("No universe selected", crate::state::ToastKind::Error);
            }

            None
        }

        TheForgeMessage::DeleteNovel(id) => {
            // ✅ PRO: NO mutar state aquí.
            // Solo abrir confirm modal. La mutación real ocurre en Message::ConfirmDelete.
            state.pending_confirm = Some(ConfirmAction::DeleteNovel(id.clone()));
            None
        }
        TheForgeMessage::SelectNovel(id) => {
            let now = Instant::now();
            let is_double = state.last_forge_novel_click.as_ref()
                .map(|(last_id, last_time)| *last_id == id && now.duration_since(*last_time).as_millis() < 500)
                .unwrap_or(false);

            auto_save_before_switch(state);

            let is_current = state.active_novel_id.as_deref() == Some(&id);

            // 🔥 FIX: Doble click debe funcionar SIEMPRE
            if is_double {
                // Guardar copia temporal del novel para rename
                if let Some(novel) = state.novels.iter().find(|n| n.id == id).cloned() {
                    state.forge_renaming_novel_temp = Some(novel);
                }

                // Si NO es el current, primero activarlo
                if !is_current {
                    state.active_novel_id = Some(id.clone());
                    state.active_novel_chapters.clear();
                    state.active_chapter_id = None;
                    state.active_chapter_scenes.clear();
                    state.active_scene_id = None;
                    state.forge_content = text_editor::Content::new();
                    cancel_debounce(state);
                }

                // Entrar a modo rename
                state.forge_renaming_novel_id = Some(id);
                state.forge_renaming_chapter_id = None;
                state.forge_renaming_scene_id = None;
                state.forge_renaming_chapter_temp = None;
                state.forge_renaming_scene_temp = None;
                state.last_forge_novel_click = None;
                return Some(operation::focus::<Message>(Id::new("forge_novel_rename")));
            }

            // Single click - solo cambiar selección si no es el actual
            if !is_current {
                state.active_novel_id = Some(id.clone());
                state.active_novel_chapters.clear();

                state.active_chapter_id = None;
                state.active_chapter_scenes.clear();

                state.active_scene_id = None;
                state.forge_content = text_editor::Content::new();
                cancel_debounce(state);
            }

            // Limpiar estado de rename
            state.forge_renaming_novel_id = None;
            state.forge_renaming_chapter_id = None;
            state.forge_renaming_scene_id = None;
            state.forge_renaming_novel_temp = None;
            state.forge_renaming_chapter_temp = None;
            state.forge_renaming_scene_temp = None;

            // Registrar el click para detectar doble-click
            state.last_forge_novel_click = Some((id, now));

            None
        }

        TheForgeMessage::NovelTitleChanged(new_title) => {
            // Actualizar en la copia temporal
            if let Some(novel) = &mut state.forge_renaming_novel_temp {
                novel.title = new_title.clone();
                novel.updated_at = chrono::Utc::now();
            }

            // También actualizar en la lista global si existe
            if let Some(novel_id) = &state.forge_renaming_novel_id {
                if let Some(novel) = state.novels.iter_mut().find(|n| n.id == *novel_id) {
                    novel.title = new_title;
                    novel.updated_at = chrono::Utc::now();
                }
            }
            None
        }

        // --- CHAPTER ACTIONS ---
        TheForgeMessage::CreateChapter(novel_id) => {
            if novel_id.is_empty() {
                return None;
            }

            // ✅ Posición basada en el árbol (si existe), fallback al active list.
            let pos: i64 = state
                .chapters_by_novel_id
                .get(&novel_id)
                .map(|v| v.len() as i64)
                .unwrap_or(state.active_novel_chapters.len() as i64);

            let chapter_id = format!("chapter-{}", uuid::Uuid::new_v4());
            let title = "Chapter".to_string();
            let now = chrono::Utc::now();

            let new_chapter = crate::model::Chapter {
                id: chapter_id.clone(),
                novel_id: novel_id.clone(),
                title: title.clone(),
                synopsis: String::new(), // ✅ antes era notes, pero tu struct usa synopsis
                status: "draft".to_string(),
                position: pos, // ✅ i64
                created_at: now,
                updated_at: now,
            };

            // ✅ Optimista: agregar local inmediato (contexto activo)
            state.active_novel_chapters.push(new_chapter.clone());

            // ✅ CRÍTICO (premium): también insertar en el árbol (fuente del outline)
            state
                .chapters_by_novel_id
                .entry(novel_id.clone())
                .or_default()
                .push(new_chapter.clone());

            // ✅ Expandir el novel y el chapter recién creado
            state.expanded_novels.insert(novel_id.clone());
            state.expanded_chapters.insert(chapter_id.clone());

            state.active_chapter_id = Some(chapter_id.clone());
            state.active_scene_id = None;
            state.active_chapter_scenes.clear();
            state.forge_content = text_editor::Content::new();
            cancel_debounce(state);

            // ✅ DB async: tu enum espera (chapter_id, novel_id, title)
            state.queue(DbAction::CreateChapter(chapter_id, novel_id, title));
            None
        }

        TheForgeMessage::DeleteChapter(id) => {
            // ✅ PRO: NO mutar state aquí.
            // Solo abrir confirm modal. La mutación real ocurre en Message::ConfirmDelete.
            state.pending_confirm = Some(ConfirmAction::DeleteChapter(id.clone()));
            None
        }
        TheForgeMessage::SelectChapter(chapter_id) => {
            let now = Instant::now();
            let is_double = state.last_forge_chapter_click.as_ref()
                .map(|(last_id, last_time)| *last_id == chapter_id && now.duration_since(*last_time).as_millis() < 500)
                .unwrap_or(false);

            auto_save_before_switch(state);

            let is_current = state.active_chapter_id.as_deref() == Some(&chapter_id);

            // 🔥 FIX: Doble click debe funcionar SIEMPRE, incluso al cambiar de chapter
            if is_double {
                crate::logger::info(&format!("🖱️ DOUBLE-CLICK Chapter: {}", chapter_id));

                // Guardar copia temporal del chapter para rename (robusto: árbol primero)
                if let Some(chapter) = find_chapter_anywhere(state, &chapter_id) {
                    crate::logger::info(&format!("   📋 Chapter encontrado con título: '{}'", chapter.title));
                    state.forge_renaming_chapter_temp = Some(chapter);
                } else {
                    crate::logger::warn(&format!(
                        "   ⚠️ Rename Chapter: no encontré chapter {} ni en árbol ni en listas activas",
                        chapter_id
                    ));
                }

                // Si NO es el current, primero activarlo
                if !is_current {
                    state.active_chapter_id = Some(chapter_id.clone());
                    state.active_chapter_scenes.clear();
                    state.active_scene_id = None;
                    state.forge_content = text_editor::Content::new();
                    cancel_debounce(state);
                }

                // Entrar a modo rename
                state.forge_renaming_chapter_id = Some(chapter_id);
                state.forge_renaming_novel_id = None;
                state.forge_renaming_scene_id = None;
                state.forge_renaming_novel_temp = None;
                state.forge_renaming_scene_temp = None;
                state.last_forge_chapter_click = None;
                return Some(operation::focus::<Message>(Id::new("forge_chapter_rename")));
            }

            // Single click - solo cambiar selección si no es el actual
            if !is_current {
                state.active_chapter_id = Some(chapter_id.clone());
                state.active_chapter_scenes.clear();

                state.active_scene_id = None;
                state.forge_content = text_editor::Content::new();
                cancel_debounce(state);
            }

            // Limpiar estado de rename
            state.forge_renaming_chapter_id = None;
            state.forge_renaming_novel_id = None;
            state.forge_renaming_scene_id = None;
            state.forge_renaming_chapter_temp = None;
            state.forge_renaming_novel_temp = None;
            state.forge_renaming_scene_temp = None;

            // Registrar el click para detectar doble-click
            state.last_forge_chapter_click = Some((chapter_id, now));

            None
        }

        TheForgeMessage::ChapterTitleChanged(new_title) => {
            crate::logger::info(&format!("⌨️ ChapterTitleChanged: '{}'", new_title));

            let now = chrono::Utc::now();

            if let Some(chapter_id) = state.forge_renaming_chapter_id.clone() {
                crate::logger::info(&format!("   📝 Actualizando chapter_id: {}", chapter_id));

                // 1) Actualizar en la copia temporal
                if let Some(chapter) = &mut state.forge_renaming_chapter_temp {
                    chapter.title = new_title.clone();
                    chapter.updated_at = now;
                    crate::logger::info(&format!("   ✅ Temp actualizado: '{}'", chapter.title));
                } else {
                    crate::logger::warn("   ⚠️ forge_renaming_chapter_temp es None!");
                }

                // 2) Actualizar en active_novel_chapters
                if let Some(chapter) = state
                    .active_novel_chapters
                    .iter_mut()
                    .find(|c| c.id == chapter_id)
                {
                    chapter.title = new_title.clone();
                    chapter.updated_at = now;
                    crate::logger::info(&format!("   ✅ active_novel_chapters actualizado: '{}'", chapter.title));
                } else {
                    crate::logger::warn(&format!("   ⚠️ Chapter {} NO encontrado en active_novel_chapters", chapter_id));
                }

                // 3) Actualizar en el árbol (chapters_by_novel_id)
                let mut found_in_tree = false;
                for (_novel_id, chapters) in state.chapters_by_novel_id.iter_mut() {
                    if let Some(chapter) = chapters.iter_mut().find(|c| c.id == chapter_id) {
                        chapter.title = new_title.clone();
                        chapter.updated_at = now;
                        crate::logger::info(&format!("   ✅ Árbol actualizado: '{}'", chapter.title));
                        found_in_tree = true;
                        break;
                    }
                }

                if !found_in_tree {
                    crate::logger::warn(&format!("   ⚠️ Chapter {} NO encontrado en chapters_by_novel_id", chapter_id));
                }

                // 🔍 VERIFICACIÓN: Leer del árbol inmediatamente después
                for (_novel_id, chapters) in state.chapters_by_novel_id.iter() {
                    if let Some(chapter) = chapters.iter().find(|c| c.id == chapter_id) {
                        crate::logger::info(&format!("   🔍 VERIFICACIÓN: Árbol ahora tiene título: '{}'", chapter.title));
                        break;
                    }
                }
            } else {
                crate::logger::warn("   ⚠️ forge_renaming_chapter_id es None!");
            }

            None
        }

        // --- SCENE ACTIONS ---
        TheForgeMessage::CreateScene(chapter_id) => {
            // ✅ DEBOUNCING: Prevenir clicks múltiples
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(state.last_create_scene_time).as_millis();

            if elapsed < 1000 {
                crate::logger::warn(&format!(
                    "   ⚠️ CreateScene ignored (debouncing: {}ms)",
                    elapsed
                ));
                return None;
            }

            state.last_create_scene_time = now;
            crate::logger::info("   📝 Creating scene...");

            // ✅ Si el botón pertenece a otro chapter, hacemos switch + limpiamos panel
            if state.active_chapter_id.as_ref() != Some(&chapter_id) {
                crate::logger::info(&format!("   📍 Switching active chapter to: {}", chapter_id));
                state.active_chapter_id = Some(chapter_id.clone());

                state.active_chapter_scenes.clear();
                state.active_scene_id = None;

                state.forge_content = text_editor::Content::new();
                cancel_debounce(state);
            }

            // ✅ Expandir el chapter automáticamente
            if !state.expanded_chapters.contains(&chapter_id) {
                crate::logger::info(&format!("   📂 Auto-expanding chapter: {}", chapter_id));
                state.expanded_chapters.insert(chapter_id.clone());
            }

            // ✅ Crear scene localmente PRIMERO (optimista)
            let new_scene = crate::model::Scene {
                id: format!("scene-{}", uuid::Uuid::new_v4()),
                chapter_id: chapter_id.clone(),
                title: "Scene".to_string(),
                body: String::new(),
                // OJO: position debe basarse en el árbol si existe, no solo en el panel
                position: state
                    .scenes_by_chapter_id
                    .get(&chapter_id)
                    .map(|v| v.len() as i64)
                    .unwrap_or(0),
                status: "draft".to_string(),
                word_count: 0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            crate::logger::info(&format!(
                "   🎯 Adding scene {} to chapter {} locally",
                new_scene.id, chapter_id
            ));

            // ✅ 1) Actualizar FUENTE del árbol (hashmap)
            {
                let entry = state.scenes_by_chapter_id.entry(chapter_id.clone()).or_default();

                // Evitar duplicados por clicks raros / reintentos
                if !entry.iter().any(|s| s.id == new_scene.id) {
                    entry.push(new_scene.clone());
                }
            }

            // ✅ 2) Actualizar panel activo (si estamos en ese chapter)
            if state.active_chapter_id.as_ref() == Some(&chapter_id) {
                if !state.active_chapter_scenes.iter().any(|s| s.id == new_scene.id) {
                    state.active_chapter_scenes.push(new_scene.clone());
                }
            }

            // ✅ Seleccionarlo como activo
            state.active_scene_id = Some(new_scene.id.clone());
            state.forge_content = text_editor::Content::new();
            cancel_debounce(state);

            // ✅ Sync a DB en background
            crate::logger::info(&format!("   💾 Syncing scene {} to DB", new_scene.id));
            state.queue(DbAction::CreateScene(
                new_scene.id.clone(),
                chapter_id,
                "Scene".to_string(),
            ));

            None
        }

        TheForgeMessage::DeleteScene(id) => {
            // ✅ PRO: NO mutar state aquí.
            // Solo abrir confirm modal. La mutación real ocurre en Message::ConfirmDelete.
            state.pending_confirm = Some(ConfirmAction::DeleteScene(id.clone()));
            None
        }

        TheForgeMessage::SelectScene(id) => {
            let now = Instant::now();
            let is_double = state.last_forge_scene_click.as_ref()
                .map(|(last_id, last_time)| *last_id == id && now.duration_since(*last_time).as_millis() < 500)
                .unwrap_or(false);

            auto_save_before_switch(state);

            let is_current = state.active_scene_id.as_deref() == Some(&id);

            // 🔥 FIX: Doble click debe funcionar SIEMPRE
            if is_double {
                // Guardar copia temporal de la scene para rename (robusto: árbol primero)
                if let Some(scene) = find_scene_anywhere(state, &id) {
                    state.forge_renaming_scene_temp = Some(scene);
                } else {
                    crate::logger::warn(&format!(
                        "   ⚠️ Rename Scene: no encontré scene {} ni en árbol ni en listas activas",
                        id
                    ));
                }

                // Si NO es la current, primero activarla
                if !is_current {
                    state.active_scene_id = Some(id.clone());
                    if let Some(scene) = state.active_chapter_scenes.iter().find(|s| s.id == id) {
                        state.forge_content = text_editor::Content::with_text(&scene.body);
                    }
                    cancel_debounce(state);
                }

                // Entrar a modo rename
                state.forge_renaming_scene_id = Some(id);
                state.forge_renaming_novel_id = None;
                state.forge_renaming_chapter_id = None;
                state.forge_renaming_novel_temp = None;
                state.forge_renaming_chapter_temp = None;
                state.last_forge_scene_click = None;
                return Some(operation::focus::<Message>(Id::new("forge_scene_rename")));
            }

            // Single click - cargar scene si no es la actual
            if !is_current {
                state.active_scene_id = Some(id.clone());
                if let Some(scene) = state.active_chapter_scenes.iter().find(|s| s.id == id) {
                    state.forge_content = text_editor::Content::with_text(&scene.body);
                }
                cancel_debounce(state);
            }

            // Limpiar estado de rename
            state.forge_renaming_scene_id = None;
            state.forge_renaming_novel_id = None;
            state.forge_renaming_chapter_id = None;
            state.forge_renaming_novel_temp = None;
            state.forge_renaming_chapter_temp = None;
            state.forge_renaming_scene_temp = None;

            // Registrar el click para detectar doble-click
            state.last_forge_scene_click = Some((id, now));

            None
        }

        TheForgeMessage::SceneTitleChanged(new_title) => {
            let now = chrono::Utc::now();

            // 1) Actualizar en la copia temporal (buffer de rename)
            if let Some(scene) = &mut state.forge_renaming_scene_temp {
                scene.title = new_title.clone();
                scene.updated_at = now;
            }

            // 2) Actualizar en la lista activa (panel/lista actual)
            if let Some(scene_id) = &state.forge_renaming_scene_id {
                if let Some(scene) = state
                    .active_chapter_scenes
                    .iter_mut()
                    .find(|s| s.id == *scene_id)
                {
                    scene.title = new_title.clone();
                    scene.updated_at = now;
                }

                // 3) ✅ PREMIUM: Actualizar también el ÁRBOL real (hashmap) para que el outline se repinte instantáneo.
                let mut updated_in_tree = false;
                for (_chapter_id, scenes) in state.scenes_by_chapter_id.iter_mut() {
                    if let Some(scene) = scenes.iter_mut().find(|s| s.id == *scene_id) {
                        scene.title = new_title.clone();
                        scene.updated_at = now;
                        updated_in_tree = true;
                        break;
                    }
                }

                if !updated_in_tree {
                    crate::logger::warn(&format!(
                        "   ⚠️ SceneTitleChanged: no encontré scene {} en scenes_by_chapter_id (UI se actualizó en activos, pero árbol no)",
                        scene_id
                    ));
                }
            }

            None
        }

        TheForgeMessage::SceneBodyChanged(action) => {
            state.forge_content.perform(action);

            // Actualizar word count y body en el scene activo
            if let Some(scene_id) = &state.active_scene_id {
                if let Some(scene) = state.active_chapter_scenes.iter_mut().find(|s| s.id == *scene_id) {
                    let text = state.forge_content.text();
                    scene.body = text.clone();
                    scene.word_count = count_words(&text);
                }
            }

            // Trigger debounce
            state.forge_last_edit = Some(Instant::now());
            let task_id = state.forge_debounce_task_id.unwrap_or(0) + 1;
            state.forge_debounce_task_id = Some(task_id);

            return Some(Task::perform(debounce_save(task_id), |id| {
                Message::TheForge(TheForgeMessage::DebounceComplete(id))
            }));
        }

        TheForgeMessage::SaveCurrentScene => {
            if let Some(scene_id) = &state.active_scene_id {
                if let Some(scene) = state.active_chapter_scenes.iter().find(|s| s.id == *scene_id).cloned() {
                    state.queue(DbAction::UpdateScene(scene));
                }
            }
            None
        }

        TheForgeMessage::DebounceComplete(completed_id) => {
            if state.forge_debounce_task_id == Some(completed_id) {
                if let Some(last_edit) = state.forge_last_edit {
                    if Instant::now().duration_since(last_edit).as_millis() >= AUTOSAVE_DELAY_MS as u128 {
                        if let Some(scene_id) = &state.active_scene_id {
                            if let Some(scene) = state.active_chapter_scenes.iter().find(|s| s.id == *scene_id).cloned() {
                                state.queue(DbAction::UpdateScene(scene));
                                state.forge_debounce_task_id = None;
                            }
                        }
                    }
                }
            }
            None
        }

        TheForgeMessage::EndRename => {
            crate::logger::info("   💾 EndRename triggered");

            // --- NOVEL RENAME ---
            if let Some(novel) = state.forge_renaming_novel_temp.take() {
                crate::logger::info(&format!(
                    "   ✅ Updating novel {} (title: '{}')",
                    novel.id, novel.title
                ));

                // 1) Lista global de novels
                if let Some(local_novel) = state.novels.iter_mut().find(|n| n.id == novel.id) {
                    local_novel.title = novel.title.clone();
                    local_novel.updated_at = novel.updated_at;
                    crate::logger::info("   ✅ Updated global novel title");
                } else {
                    crate::logger::warn(&format!(
                        "   ⚠️ Novel {} not found in global list",
                        novel.id
                    ));
                }

                state.queue(DbAction::UpdateNovel(novel));
            }

            // --- CHAPTER RENAME ---
            {
                crate::logger::info("🏁 EndRename CHAPTER iniciado");

                let temp = state.forge_renaming_chapter_temp.take();

                if let Some(temp_chapter) = &temp {
                    crate::logger::info(&format!("   📦 Temp tiene título: '{}'", temp_chapter.title));
                } else {
                    crate::logger::warn("   ⚠️ Temp es None!");
                }

                if let Some(chapter_id) = state.forge_renaming_chapter_id.clone() {
                    crate::logger::info(&format!("   🔑 Chapter ID: {}", chapter_id));

                    if let Some(chapter_from_temp) = temp {
                        crate::logger::info(&format!(
                            "   ✅ Persisting chapter rename {} (title: '{}')",
                            chapter_from_temp.id, chapter_from_temp.title
                        ));

                        // ✅ Actualizar el árbol (chapters_by_novel_id)
                        let mut found_in_tree = false;
                        for (novel_id, chapters) in state.chapters_by_novel_id.iter_mut() {
                            if let Some(local) = chapters.iter_mut().find(|c| c.id == chapter_id) {
                                local.title = chapter_from_temp.title.clone();
                                local.updated_at = chapter_from_temp.updated_at;
                                crate::logger::info(&format!(
                                    "   ✅ Árbol actualizado en EndRename con: '{}' (novel: {})",
                                    local.title, novel_id
                                ));
                                found_in_tree = true;
                                break;
                            }
                        }

                        if !found_in_tree {
                            crate::logger::warn(&format!("   ⚠️ Chapter {} NO encontrado en árbol para actualizar", chapter_id));
                        }

                        // 🔍 VERIFICACIÓN: Leer del árbol inmediatamente después
                        for (_novel_id, chapters) in state.chapters_by_novel_id.iter() {
                            if let Some(local) = chapters.iter().find(|c| c.id == chapter_id) {
                                crate::logger::info(&format!("   🔍 VERIFICACIÓN EndRename: Árbol tiene: '{}'", local.title));
                                break;
                            }
                        }

                        // ✅ Actualizar active_novel_chapters también
                        if let Some(local) = state
                            .active_novel_chapters
                            .iter_mut()
                            .find(|c| c.id == chapter_id)
                        {
                            local.title = chapter_from_temp.title.clone();
                            local.updated_at = chapter_from_temp.updated_at;
                            crate::logger::info(&format!("   ✅ active_novel_chapters actualizado: '{}'", local.title));
                        } else {
                            crate::logger::warn(&format!("   ⚠️ Chapter {} NO encontrado en active_novel_chapters", chapter_id));
                        }

                        state.queue(DbAction::UpdateChapter(chapter_from_temp));
                        crate::logger::info("   💾 DbAction::UpdateChapter encolado");
                    } else {
                        crate::logger::warn(&format!(
                            "   ⚠️ EndRename CHAPTER: temp was None for chapter_id {}",
                            chapter_id
                        ));
                    }
                } else {
                    crate::logger::warn("   ⚠️ forge_renaming_chapter_id es None!");
                }

                // ✅ FIX: Forzar re-render del outline (Iced no detecta cambios dentro de HashMaps)
                state.forge_outline_version = state.forge_outline_version.wrapping_add(1);
                crate::logger::info(&format!("   🔄 forge_outline_version: {}", state.forge_outline_version));
            }
            // --- SCENE RENAME ---
            if let Some(scene) = state.forge_renaming_scene_temp.take() {
                crate::logger::info(&format!(
                    "   ✅ Updating scene {} (title: '{}')",
                    scene.id, scene.title
                ));

                // 1) Árbol (hashmap)
                if let Some(list) = state.scenes_by_chapter_id.get_mut(&scene.chapter_id) {
                    if let Some(local) = list.iter_mut().find(|s| s.id == scene.id) {
                        local.title = scene.title.clone();
                        local.updated_at = scene.updated_at;
                        crate::logger::info("   ✅ Updated scene title in scenes_by_chapter_id");
                    } else {
                        crate::logger::warn(&format!(
                            "   ⚠️ Scene {} not found inside scenes_by_chapter_id[{}]",
                            scene.id, scene.chapter_id
                        ));
                    }
                } else {
                    crate::logger::warn(&format!(
                        "   ⚠️ scenes_by_chapter_id has no entry for chapter {} (scene {})",
                        scene.chapter_id, scene.id
                    ));
                }

                // 2) Panel activo (si aplica)
                if state.active_chapter_id.as_deref() == Some(&scene.chapter_id) {
                    if let Some(local) = state
                        .active_chapter_scenes
                        .iter_mut()
                        .find(|s| s.id == scene.id)
                    {
                        local.title = scene.title.clone();
                        local.updated_at = scene.updated_at;
                        crate::logger::info("   ✅ Updated scene title in active_chapter_scenes");
                    }
                }

                state.queue(DbAction::UpdateScene(scene));
            }

            // Limpiar estado de rename
            state.forge_renaming_novel_id = None;
            state.forge_renaming_chapter_id = None;
            state.forge_renaming_scene_id = None;
            state.last_forge_novel_click = None;
            state.last_forge_chapter_click = None;
            state.last_forge_scene_click = None;

            crate::logger::info("   ✅ EndRename complete");
            None
        }

        TheForgeMessage::ToggleNovel(novel_id) => {
            if state.expanded_novels.contains(&novel_id) {
                // ✅ Colapsar: solo toggle visual
                state.expanded_novels.remove(&novel_id);
                crate::logger::info(&format!("   📁 Collapsed novel: {}", novel_id));
                None
            } else {
                // ✅ Expandir: setear activo
                state.expanded_novels.insert(novel_id.clone());
                state.active_novel_id = Some(novel_id.clone());

                crate::logger::info(&format!("   📂 Expanded novel: {}", novel_id));

                // ✅ NUEVO (FASE 2): pedir carga (sin DB aquí)
                Some(Task::done(Message::ForgeRequestLoadChapters(novel_id)))
            }
        }

        TheForgeMessage::ToggleChapter(chapter_id) => {
            // 🛑 PRO: si estamos en modo rename, no permitimos toggles que limpien listas.
            // Esto evita el bug de “renombré chapter y desaparecieron scenes”.
            let renaming_any = state.forge_renaming_novel_id.is_some()
                || state.forge_renaming_chapter_id.is_some()
                || state.forge_renaming_scene_id.is_some();

            if renaming_any {
                // Cerramos rename de forma segura, pero NO colapsamos ni limpiamos nada.
                return Some(Task::done(Message::TheForge(TheForgeMessage::EndRename)));
            }

            let is_expanded = state.expanded_chapters.contains(&chapter_id);

            if is_expanded {
                // ✅ Collapse
                state.expanded_chapters.remove(&chapter_id);

                if state.active_chapter_id.as_ref() == Some(&chapter_id) {
                    state.active_chapter_id = None;
                    state.active_chapter_scenes.clear();
                    state.active_scene_id = None;
                }

                None
            } else {
                // ✅ Expand
                state.expanded_chapters.insert(chapter_id.clone());
                state.active_chapter_id = Some(chapter_id.clone());
                state.active_scene_id = None;

                // ✅ NUEVO (FASE 2): pedir carga (sin DB aquí)
                Some(Task::done(Message::ForgeRequestLoadScenes(chapter_id)))
            }
        }

        // ✅ NUEVO: Drag & Drop
        TheForgeMessage::ChapterDragged(chapter_id, new_position) => {
            state.queue(DbAction::ReorderChapter(chapter_id, new_position as i64));
            None
        }

        TheForgeMessage::SceneDragged(scene_id, new_position) => {
            state.queue(DbAction::ReorderScene(scene_id, new_position as i64));
            None
        }


    }
}

// --- HELPER FUNCTIONS ---

fn reset_forge_state(state: &mut AppState) {
    state.novels.clear();
    state.active_novel_id = None;
    // ✅ Al crear un novel nuevo: reseteamos SOLO la vista activa / selección,
    // pero NO destruimos caches de otros novels.
    state.active_novel_chapters.clear();
    state.active_chapter_id = None;
    state.active_chapter_scenes.clear();
    state.active_scene_id = None;
    state.expanded_chapters.clear();
    state.expanded_novels.clear();
    state.forge_renaming_novel_id = None;
    state.forge_renaming_chapter_id = None;
    state.forge_renaming_scene_id = None;

    // ✅ NUEVO: Limpiar copias temporales
    state.forge_renaming_novel_temp = None;
    state.forge_renaming_chapter_temp = None;
    state.forge_renaming_scene_temp = None;

    state.forge_content = text_editor::Content::new();

    cancel_debounce(state);
}
fn auto_save_before_switch(state: &mut AppState) {
    if let Some(scene_id) = &state.active_scene_id {
        if let Some(scene) = state.active_chapter_scenes.iter().find(|s| s.id == *scene_id).cloned() {
            state.queue(DbAction::UpdateScene(scene));
            crate::logger::info("Auto-saved before switching");
        }
    }
}

pub(crate) fn cancel_debounce(state: &mut AppState) {
    state.forge_last_edit = None;
    state.forge_debounce_task_id = None;
}

fn count_words(text: &str) -> i64 {
    text.split_whitespace().count() as i64
}

async fn debounce_save(id: u64) -> u64 {
    tokio::time::sleep(Duration::from_millis(AUTOSAVE_DELAY_MS)).await;
    id
}

// ============================================
// MOUSE HANDLING
// ============================================

/// Maneja el evento MouseReleased para The Forge
/// Por ahora es un stub - drag & drop se implementará después
pub fn handle_mouse_released(_state: &mut AppState) {
    // TODO: Implementar drag & drop para chapters y scenes
    // Similar a pm_controller::handle_mouse_released
}
