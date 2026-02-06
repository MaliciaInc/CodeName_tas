use iced::Task;

use crate::app::{AppState, Message, Route};
use crate::db::Database;
use crate::state::ForgeLoadKey;

const NOVELS_THROTTLE_MS: u128 = 800;
const CHAPTERS_THROTTLE_MS: u128 = 800;
const SCENES_THROTTLE_MS: u128 = 800;

// =========================
// NOVELS
// =========================
pub fn load_novels_if_needed(state: &mut AppState, db: &Database) -> Vec<Task<Message>> {
    let mut tasks: Vec<Task<Message>> = Vec::new();

    // Solo aplica si estamos en Forge
    if !matches!(state.route, Route::Forge) {
        return tasks;
    }

    // Si ya tenemos novels, no hacemos nada
    if !state.novels.is_empty() {
        return tasks;
    }

    // Gating: si ya hay una carga de novels en progreso, no repetimos
    if state.forge_loading_in_progress.contains(&ForgeLoadKey::Novels) {
        return tasks;
    }

    // Throttle temporal (evita loops)
    let now = std::time::Instant::now();
    let elapsed = now.duration_since(state.last_novels_reload).as_millis() as u128;
    if elapsed < NOVELS_THROTTLE_MS {
        return tasks;
    }

    // Requiere universe cargado para forge
    let uid = match state.loaded_forge_universe.clone() {
        Some(uid) => uid,
        None => {
            crate::logger::info("⚠️ Forge: loaded_forge_universe is None, skipping novels load");
            return tasks;
        }
    };

    // Marcar estado
    state.last_novels_reload = now;
    state.forge_loading_in_progress.insert(ForgeLoadKey::Novels);

    crate::logger::info("🔄 ForgeDataController: Loading novels");

    let db = db.clone();
    tasks.push(Task::perform(
        async move { db.get_novels(Some(uid)).await.map_err(|e| e.to_string()) },
        Message::NovelsFetched,
    ));

    tasks
}

pub fn mark_novels_load_finished(state: &mut AppState) {
    state.forge_loading_in_progress.remove(&ForgeLoadKey::Novels);
}

// =========================
// CHAPTERS
// =========================
pub fn load_chapters_if_needed(
    state: &mut AppState,
    db: &Database,
    novel_id: String,
) -> Vec<Task<Message>> {
    let mut tasks: Vec<Task<Message>> = Vec::new();

    if !matches!(state.route, Route::Forge) {
        return tasks;
    }

    // Solo cargamos si ese novel es el activo (evita cargas por clicks viejos)
    if state.active_novel_id.as_ref() != Some(&novel_id) {
        state.debug_push(
            crate::state::DebugEventKind::Warn,
            format!("🫥 skip load_chapters: novel_id={} no es activo (active={:?})", novel_id, state.active_novel_id),
        );
        return tasks;
    }

    let key = ForgeLoadKey::Chapters {
        novel_id: novel_id.clone(),
    };

    // Gating
    if state.forge_loading_in_progress.contains(&key) {
        state.debug_push(
            crate::state::DebugEventKind::Warn,
            format!("⛔ gating load_chapters: {:?} ya en progreso", key),
        );
        return tasks;
    }

    // Throttle por "loaded_for"
    let now = std::time::Instant::now();
    if let Some(last_loaded) = state.forge_chapters_loaded_for.get(&novel_id) {
        let elapsed = now.duration_since(*last_loaded).as_millis() as u128;
        if elapsed < CHAPTERS_THROTTLE_MS {
            state.debug_push(
                crate::state::DebugEventKind::Info,
                format!("⏳ throttle load_chapters(novel_id={}): {}ms", novel_id, elapsed),
            );
            return tasks;
        }
    }

    // Throttle global adicional (evita loops por spam de mensajes)
    let elapsed_global = now.duration_since(state.last_chapters_reload).as_millis() as u128;
    if elapsed_global < CHAPTERS_THROTTLE_MS {
        return tasks;
    }

    state.last_chapters_reload = now;
    state.forge_loading_in_progress.insert(key);

    state.debug_push(
        crate::state::DebugEventKind::Info,
        format!("🚀 begin load_chapters for novel_id={}", novel_id),
    );

    crate::logger::info(&format!(
        "🔄 ForgeDataController: Loading chapters for novel {}",
        novel_id
    ));

    let db = db.clone();
    let nid_for_async = novel_id.clone();
    let nid_for_msg = novel_id.clone();

    tasks.push(Task::perform(
        async move { db.get_chapters(nid_for_async).await.map_err(|e| e.to_string()) },
        move |result| Message::ForgeChaptersFetched {
            novel_id: nid_for_msg.clone(),
            result,
        },
    ));

    tasks
}

pub fn mark_chapters_load_finished(state: &mut AppState, novel_id: String) {
    let key = ForgeLoadKey::Chapters { novel_id: novel_id.clone() };
    state.forge_loading_in_progress.remove(&key);
    state.forge_chapters_loaded_for.insert(novel_id, std::time::Instant::now());
}

