// ========================================
// demo.rs - Inyección y reseteo de datos demo
// ========================================
// Este módulo maneja la inyección de datos de ejemplo y el reseteo selectivo por scopes

use crate::state::DemoResetScope;
use crate::db::Database;

impl Database {
    pub async fn inject_demo_data(&self, universe_id: String) -> Result<(), sqlx::Error> {
        crate::db_seed::run_all(&self.pool, &universe_id).await?;
        Ok(())
    }

    pub async fn reset_demo_data_scoped(&self, universe_id: String, scope: DemoResetScope) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        match scope {
            DemoResetScope::All => {
                sqlx::query("UPDATE bestiary_entries SET home_location_id = NULL WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM bestiary_entries WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM timeline_events WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM timeline_eras WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM locations WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM cards WHERE column_id IN (SELECT id FROM board_columns WHERE board_id='board-main')").execute(&mut *tx).await?;
            }
            DemoResetScope::Timeline => {
                sqlx::query("DELETE FROM timeline_events WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM timeline_eras WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
            }
            DemoResetScope::Locations => {
                sqlx::query("UPDATE bestiary_entries SET home_location_id = NULL WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM locations WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
            }
            DemoResetScope::Bestiary => {
                sqlx::query("DELETE FROM bestiary_entries WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
            }
            DemoResetScope::PmTools => {
                sqlx::query("DELETE FROM cards WHERE column_id IN (SELECT id FROM board_columns WHERE board_id='board-main')").execute(&mut *tx).await?;
            }
        }
        tx.commit().await?;
        self.repair_integrity().await?;
        match scope {
            DemoResetScope::All => { crate::db_seed::run_all(&self.pool, &universe_id).await?; }
            DemoResetScope::Timeline => { crate::db_seed::timeline::seed(&self.pool, &universe_id).await?; }
            DemoResetScope::Locations => { crate::db_seed::locations::seed(&self.pool, &universe_id).await?; }
            DemoResetScope::Bestiary => { crate::db_seed::bestiary::seed(&self.pool, &universe_id).await?; }
            DemoResetScope::PmTools => { crate::db_seed::pm_tools::seed(&self.pool).await?; }
        }
        Ok(())
    }
}