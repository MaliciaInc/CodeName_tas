use crate::db::Database;
use crate::model::AuditLogEntry;
use uuid::Uuid;

impl Database {
    // =========================
    // Escritura: hook central
    // =========================
    //
    // Nota de performance:
    // - Todo es O(1).
    // - No serializamos payloads gigantes aquí.
    // - entity_id/details_json se bindean por referencia (&str) para evitar heap churn.
    pub async fn insert_audit_log(
        &self,
        action: &str,
        entity_type: &str,
        entity_id: &str,
        details_json: &str,
    ) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO audit_log (id, ts, action, entity_type, entity_id, details_json)
             VALUES (?, unixepoch(), ?, ?, ?, ?)",
        )
            .bind(id)
            .bind(action)
            .bind(entity_type)
            .bind(entity_id)
            .bind(details_json)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // =========================
    // Lectura
    // =========================
    pub async fn get_audit_log(&self, limit: i64) -> Result<Vec<AuditLogEntry>, sqlx::Error> {
        sqlx::query_as::<_, AuditLogEntry>(
            "SELECT id, ts, action, entity_type, entity_id, details_json
             FROM audit_log
             ORDER BY ts DESC
             LIMIT ?",
        )
            .bind(limit)
            .fetch_all(&self.pool)
            .await
    }
}