// =========================
// SCENES
// =========================
pub fn load_scenes_if_needed(
    state: &mut AppState,
    db: &Database,
    chapter_id: String,
) -> Vec<Task<Message>> {
    let mut tasks: Vec<Task<Message>> = Vec::new();

    if !matches!(state.route, Route::Forge) {
        return tasks;
    }

    // Solo cargamos si ese chapter es el activo
    if state.active_chapter_id.as_ref() != Some(&chapter_id) {
        state.debug_push(
            crate::state::DebugEventKind::Warn,
            format!("🫥 skip load_scenes: chapter_id={} no es activo (active={:?})", chapter_id, state.active_chapter_id),
        );
        return tasks;
    }

    let key = ForgeLoadKey::Scenes {
        chapter_id: chapter_id.clone(),
    };

    // Gating
    if state.forge_loading_in_progress.contains(&key) {
        state.debug_push(
            crate::state::DebugEventKind::Warn,
            format!("⛔ gating load_scenes: {:?} ya en progreso", key),
        );
        return tasks;
    }

    // Throttle por "loaded_for"
    let now = std::time::Instant::now();
    if let Some(last_loaded) = state.forge_scenes_loaded_for.get(&chapter_id) {
        let elapsed = now.duration_since(*last_loaded).as_millis() as u128;
        if elapsed < SCENES_THROTTLE_MS {
            state.debug_push(
                crate::state::DebugEventKind::Info,
                format!("⏳ throttle load_scenes(chapter_id={}): {}ms", chapter_id, elapsed),
            );
            return tasks;
        }
    }

    // Throttle global adicional
    let elapsed_global = now.duration_since(state.last_scenes_reload).as_millis() as u128;
    if elapsed_global < SCENES_THROTTLE_MS {
        return tasks;
    }

    state.last_scenes_reload = now;
    state.forge_loading_in_progress.insert(key);

    state.debug_push(
        crate::state::DebugEventKind::Info,
        format!("🚀 begin load_scenes for chapter_id={}", chapter_id),
    );

    crate::logger::info(&format!(
        "🔄 ForgeDataController: Loading scenes for chapter {}",
        chapter_id
    ));

    let db = db.clone();
    let cid_for_async = chapter_id.clone();
    let cid_for_msg = chapter_id.clone();

    // ✅ PRO: emitimos ForgeScenesFetched con chapter_id para poder ignorar respuestas tardías (out-of-order)
    tasks.push(Task::perform(
        async move { db.get_scenes(cid_for_async).await.map_err(|e| e.to_string()) },
        move |result| Message::ForgeScenesFetched {
            chapter_id: cid_for_msg.clone(),
            result,
        },
    ));
    tasks
}

pub fn mark_scenes_load_finished(state: &mut AppState, chapter_id: String) {
    let key = ForgeLoadKey::Scenes { chapter_id: chapter_id.clone() };
    state.forge_loading_in_progress.remove(&key);
    state.forge_scenes_loaded_for.insert(chapter_id, std::time::Instant::now());
}

// =========================
// INVALIDATION (PRO) - NO BORRAR data local
// =========================

pub fn invalidate_novels_cache(state: &mut AppState) {
    // PRO: no borramos novels locales (evita “desaparecieron mis cosas”)
    // Solo quitamos gating para permitir recargas futuras si aplica.
    state.debug_record_invalidation("forge:novels", "*", "invalidate_novels_cache()");

    state.forge_loading_in_progress.remove(&ForgeLoadKey::Novels);
    state.last_novels_reload = std::time::Instant::now()
        - std::time::Duration::from_millis(NOVELS_THROTTLE_MS as u64 + 1);
}

pub fn invalidate_chapters_cache(state: &mut AppState, novel_id: &str) {
    state.debug_record_invalidation(
        "forge:chapters",
        novel_id,
        "invalidate_chapters_cache(novel_id)",
    );

    state.forge_chapters_loaded_for.remove(novel_id);

    state.forge_loading_in_progress.remove(&ForgeLoadKey::Chapters {
        novel_id: novel_id.to_string(),
    });

    state.last_chapters_reload = std::time::Instant::now()
        - std::time::Duration::from_millis(CHAPTERS_THROTTLE_MS as u64 + 1);
}

pub fn invalidate_scenes_cache(state: &mut AppState, chapter_id: &str) {
    state.debug_record_invalidation(
        "forge:scenes",
        chapter_id,
        "invalidate_scenes_cache(chapter_id)",
    );

    state.forge_scenes_loaded_for.remove(chapter_id);

    state.forge_loading_in_progress.remove(&ForgeLoadKey::Scenes {
        chapter_id: chapter_id.to_string(),
    });

    state.last_scenes_reload = std::time::Instant::now()
        - std::time::Duration::from_millis(SCENES_THROTTLE_MS as u64 + 1);
}

// =========================
// RESET TOTAL (al cambiar universe / salir de Forge)
// =========================
pub fn reset_all_data(state: &mut AppState) {
    crate::logger::info("🧹 ForgeDataController: Resetting all Forge data");

    state.novels.clear();
    state.active_novel_chapters.clear();
    state.active_chapter_scenes.clear();

    state.active_novel_id = None;
    state.active_chapter_id = None;
    state.active_scene_id = None;

    state.expanded_novels.clear();
    state.expanded_chapters.clear();

    state.forge_chapters_loaded_for.clear();
    state.forge_scenes_loaded_for.clear();
    state.forge_loading_in_progress.clear();

    // “Permití recarga inmediata” si el usuario entra de nuevo
    state.last_novels_reload = std::time::Instant::now()
        - std::time::Duration::from_millis(NOVELS_THROTTLE_MS as u64 + 1);
    state.last_chapters_reload = std::time::Instant::now()
        - std::time::Duration::from_millis(CHAPTERS_THROTTLE_MS as u64 + 1);
    state.last_scenes_reload = std::time::Instant::now()
        - std::time::Duration::from_millis(SCENES_THROTTLE_MS as u64 + 1);
}
