// ========================================
// timeline.rs - Gestión de timeline (eras y eventos)
// ========================================
// Este módulo maneja CRUD de eras y eventos de timeline

use crate::model::{TimelineEra, TimelineEvent};
use crate::db::Database;

impl Database {
    pub async fn get_timeline_eras(&self, universe_id: String) -> Result<Vec<TimelineEra>, sqlx::Error> {
        sqlx::query_as::<_, TimelineEra>("SELECT id, universe_id, name, start_year, NULLIF(end_year, 0) as end_year, description, color FROM timeline_eras WHERE universe_id = ? ORDER BY start_year ASC")
            .bind(universe_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_timeline_events(&self, universe_id: String) -> Result<Vec<TimelineEvent>, sqlx::Error> {
        sqlx::query_as::<_, TimelineEvent>("SELECT id, universe_id, title, description, year, display_date, importance, kind, color, location_id FROM timeline_events WHERE universe_id = ? ORDER BY year ASC")
            .bind(universe_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn upsert_timeline_era(&self, e: TimelineEra) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO timeline_eras (id, universe_id, name, start_year, end_year, description, color, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, unixepoch()) ON CONFLICT(id) DO UPDATE SET name=excluded.name, start_year=excluded.start_year, end_year=excluded.end_year, description=excluded.description, color=excluded.color, updated_at=unixepoch()")
            .bind(e.id).bind(e.universe_id).bind(e.name).bind(e.start_year).bind(e.end_year.unwrap_or(0)).bind(e.description).bind(e.color)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_timeline_era(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM timeline_eras WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn upsert_timeline_event(&self, e: TimelineEvent) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO timeline_events (id, universe_id, title, description, year, display_date, importance, kind, color, location_id, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch()) ON CONFLICT(id) DO UPDATE SET title=excluded.title, description=excluded.description, year=excluded.year, display_date=excluded.display_date, importance=excluded.importance, kind=excluded.kind, color=excluded.color, location_id=excluded.location_id, updated_at=unixepoch()")
            .bind(e.id).bind(e.universe_id).bind(e.title).bind(e.description).bind(e.year).bind(e.display_date).bind(e.importance).bind(e.kind).bind(e.color).bind(e.location_id)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_timeline_event(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM timeline_events WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }
}