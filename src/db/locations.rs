// ========================================
// locations.rs - Gestión de locaciones
// ========================================
// Este módulo maneja CRUD de locaciones con soporte para jerarquías (parent_id)

use crate::model::Location;
use crate::db::Database;

impl Database {
    pub async fn get_locations_flat(&self, universe_id: String) -> Result<Vec<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            "SELECT id, universe_id, parent_id, name, description, kind FROM locations WHERE universe_id = ? ORDER BY name ASC"
        )
            .bind(universe_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn upsert_location(&self, l: Location) -> Result<(), Box<dyn std::error::Error>> {
        // ✅ Guard de capability
        self.require_capability("locations").await?;

        sqlx::query("INSERT INTO locations (id, universe_id, parent_id, name, description, kind, updated_at) VALUES (?, ?, ?, ?, ?, ?, unixepoch()) ON CONFLICT(id) DO UPDATE SET parent_id=excluded.parent_id, name=excluded.name, description=excluded.description, kind=excluded.kind, updated_at=unixepoch()")
            .bind(l.id).bind(l.universe_id).bind(l.parent_id).bind(l.name).bind(l.description).bind(l.kind)
            .execute(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(())
    }

    pub async fn delete_location(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM locations WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }
}