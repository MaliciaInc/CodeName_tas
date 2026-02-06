// ========================================
// trash.rs - Sistema de papelera (soft delete)
// ========================================
// Este módulo maneja el soft delete con posibilidad de restauración

use sqlx::{Row, SqlitePool};
use uuid::Uuid;
use crate::model::{TrashEntry, Universe, Board, Novel, Chapter, Scene, Creature, Location, TimelineEvent, TimelineEra};
use crate::db::Database;

impl Database {
    pub async fn move_to_trash(
        &self,
        target_type: &str,
        target_id: &str,
        display_name: &str,
        display_info: Option<&str>,
        parent_type: Option<&str>,
        parent_id: Option<&str>,
        payload_json: &str,
    ) -> Result<String, sqlx::Error> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO trash_entry (id, deleted_at, target_type, target_id, parent_type, parent_id, display_name, display_info, payload_json)
                    VALUES (?, unixepoch(), ?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(&id)
            .bind(target_type)
            .bind(target_id)
            .bind(parent_type)
            .bind(parent_id)
            .bind(display_name)
            .bind(display_info)
            .bind(payload_json)
            .execute(&self.pool)
            .await?;

        Ok(id)
    }

    /// Atomically: insert trash_entry + delete the original rows in ONE transaction.
    pub async fn move_to_trash_and_delete(
        &self,
        target_type: &str,
        target_id: &str,
        display_name: &str,
        display_info: Option<&str>,
        parent_type: Option<&str>,
        parent_id: Option<&str>,
        payload_json: &str,
    ) -> Result<String, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let trash_id = Uuid::new_v4().to_string();

