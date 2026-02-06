// ========================================
// creatures.rs - Gestión del bestiario
// ========================================
// Este módulo maneja CRUD de criaturas (bestiary_entries)

use crate::model::Creature;
use crate::db::Database;

impl Database {
    pub async fn get_creatures(&self, universe_id: String) -> Result<Vec<Creature>, sqlx::Error> {
        sqlx::query_as::<_, Creature>(
            "SELECT id, name, kind, habitat, description, danger, home_location_id, archived
                        FROM bestiary_entries
                        WHERE universe_id = ?"
        )
            .bind(universe_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn upsert_creature(&self, c: Creature, universe_id: String) -> Result<(), Box<dyn std::error::Error>> {
        // ✅ Guard de capability
        self.require_capability("bestiary").await?;

        sqlx::query("INSERT INTO bestiary_entries (id, universe_id, name, kind, habitat, description, danger, home_location_id, archived, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch()) ON CONFLICT(id) DO UPDATE SET name=excluded.name, kind=excluded.kind, habitat=excluded.habitat, description=excluded.description, danger=excluded.danger, home_location_id=excluded.home_location_id, archived=excluded.archived, updated_at=unixepoch()")
            .bind(c.id).bind(universe_id).bind(c.name).bind(c.kind).bind(c.habitat).bind(c.description).bind(c.danger).bind(c.home_location_id).bind(c.archived)
            .execute(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(())
    }

    pub async fn set_creature_archived(&self, id: String, archived: bool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE bestiary_entries SET archived = ? WHERE id = ?").bind(archived).bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_creature(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM bestiary_entries WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }
}