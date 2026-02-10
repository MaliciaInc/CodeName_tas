use iced::Task;

use crate::app::Message;
use crate::db::Database;
use crate::state::DbAction;

/// Spawn an iced Task that executes a DbAction and reports back as Message::ActionDone.
/// Keep UI/controllers clean: all DbAction execution logic lives here.
pub fn task_execute(db: Database, action: DbAction) -> Task<Message> {
    Task::perform(async move { execute(db, action).await }, Message::ActionDone)
}

#[derive(Debug)]
struct AuditSpec {
    action: &'static str,
    entity_type: &'static str,
    entity_id: String,
    // Mantenerlo liviano por ahora. Cuando quieras, lo enriquecés con JSON real.
    details_json: &'static str,
}

/// Execute a DbAction against the Database.
/// IMPORTANT: Mantiene el comportamiento; agrega hook de auditoría post-éxito.
pub async fn execute(db: Database, action: DbAction) -> Result<(), String> {
    let mut audit: Option<AuditSpec> = None;

    let result: Result<(), String> = match action {
        // -----------------------------
        // UNIVERSE
        // -----------------------------
        DbAction::CreateUniverse { id, name, desc } => {
            // Nota: clon pequeño e inevitable aquí si queremos log + mantener firma actual.
            // (Alternativa “perfecta”: cambiar DB API a &str, pero eso ya es refactor más grande.)
            audit = Some(AuditSpec {
                action: "create_universe",
                entity_type: "universe",
                entity_id: id.clone(),
                details_json: "",
            });

            db.create_universe(id, name, desc).await.map_err(|e| e.to_string())
        }

        // -----------------------------
        // DEMO DATA
        // -----------------------------
        DbAction::InjectDemoData(id) => {
            audit = Some(AuditSpec {
                action: "inject_demo_data",
                entity_type: "universe",
                entity_id: id.clone(),
                details_json: "",
            });

            db.inject_demo_data(id).await.map_err(|e| e.to_string())
        }

        DbAction::ResetDemoDataScoped(id, scope) => {
            // Scope no lo serializo aún para no meter strings grandes; si querés lo metemos luego.
            audit = Some(AuditSpec {
                action: "reset_demo_data_scoped",
                entity_type: "universe",
                entity_id: id.clone(),
                details_json: "",
            });

            db.reset_demo_data_scoped(id, scope).await.map_err(|e| e.to_string())
        }

        // -----------------------------
        // SNAPSHOTS
        // -----------------------------
        DbAction::SnapshotCreate { universe_id, name } => {
            audit = Some(AuditSpec {
                action: "snapshot_create",
                entity_type: "universe",
                entity_id: universe_id.clone(),
                details_json: "",
            });

            db.snapshot_create(universe_id, name)
                .await
                .map_err(|e| e.to_string())
        }
        DbAction::SnapshotDelete { snapshot_id } => {
            audit = Some(AuditSpec {
                action: "snapshot_delete",
                entity_type: "snapshot",
                entity_id: snapshot_id.clone(),
                details_json: "",
            });

            db.snapshot_delete(snapshot_id)
                .await
                .map_err(|e: sqlx::Error| e.to_string())
        }

        DbAction::SnapshotRestore { snapshot_id } => {
            audit = Some(AuditSpec {
                action: "snapshot_restore",
                entity_type: "snapshot",
                entity_id: snapshot_id.clone(),
                details_json: "",
            });

            db.snapshot_restore(snapshot_id).await.map_err(|e| e.to_string())
        }

        // -----------------------------
        // PM BOARDS
        // -----------------------------
        DbAction::CreateBoard { id, name } => {
            audit = Some(AuditSpec {
                action: "create_board",
                entity_type: "board",
                entity_id: id.clone(),
                details_json: "",
            });

            db.create_board(id, name).await.map_err(|e| e.to_string())
        }

        // -----------------------------
        // BESTIARY
        // -----------------------------
        DbAction::SaveCreature(c, uid) => {
            // Evitamos clonar payloads; solo el ID (pequeño).
            audit = Some(AuditSpec {
                action: "save_creature",
                entity_type: "creature",
                entity_id: c.id.clone(),
                details_json: "",
            });

            db.upsert_creature(c, uid).await.map_err(|e| e.to_string())
        }

        DbAction::ArchiveCreature(id, st) => {
            audit = Some(AuditSpec {
                action: if st { "archive_creature" } else { "restore_creature" },
                entity_type: "creature",
                entity_id: id.clone(),
                details_json: "",
            });

            db.set_creature_archived(id, st).await.map_err(|e| e.to_string())
        }

        // -----------------------------
        // LOCATIONS
        // -----------------------------
        DbAction::SaveLocation(l) => {
            audit = Some(AuditSpec {
                action: "save_location",
                entity_type: "location",
                entity_id: l.id.clone(),
                details_json: "",
            });

            db.upsert_location(l).await.map_err(|e| e.to_string())
        }

        // -----------------------------
        // TIMELINE
        // -----------------------------
        DbAction::SaveEvent(e) => {
            audit = Some(AuditSpec {
                action: "save_timeline_event",
                entity_type: "timeline_event",
                entity_id: e.id.clone(),
                details_json: "",
            });

            db.upsert_timeline_event(e).await.map_err(|e| e.to_string())
        }

        DbAction::SaveEra(e) => {
            audit = Some(AuditSpec {
                action: "save_timeline_era",
                entity_type: "timeline_era",
                entity_id: e.id.clone(),
                details_json: "",
            });

            db.upsert_timeline_era(e).await.map_err(|e| e.to_string())
        }

        // -----------------------------
        // PM CARDS
        // -----------------------------
        DbAction::SaveCard(c) => {
            audit = Some(AuditSpec {
                action: "save_card",
                entity_type: "card",
                entity_id: c.id.clone(),
                details_json: "",
            });

            db.upsert_card(c).await.map_err(|e| e.to_string())
        }

        DbAction::MoveCard(cid, col, pos) => {
            audit = Some(AuditSpec {
                action: "move_card",
                entity_type: "card",
                entity_id: cid.clone(),
                details_json: "",
            });

            db.move_card(cid, col, pos).await.map_err(|e| e.to_string())
        }

        DbAction::RebalanceColumn(col) => {
            audit = Some(AuditSpec {
                action: "rebalance_column",
                entity_type: "board_column",
                entity_id: col.clone(),
                details_json: "",
            });

            db.rebalance_column(col).await.map_err(|e| e.to_string())
        }

        DbAction::DeleteCard(id) => {
            audit = Some(AuditSpec {
                action: "delete_card",
                entity_type: "card",
                entity_id: id.clone(),
                details_json: "",
            });

            db.delete_card(id).await.map_err(|e| e.to_string())
        }

        // -----------------------------
        // THE FORGE (NOVEL/CHAPTER/SCENE)
        // -----------------------------
        DbAction::CreateNovel(novel_id, universe_id, title) => {
            audit = Some(AuditSpec {
                action: "create_novel",
                entity_type: "novel",
                entity_id: novel_id.clone(),
                details_json: "",
            });

            db.create_novel_with_id(novel_id, universe_id, title)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }

        DbAction::UpdateNovel(novel) => {
            audit = Some(AuditSpec {
                action: "update_novel",
                entity_type: "novel",
                entity_id: novel.id.clone(),
                details_json: "",
            });

            db.update_novel(novel).await.map_err(|e| e.to_string())
        }

        DbAction::CreateChapter(chapter_id, novel_id, title) => {
            audit = Some(AuditSpec {
                action: "create_chapter",
                entity_type: "chapter",
                entity_id: chapter_id.clone(),
                details_json: "",
            });

            db.create_chapter_with_id(chapter_id, novel_id, title)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }

        DbAction::UpdateChapter(chapter) => {
            audit = Some(AuditSpec {
                action: "update_chapter",
                entity_type: "chapter",
                entity_id: chapter.id.clone(),
                details_json: "",
            });

            db.update_chapter(chapter).await.map_err(|e| e.to_string())
        }

        DbAction::ReorderChapter(chapter_id, new_position) => {
            audit = Some(AuditSpec {
                action: "reorder_chapter",
                entity_type: "chapter",
                entity_id: chapter_id.clone(),
                details_json: "",
            });

            db.reorder_chapter(chapter_id, new_position)
                .await
                .map_err(|e| e.to_string())
        }

        DbAction::CreateScene(scene_id, chapter_id, title) => {
            audit = Some(AuditSpec {
                action: "create_scene",
                entity_type: "scene",
                entity_id: scene_id.clone(),
                details_json: "",
            });

            db.create_scene_with_id(scene_id, chapter_id, title)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }

        DbAction::UpdateScene(scene) => {
            audit = Some(AuditSpec {
                action: "update_scene",
                entity_type: "scene",
                entity_id: scene.id.clone(),
                details_json: "",
            });

            db.update_scene(scene).await.map_err(|e| e.to_string())
        }

        DbAction::ReorderScene(scene_id, new_position) => {
            audit = Some(AuditSpec {
                action: "reorder_scene",
                entity_type: "scene",
                entity_id: scene_id.clone(),
                details_json: "",
            });

            db.reorder_scene(scene_id, new_position)
                .await
                .map_err(|e| e.to_string())
        }

        // -----------------------------
        // TRASH SYSTEM
        // -----------------------------
        // IMPORTANTE:
        // - Trash ya tiene lógica propia (y típicamente es transaccional).
        // - Para evitar doble-log o inconsistencias, no metemos audit aquí.
        DbAction::MoveToTrash {
            target_type,
            target_id,
            display_name,
            display_info,
            parent_type,
            parent_id,
            payload_json,
        } => {
            db.move_to_trash_and_delete(
                &target_type,
                &target_id,
                &display_name,
                display_info.as_deref(),
                parent_type.as_deref(),
                parent_id.as_deref(),
                &payload_json,
            )
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }

        DbAction::RestoreFromTrash(entry_id) => {
            db.restore_from_trash(&entry_id).await.map_err(|e| e.to_string())
        }

        DbAction::PermanentDelete(entry_id) => {
            db.permanent_delete(&entry_id).await.map_err(|e| e.to_string())
        }

        DbAction::EmptyTrash => db.empty_trash().await.map_err(|e| e.to_string()),

        DbAction::CleanupOldTrash => db
            .cleanup_old_trash(14)
            .await
            .map_err(|e| e.to_string())
            .map(|_| ()),
    };

    // Hook de auditoría SOLO si la acción fue exitosa.
    if result.is_ok() {
        if let Some(a) = audit {
            // Si falla el audit, NO rompemos la acción principal.
            // Esto es clave: auditoría es “best effort” al inicio; luego podés endurecerlo.
            if let Err(e) = db
                .insert_audit_log(a.action, a.entity_type, &a.entity_id, a.details_json)
                .await
            {
                crate::logger::error(&format!(
                    "⚠️ audit_log insert failed: action={} entity_type={} entity_id={} err={}",
                    a.action, a.entity_type, a.entity_id, e
                ));
            }
        }
    }

    result
}
