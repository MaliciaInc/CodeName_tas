// ========================================
// audit.rs - Sistema de auditoría
// ========================================
// Este módulo maneja el registro de acciones para auditoría

use crate::model::AuditLogEntry;
use crate::db::Database;

impl Database {
    // El sistema de auditoría ya está integrado en las funciones de trash
    // Aquí solo exponemos consultas

    pub async fn get_audit_log(&self, limit: i64) -> Result<Vec<AuditLogEntry>, sqlx::Error> {
        sqlx::query_as::<_, AuditLogEntry>(
            "SELECT id, ts, action, entity_type, entity_id, details_json
             FROM audit_log
             ORDER BY ts DESC
             LIMIT ?"
        )
            .bind(limit)
            .fetch_all(&self.pool)
            .await
    }
}