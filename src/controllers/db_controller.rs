use iced::Task;

use crate::app::Message;
use crate::db::Database;
use crate::state::DbAction;


/// Spawn an iced Task that executes a DbAction and reports back as Message::ActionDone.
/// Keep UI/controllers clean: all DbAction execution logic lives here.
pub fn task_execute(db: Database, action: DbAction) -> Task<Message> {
    Task::perform(async move { execute(db, action).await }, Message::ActionDone)
}

/// Execute a DbAction against the Database.
/// IMPORTANT: Minimal, no refactors, no invented behavior. Mirrors the previous match logic.
pub async fn execute(db: Database, action: DbAction) -> Result<(), String> {
    match action {
        // -----------------------------
        // UNIVERSE
        // -----------------------------
        DbAction::CreateUniverse { id, name, desc } => {
            db.create_universe(id, name, desc).await.map_err(|e| e.to_string())
        }

        // -----------------------------
        // DEMO DATA
        // -----------------------------
        DbAction::InjectDemoData(id) => db.inject_demo_data(id).await.map_err(|e| e.to_string()),
        DbAction::ResetDemoDataScoped(id, scope) => db
            .reset_demo_data_scoped(id, scope)
            .await
            .map_err(|e| e.to_string()),

        // -----------------------------
        // SNAPSHOTS
        // -----------------------------
        DbAction::SnapshotCreate { universe_id, name } => db
            .snapshot_create(universe_id, name)
            .await
            .map_err(|e| e.to_string()),
        DbAction::SnapshotDelete { snapshot_id } => db
            .snapshot_delete(snapshot_id)
            .await
            .map_err(|e| e.to_string()),
        DbAction::SnapshotRestore { snapshot_id } => db
            .snapshot_restore(snapshot_id)
            .await
            .map_err(|e| e.to_string()),

        // -----------------------------
        // PM BOARDS
        // -----------------------------
        DbAction::CreateBoard { id, name } => {
            db.create_board(id, name).await.map_err(|e| e.to_string())
        }

        // -----------------------------
        // BESTIARY
        // -----------------------------
        DbAction::SaveCreature(c, uid) => db.upsert_creature(c, uid).await.map_err(|e| e.to_string()),
        DbAction::ArchiveCreature(id, st) => db.set_creature_archived(id, st).await.map_err(|e| e.to_string()),

        // -----------------------------
        // LOCATIONS
        // -----------------------------
        DbAction::SaveLocation(l) => db.upsert_location(l).await.map_err(|e| e.to_string()),

        // -----------------------------
        // TIMELINE
        // -----------------------------
        DbAction::SaveEvent(e) => db.upsert_timeline_event(e).await.map_err(|e| e.to_string()),
        DbAction::SaveEra(e) => db.upsert_timeline_era(e).await.map_err(|e| e.to_string()),

        // -----------------------------
        // PM CARDS
        // -----------------------------
        DbAction::SaveCard(c) => db.upsert_card(c).await.map_err(|e| e.to_string()),
        DbAction::MoveCard(cid, col, pos) => db.move_card(cid, col, pos).await.map_err(|e| e.to_string()),
        DbAction::RebalanceColumn(col) => db.rebalance_column(col).await.map_err(|e| e.to_string()),
        DbAction::DeleteCard(id) => db.delete_card(id).await.map_err(|e| e.to_string()),

        // -----------------------------
        // THE FORGE (NOVEL/CHAPTER/SCENE)
        // -----------------------------
        DbAction::CreateNovel(novel_id, universe_id, title) => db
            .create_novel_with_id(novel_id, universe_id, title)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string()),
        DbAction::UpdateNovel(novel) => db.update_novel(novel).await.map_err(|e| e.to_string()),

        DbAction::CreateChapter(chapter_id, novel_id, title) => db
            .create_chapter_with_id(chapter_id, novel_id, title)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string()),
        DbAction::UpdateChapter(chapter) => db.update_chapter(chapter).await.map_err(|e| e.to_string()),
        DbAction::ReorderChapter(chapter_id, new_position) => db.reorder_chapter(chapter_id, new_position).await.map_err(|e| e.to_string()),

        DbAction::CreateScene(scene_id, chapter_id, title) =>
            db.create_scene_with_id(scene_id, chapter_id, title)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string()),
        DbAction::UpdateScene(scene) => db.update_scene(scene).await.map_err(|e| e.to_string()),
        DbAction::ReorderScene(scene_id, new_position) => db.reorder_scene(scene_id, new_position).await.map_err(|e| e.to_string()),

        // -----------------------------
        // TRASH SYSTEM
        // -----------------------------
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

        DbAction::EmptyTrash => {
            db.empty_trash().await.map_err(|e| e.to_string())
        }

        DbAction::CleanupOldTrash => {
            db.cleanup_old_trash(14).await.map_err(|e| e.to_string()).map(|_| ())
        }
    }
}