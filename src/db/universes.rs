// ========================================
// universes.rs - Gestión de universos y snapshots
// ========================================
// Este módulo maneja la creación, actualización, eliminación y validación de universos.
// También incluye el sistema de snapshots (backup/restore).

use sqlx::Row;
use uuid::Uuid;
use flate2::{Compression, write::GzEncoder};
use base64::{engine::general_purpose, Engine as _};


use crate::model::{
    Universe, UniverseSnapshot, UniverseSnapshotPayload, Card,
};
use crate::db::Database;

impl Database {
    pub async fn get_all_universes(&self) -> Result<Vec<Universe>, sqlx::Error> {
        crate::logger::info("🔍 DB: Querying universes...");

        let result = sqlx::query_as::<_, Universe>(
            "SELECT id, name, description, archived
                        FROM universes
                        WHERE id != 'u-standalone'
                        ORDER BY name ASC"
        )
            .fetch_all(&self.pool)
            .await?;

        crate::logger::info(&format!("✅ DB: Found {} universes", result.len()));

        // ✅ NUEVO: Log cada universe encontrado
        for u in &result {
            crate::logger::info(&format!("  - {} ({})", u.name, u.id));
        }

        Ok(result)
    }

    pub async fn create_universe(&self, id: String, name: String, desc: String) -> Result<(), Box<dyn std::error::Error>> {
        // ✅ Guard de capability
        self.require_capability("universes").await?;

        sqlx::query("INSERT INTO universes (id, name, description) VALUES (?, ?, ?)")
            .bind(id)
            .bind(name)
            .bind(desc)
            .execute(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(())
    }

    pub async fn delete_universe(&self, id: String) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Bestiary (en schema viejo no tenía ON DELETE CASCADE)
        sqlx::query("DELETE FROM bestiary_entries WHERE universe_id = ?1")
            .bind(&id)
            .execute(&mut *tx)
            .await?;

        // Locations (en algunos schemas sí tiene cascade, pero borrarlo explícito no hace daño)
        sqlx::query("DELETE FROM locations WHERE universe_id = ?1")
            .bind(&id)
            .execute(&mut *tx)
            .await?;

        // Timeline
        sqlx::query("DELETE FROM timeline_events WHERE universe_id = ?1")
            .bind(&id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM timeline_eras WHERE universe_id = ?1")
            .bind(&id)
            .execute(&mut *tx)
            .await?;

        // Finalmente borrar universe
        sqlx::query("DELETE FROM universes WHERE id = ?1")
            .bind(&id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn validate_universe(&self, universe_id: String) -> Result<Vec<String>, sqlx::Error> {
        let mut issues: Vec<String> = Vec::new();
        let rows = sqlx::query("SELECT b.id, b.name, b.home_location_id FROM bestiary_entries b WHERE b.universe_id = ? AND b.home_location_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM locations l WHERE l.id = b.home_location_id)").bind(&universe_id).fetch_all(&self.pool).await?;
        for r in rows {
            issues.push(format!(
                "Creature '{}' ({}) references missing location_id={}",
                r.get::<String, _>("name"),
                r.get::<String, _>("id"),
                r.get::<String, _>("home_location_id")
            ));
        }
        Ok(issues)
    }

    // --- SNAPSHOTS ---

    pub async fn snapshot_list(&self, universe_id: String) -> Result<Vec<UniverseSnapshot>, sqlx::Error> {
        sqlx::query_as::<_, UniverseSnapshot>(
            "SELECT id, universe_id, name, created_at, size_bytes
         FROM universe_snapshots
         WHERE universe_id = ?
         ORDER BY created_at DESC"
        )
            .bind(universe_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn snapshot_create(&self, universe_id: String, name: String) -> Result<(), sqlx::Error> {
        let universe = sqlx::query_as::<_, Universe>(
            "SELECT id, name, description, archived FROM universes WHERE id = ?"
        )
            .bind(&universe_id)
            .fetch_one(&self.pool)
            .await?;

        let creatures = self.get_creatures(universe_id.clone()).await?;
        let locations = self.get_locations_flat(universe_id.clone()).await?;
        let eras = self.get_timeline_eras(universe_id.clone()).await?;
        let events = self.get_timeline_events(universe_id.clone()).await?;

        let pm_cards: Vec<Card> = sqlx::query_as::<_, Card>(
            "SELECT id, column_id, title, description, position, priority
         FROM cards
         WHERE column_id IN (
             SELECT id FROM board_columns WHERE board_id='board-main'
         )
         ORDER BY position ASC"
        )
            .fetch_all(&self.pool)
            .await?;

        let payload = UniverseSnapshotPayload {
            universe, creatures, locations,
            timeline_eras: eras,
            timeline_events: events,
            pm_cards,
        };

        // ✅ APLICADO: Serializar a bytes directamente
        let json_bytes = serde_json::to_vec(&payload)
            .map_err(|e| sqlx::Error::Protocol(format!("JSON serialize fail: {e}").into()))?;

        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        use std::io::Write;
        e.write_all(&json_bytes)
            .map_err(|e| sqlx::Error::Protocol(format!("Compress write fail: {e}").into()))?;

        let compressed = e.finish()
            .map_err(|e| sqlx::Error::Protocol(format!("Compress finish fail: {e}").into()))?;

        let size_bytes = compressed.len() as i64;
        let sid = format!("snap-{}", Uuid::new_v4());

        // ✅ APLICADO: Guardar como BLOB con Fallback
        let res = sqlx::query(
            "INSERT INTO universe_snapshots (id, universe_id, name, size_bytes, compressed_blob, compressed_b64)
             VALUES (?, ?, ?, ?, ?, '')"
        )
            .bind(&sid)
            .bind(&universe_id)
            .bind(&name)
            .bind(size_bytes)
            .bind(&compressed)
            .execute(&self.pool)
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("no such column: compressed_blob") {
                    let compressed_b64 = general_purpose::STANDARD.encode(compressed);
                    sqlx::query(
                        "INSERT INTO universe_snapshots (id, universe_id, name, size_bytes, compressed_b64)
                         VALUES (?, ?, ?, ?, ?)"
                    )
                        .bind(sid)
                        .bind(universe_id)
                        .bind(name)
                        .bind(size_bytes)
                        .bind(compressed_b64)
                        .execute(&self.pool)
                        .await?;
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    pub async fn snapshot_delete(&self, snapshot_id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM universe_snapshots WHERE id = ?")
            .bind(snapshot_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn snapshot_restore(&self, snapshot_id: String) -> Result<(), sqlx::Error> {
        // ✅ APLICADO: Lectura híbrida Blob/Base64
        let row_res = sqlx::query(
            "SELECT compressed_blob, compressed_b64 FROM universe_snapshots WHERE id = ?"
        )
            .bind(&snapshot_id)
            .fetch_one(&self.pool)
            .await;

        let (blob_opt, b64): (Option<Vec<u8>>, String) = match row_res {
            Ok(row) => (row.try_get("compressed_blob").ok(), row.get("compressed_b64")),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("no such column: compressed_blob") {
                    let row: sqlx::sqlite::SqliteRow =
                        sqlx::query("SELECT compressed_b64 FROM universe_snapshots WHERE id = ?")
                            .bind(&snapshot_id)
                            .fetch_one(&self.pool)
                            .await?;
                    (None, row.get("compressed_b64"))
                } else {
                    return Err(e);
                }
            }
        };

        let compressed_bytes = if let Some(b) = blob_opt {
            b
        } else {
            general_purpose::STANDARD
                .decode(b64)
                .map_err(|e| sqlx::Error::Protocol(format!("Base64 decode fail: {e}").into()))?
        };

        // ✅ APLICADO: Decompress a bytes
        let mut d = flate2::read::GzDecoder::new(&compressed_bytes[..]);
        let mut out = Vec::new();
        use std::io::Read;
        d.read_to_end(&mut out)
            .map_err(|e| sqlx::Error::Protocol(format!("Decompress fail: {e}").into()))?;

        let payload: UniverseSnapshotPayload =
            serde_json::from_slice(&out)
                .map_err(|e| sqlx::Error::Protocol(format!("JSON parse fail: {e}").into()))?;

        let mut tx = self.pool.begin().await?;

        sqlx::query("UPDATE universes SET name = ?, description = ?, archived = ? WHERE id = ?")
            .bind(payload.universe.name)
            .bind(payload.universe.description)
            .bind(payload.universe.archived)
            .bind(payload.universe.id.clone())
            .execute(&mut *tx).await?;

        sqlx::query("UPDATE bestiary_entries SET home_location_id = NULL WHERE universe_id = ?").bind(&payload.universe.id).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM bestiary_entries WHERE universe_id = ?").bind(&payload.universe.id).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM timeline_events WHERE universe_id = ?").bind(&payload.universe.id).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM timeline_eras WHERE universe_id = ?").bind(&payload.universe.id).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM locations WHERE universe_id = ?").bind(&payload.universe.id).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM cards WHERE column_id IN (SELECT id FROM board_columns WHERE board_id='board-main')").execute(&mut *tx).await?;

        for c in payload.creatures {
            sqlx::query("INSERT INTO bestiary_entries (id, universe_id, name, kind, habitat, description, danger, home_location_id, archived) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(c.id).bind(&payload.universe.id).bind(c.name).bind(c.kind).bind(c.habitat).bind(c.description).bind(c.danger).bind(c.home_location_id).bind(c.archived)
                .execute(&mut *tx).await?;
        }

        for l in payload.locations {
            sqlx::query("INSERT INTO locations (id, universe_id, parent_id, name, description, kind) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(l.id).bind(&payload.universe.id).bind(l.parent_id).bind(l.name).bind(l.description).bind(l.kind)
                .execute(&mut *tx).await?;
        }

        for e in payload.timeline_eras {
            sqlx::query("INSERT INTO timeline_eras (id, universe_id, name, start_year, end_year, description, color) VALUES (?, ?, ?, ?, ?, ?, ?)")
                .bind(e.id).bind(&payload.universe.id).bind(e.name).bind(e.start_year).bind(e.end_year.unwrap_or(0)).bind(e.description).bind(e.color)
                .execute(&mut *tx).await?;
        }

        for ev in payload.timeline_events {
            sqlx::query("INSERT INTO timeline_events (id, universe_id, title, description, year, display_date, importance, kind, color, location_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(ev.id).bind(&payload.universe.id).bind(ev.title).bind(ev.description).bind(ev.year).bind(ev.display_date).bind(ev.importance).bind(ev.kind).bind(ev.color).bind(ev.location_id)
                .execute(&mut *tx).await?;
        }

        for card in payload.pm_cards {
            sqlx::query("INSERT INTO cards (id, column_id, title, description, position, priority) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(card.id).bind(card.column_id).bind(card.title).bind(card.description).bind(card.position).bind(card.priority)
                .execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }
}