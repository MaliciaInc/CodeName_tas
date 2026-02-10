// src/forge_draft.rs
// Draft Recovery de The Forge: borradores locales por scene_id.
// Objetivo: cero pérdida de texto aunque la app crashee o se cierre mal.

use directories::ProjectDirs;
use std::path::{Path, PathBuf};

fn best_effort_dir() -> PathBuf {
    // Similar a tu logger: AppData/Local (Windows) o fallback a %TEMP%
    if let Some(p) = ProjectDirs::from("com", "TitanArchitects", "TAS") {
        p.data_dir().to_path_buf()
    } else {
        std::env::temp_dir().join("TAS")
    }
}

fn drafts_dir() -> PathBuf {
    best_effort_dir().join("forge_drafts")
}

fn sanitize_scene_id(scene_id: &str) -> String {
    // Evita problemas de filesystem (aunque UUID normalmente es seguro).
    // Permitimos [a-zA-Z0-9-_], lo demás => '_'
    scene_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn draft_path(scene_id: &str) -> PathBuf {
    let file = format!("{}.draft.txt", sanitize_scene_id(scene_id));
    drafts_dir().join(file)
}

pub async fn write_draft(scene_id: &str, body: &str) -> Result<(), String> {
    let path = draft_path(scene_id);

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("No pude crear folder de drafts: {e}"))?;
    }

    // Escritura atómica: escribir a temp y luego rename.
    // En Windows rename sobre existente puede fallar; hacemos remove previa best-effort.
    let tmp = path.with_extension("tmp");

    tokio::fs::write(&tmp, body.as_bytes())
        .await
        .map_err(|e| format!("No pude escribir draft tmp: {e}"))?;

    // Best-effort: si existe destino, intentamos borrarlo primero
    let _ = tokio::fs::remove_file(&path).await;

    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| format!("No pude hacer rename del draft: {e}"))?;

    Ok(())
}

pub async fn read_draft(scene_id: &str) -> Result<Option<String>, String> {
    let path = draft_path(scene_id);
    if !Path::new(&path).exists() {
        return Ok(None);
    }

    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| format!("No pude leer draft: {e}"))?;

    let text = String::from_utf8(bytes).map_err(|e| format!("Draft no es UTF-8 válido: {e}"))?;
    Ok(Some(text))
}