        // 1) Insert trash entry
        sqlx::query(
            "INSERT INTO trash_entry (id, deleted_at, target_type, target_id, parent_type, parent_id, display_name, display_info, payload_json)
            VALUES (?, unixepoch(), ?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(&trash_id)
            .bind(target_type)
            .bind(target_id)
            .bind(parent_type)
            .bind(parent_id)
            .bind(display_name)
            .bind(display_info)
            .bind(payload_json)
            .execute(&mut *tx)
            .await?;

        // 1.1) Audit log
        let audit_id = Uuid::new_v4().to_string();
        let details_json = format!(
            r#"{{"trash_id":"{}","display_name":{},"parent_type":{},"parent_id":{}}}"#,
            trash_id,
            serde_json::to_string(display_name).unwrap_or_else(|_| "\"\"".to_string()),
            match parent_type {
                Some(v) => serde_json::to_string(v).unwrap_or_else(|_| "null".to_string()),
                None => "null".to_string(),
            },
            match parent_id {
                Some(v) => serde_json::to_string(v).unwrap_or_else(|_| "null".to_string()),
                None => "null".to_string(),
            }
        );

        sqlx::query(
            "INSERT INTO audit_log (id, ts, action, entity_type, entity_id, details_json)
             VALUES (?, unixepoch(), ?, ?, ?, ?)"
        )
            .bind(&audit_id)
            .bind("trash_move_and_delete")
            .bind(target_type)
            .bind(target_id)
            .bind(details_json)
            .execute(&mut *tx)
            .await?;

        // 2) Delete source rows
        match target_type {
            "universe" => {
                sqlx::query("DELETE FROM bestiary_entries WHERE universe_id = ?1")
                    .bind(target_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM locations WHERE universe_id = ?1")
                    .bind(target_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM timeline_events WHERE universe_id = ?1")
                    .bind(target_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM timeline_eras WHERE universe_id = ?1")
                    .bind(target_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM universes WHERE id = ?1")
                    .bind(target_id).execute(&mut *tx).await?;
            }
            "board" => {
                sqlx::query("DELETE FROM cards WHERE column_id IN (SELECT id FROM board_columns WHERE board_id = ?)")
                    .bind(target_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM board_columns WHERE board_id = ?")
                    .bind(target_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM boards WHERE id = ?")
                    .bind(target_id).execute(&mut *tx).await?;
            }
            "novel" => {
                sqlx::query("DELETE FROM novels WHERE id = ?")
                    .bind(target_id).execute(&mut *tx).await?;
            }
            "chapter" => {
                sqlx::query("DELETE FROM chapters WHERE id = ?")
                    .bind(target_id).execute(&mut *tx).await?;
            }
            "scene" => {
                sqlx::query("DELETE FROM scenes WHERE id = ?")
                    .bind(target_id).execute(&mut *tx).await?;
            }
            "creature" => {
                sqlx::query("DELETE FROM bestiary_entries WHERE id = ?")
                    .bind(target_id).execute(&mut *tx).await?;
            }
            "location" => {
                sqlx::query("DELETE FROM locations WHERE id = ?")
                    .bind(target_id).execute(&mut *tx).await?;
            }
            "event" => {
                sqlx::query("DELETE FROM timeline_events WHERE id = ?")
                    .bind(target_id).execute(&mut *tx).await?;
            }
            "era" => {
                sqlx::query("DELETE FROM timeline_eras WHERE id = ?")
                    .bind(target_id).execute(&mut *tx).await?;
            }
            other => {
                return Err(sqlx::Error::Protocol(
                    format!("Unknown trash target_type: {}", other).into(),
                ));
            }
        }

        tx.commit().await?;
        Ok(trash_id)
    }

    pub async fn get_trash_entries(&self) -> Result<Vec<TrashEntry>, sqlx::Error> {
        sqlx::query_as::<_, TrashEntry>(
            "SELECT id, deleted_at, target_type, target_id, parent_type, parent_id, display_name, display_info, payload_json
            FROM trash_entry ORDER BY deleted_at DESC"
        )
            .fetch_all(&self.pool)
            .await
    }

    pub async fn permanent_delete(&self, trash_entry_id: &str) -> Result<(), sqlx::Error> {
        let row = sqlx::query(
            "SELECT target_type, target_id, display_name FROM trash_entry WHERE id = ? LIMIT 1"
        )
            .bind(trash_entry_id)
            .fetch_optional(&self.pool)
            .await?;

        sqlx::query("DELETE FROM trash_entry WHERE id = ?")
            .bind(trash_entry_id)
            .execute(&self.pool)
            .await?;

        if let Some(r) = row {
            let target_type: String = r.get("target_type");
            let target_id: String = r.get("target_id");
            let display_name: String = r.get("display_name");

            let audit_id = Uuid::new_v4().to_string();
            let details_json = format!(
                r#"{{"trash_entry_id":"{}","display_name":{}}}"#,
                trash_entry_id,
                serde_json::to_string(&display_name).unwrap_or_else(|_| "\"\"".to_string())
            );

            sqlx::query(
                "INSERT INTO audit_log (id, ts, action, entity_type, entity_id, details_json)
                 VALUES (?, unixepoch(), ?, ?, ?, ?)"
            )
                .bind(audit_id)
                .bind("trash_permanent_delete")
                .bind(target_type)
                .bind(target_id)
                .bind(details_json)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    pub async fn empty_trash(&self) -> Result<(), sqlx::Error> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM trash_entry")
            .fetch_one(&self.pool)
            .await?;

        sqlx::query("DELETE FROM trash_entry")
            .execute(&self.pool)
            .await?;

        let audit_id = Uuid::new_v4().to_string();
        let details_json = format!(r#"{{"count":{}}}"#, count);

        sqlx::query(
            "INSERT INTO audit_log (id, ts, action, entity_type, entity_id, details_json)
             VALUES (?, unixepoch(), ?, ?, ?, ?)"
        )
            .bind(audit_id)
            .bind("trash_empty")
            .bind("trash")
            .bind("trash")
            .bind(details_json)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn cleanup_old_trash(&self, days: i64) -> Result<usize, sqlx::Error> {
        let days = days.max(0);

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs() as i64;

        let seconds = days.saturating_mul(24 * 60 * 60);
        let cutoff = now_secs.saturating_sub(seconds);

        let result = sqlx::query("DELETE FROM trash_entry WHERE deleted_at < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as usize)
    }

    pub async fn restore_from_trash(&self, trash_entry_id: &str) -> Result<(), sqlx::Error> {
        // 1) Capability gate
        if let Err(e) = self.require_capability("trash").await {
            return Err(sqlx::Error::Protocol(format!("Trash restore blocked by capability: {}", e).into()));
        }

        // 2) Helper para validación FK
        async fn ensure_exists(pool: &SqlitePool, kind: &str, id: &str) -> Result<(), sqlx::Error> {
            let table = match kind {
                "universes" => "universes",
                "novels" => "novels",
                "chapters" => "chapters",
                "locations" => "locations",
                _ => {
                    return Err(sqlx::Error::Protocol(
                        format!("ensure_exists called with unsupported kind: {}", kind).into(),
                    ));
                }
            };

            let q = format!("SELECT 1 FROM {} WHERE id = ? LIMIT 1", table);
            let row = sqlx::query(&q).bind(id).fetch_optional(pool).await?;
            if row.is_none() {
                return Err(sqlx::Error::Protocol(
                    format!("Cannot restore: missing {} record id={}", kind, id).into(),
                ));
            }
            Ok(())
        }

        let entry: TrashEntry = sqlx::query_as("SELECT * FROM trash_entry WHERE id = ?")
            .bind(trash_entry_id)
            .fetch_one(&self.pool)
            .await?;

        match entry.target_type.as_str() {
            "universe" => {
                if let Err(e) = self.require_capability("worldbuilding").await {
                    return Err(sqlx::Error::Protocol(format!("Universe restore blocked by capability: {}", e).into()));
                }
                let universe: Universe = serde_json::from_str(&entry.payload_json)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                self.restore_universe(universe).await?;
            }
            "board" => {
                if let Err(e) = self.require_capability("pm").await {
                    return Err(sqlx::Error::Protocol(format!("Board restore blocked by capability: {}", e).into()));
                }
                let board: Board = serde_json::from_str(&entry.payload_json)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                self.restore_board(board).await?;
            }
            "novel" => {
                if let Err(e) = self.require_capability("novel").await {
                    return Err(sqlx::Error::Protocol(format!("Novel restore blocked by capability: {}", e).into()));
                }
                let novel: Novel = serde_json::from_str(&entry.payload_json)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                if let Some(universe_id) = novel.universe_id.as_deref() {
                    ensure_exists(&self.pool, "universes", universe_id).await?;
                }
                self.restore_novel(novel).await?;
            }
            "chapter" => {
                if let Err(e) = self.require_capability("novel").await {
                    return Err(sqlx::Error::Protocol(format!("Chapter restore blocked by capability: {}", e).into()));
                }
                let chapter: Chapter = serde_json::from_str(&entry.payload_json)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                ensure_exists(&self.pool, "novels", &chapter.novel_id).await?;
                self.restore_chapter(chapter).await?;
            }
            "scene" => {
                if let Err(e) = self.require_capability("novel").await {
                    return Err(sqlx::Error::Protocol(format!("Scene restore blocked by capability: {}", e).into()));
                }
                let scene: Scene = serde_json::from_str(&entry.payload_json)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                ensure_exists(&self.pool, "chapters", &scene.chapter_id).await?;
                self.restore_scene(scene).await?;
            }
            "creature" => {
                if let Err(e) = self.require_capability("worldbuilding").await {
                    return Err(sqlx::Error::Protocol(format!("Creature restore blocked by capability: {}", e).into()));
                }
                let creature: Creature = serde_json::from_str(&entry.payload_json)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                let universe_id = entry
                    .parent_id
                    .as_ref()
                    .ok_or_else(|| sqlx::Error::Decode("Missing parent_id for creature".into()))?;
                ensure_exists(&self.pool, "universes", universe_id).await?;
                if let Some(loc_id) = creature.home_location_id.as_deref() {
                    ensure_exists(&self.pool, "locations", loc_id).await?;
                }
                self.restore_creature(creature, universe_id).await?;
            }
            "location" => {
                if let Err(e) = self.require_capability("worldbuilding").await {
                    return Err(sqlx::Error::Protocol(format!("Location restore blocked by capability: {}", e).into()));
                }
                let location: Location = serde_json::from_str(&entry.payload_json)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                ensure_exists(&self.pool, "universes", &location.universe_id).await?;
                if let Some(parent_id) = location.parent_id.as_deref() {
                    ensure_exists(&self.pool, "locations", parent_id).await?;
                }
                self.restore_location(location).await?;
            }
            "event" => {
                if let Err(e) = self.require_capability("timeline").await {
                    return Err(sqlx::Error::Protocol(format!("Event restore blocked by capability: {}", e).into()));
                }
                let event: TimelineEvent = serde_json::from_str(&entry.payload_json)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                ensure_exists(&self.pool, "universes", &event.universe_id).await?;
                if let Some(loc_id) = event.location_id.as_deref() {
                    ensure_exists(&self.pool, "locations", loc_id).await?;
                }
                self.restore_event(event).await?;
            }
            "era" => {
                if let Err(e) = self.require_capability("timeline").await {
                    return Err(sqlx::Error::Protocol(format!("Era restore blocked by capability: {}", e).into()));
                }
                let era: TimelineEra = serde_json::from_str(&entry.payload_json)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                ensure_exists(&self.pool, "universes", &era.universe_id).await?;
                self.restore_era(era).await?;
            }
            other => {
                return Err(sqlx::Error::Protocol(
                    format!("Unsupported target_type in trash restore: {}", other).into(),
                ));
            }
        }

        sqlx::query("DELETE FROM trash_entry WHERE id = ?")
            .bind(trash_entry_id)
            .execute(&self.pool)
            .await?;

        let audit_id = Uuid::new_v4().to_string();
        let details_json = format!(r#"{{"trash_entry_id":"{}"}}"#, trash_entry_id);

        sqlx::query(
            "INSERT INTO audit_log (id, ts, action, entity_type, entity_id, details_json)
             VALUES (?, unixepoch(), ?, ?, ?, ?)"
        )
            .bind(audit_id)
            .bind("trash_restore")
            .bind("trash")
            .bind(trash_entry_id)
            .bind(details_json)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // --- Helper functions for restore ---

    async fn restore_universe(&self, universe: Universe) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO universes (id, name, description, archived)
                VALUES (?, ?, ?, ?)"
        )
            .bind(&universe.id)
            .bind(&universe.name)
            .bind(&universe.description)
            .bind(&universe.archived)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn restore_creature(&self, creature: Creature, universe_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO bestiary_entries (id, universe_id, name, kind, habitat, description, danger, home_location_id, archived)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(&creature.id)
            .bind(universe_id)
            .bind(&creature.name)
            .bind(&creature.kind)
            .bind(&creature.habitat)
            .bind(&creature.description)
            .bind(&creature.danger)
            .bind(&creature.home_location_id)
            .bind(&creature.archived)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn restore_location(&self, location: Location) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO locations (id, universe_id, parent_id, name, description, kind)
                VALUES (?, ?, ?, ?, ?, ?)"
        )
            .bind(&location.id)
            .bind(&location.universe_id)
            .bind(&location.parent_id)
            .bind(&location.name)
            .bind(&location.description)
            .bind(&location.kind)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn restore_event(&self, event: TimelineEvent) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO timeline_events (id, universe_id, title, display_date, year, description, location_id)
                VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(&event.id)
            .bind(&event.universe_id)
            .bind(&event.title)
            .bind(&event.display_date)
            .bind(&event.year)
            .bind(&event.description)
            .bind(&event.location_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn restore_era(&self, era: TimelineEra) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO timeline_eras (id, universe_id, name, start_year, end_year, description, color)
                VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(&era.id)
            .bind(&era.universe_id)
            .bind(&era.name)
            .bind(&era.start_year)
            .bind(&era.end_year)
            .bind(&era.description)
            .bind(&era.color)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn restore_board(&self, board: Board) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO boards (id, name, kind)
                VALUES (?, ?, ?)"
        )
            .bind(&board.id)
            .bind(&board.name)
            .bind(&board.kind)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn restore_novel(&self, novel: Novel) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO novels (id, universe_id, title, synopsis, status)
                VALUES (?, ?, ?, ?, ?)"
        )
            .bind(&novel.id)
            .bind(&novel.universe_id)
            .bind(&novel.title)
            .bind(&novel.synopsis)
            .bind(&novel.status)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn restore_chapter(&self, chapter: Chapter) -> Result<(), sqlx::Error> {
        let (exists,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM novels WHERE id = ?")
            .bind(&chapter.novel_id)
            .fetch_one(&self.pool)
            .await?;

        if exists == 0 {
            return Err(sqlx::Error::Protocol(
                format!(
                    "Cannot restore chapter {}: novel {} not found",
                    chapter.id, chapter.novel_id
                )
                    .into(),
            ));
        }

        sqlx::query(
            "INSERT INTO chapters (id, novel_id, title, position, synopsis, status)
                    VALUES (?, ?, ?, ?, ?, ?)",
        )
            .bind(&chapter.id)
            .bind(&chapter.novel_id)
            .bind(&chapter.title)
            .bind(chapter.position)
            .bind(&chapter.synopsis)
            .bind(&chapter.status)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn restore_scene(&self, scene: Scene) -> Result<(), sqlx::Error> {
        let (exists,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chapters WHERE id = ?")
            .bind(&scene.chapter_id)
            .fetch_one(&self.pool)
            .await?;

        if exists == 0 {
            return Err(sqlx::Error::Protocol(
                format!(
                    "Cannot restore scene {}: chapter {} not found",
                    scene.id, scene.chapter_id
                )
                    .into(),
            ));
        }

        sqlx::query(
            "INSERT INTO scenes (id, chapter_id, title, body, position, status, word_count)
                        VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
            .bind(&scene.id)
            .bind(&scene.chapter_id)
            .bind(&scene.title)
            .bind(&scene.body)
            .bind(scene.position)
            .bind(&scene.status)
            .bind(scene.word_count)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}