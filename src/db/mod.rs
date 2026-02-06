// ========================================
// mod.rs - Punto de entrada principal del módulo DB
// ========================================
// Este archivo organiza todos los submódulos y expone la estructura Database

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteJournalMode, SqliteSynchronous};
use std::{sync::OnceLock, time::Duration};
use std::path::PathBuf;

// Submódulos - cada uno maneja una área específica de funcionalidad
mod helpers;
mod migrations;
mod schema_guard;
mod universes;
mod locations;
mod creatures;
mod timeline;
mod kanban;
mod novels;
mod trash;
mod demo;
mod audit;

// Re-exportar la estructura principal
#[derive(Debug, Clone)]
pub struct Database {
    pub pool: SqlitePool,
    pub capabilities: crate::guards::CapabilitiesCache,
}

static DB_CONNECT_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

impl Database {
    pub async fn connect(db_path: PathBuf) -> Result<Self, sqlx::Error> {
        // ✅ Evita múltiples connects concurrentes (UI/tasks)
        let _guard = DB_CONNECT_LOCK
            .get_or_init(|| tokio::sync::Mutex::new(()))
            .lock()
            .await;

        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            // ✅ Espera locks en vez de fallar inmediato
            .busy_timeout(Duration::from_secs(15))
            // ✅ Reduce contención lector/escritor
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);

        // ✅ SQLite desktop: 1 conexión suele ser lo más estable
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        // PRAGMAs extra por si alguna conexión no heredó settings
        sqlx::query("PRAGMA foreign_keys = ON;").execute(&pool).await?;
        sqlx::query("PRAGMA busy_timeout = 15000;").execute(&pool).await?;
        sqlx::query("PRAGMA journal_mode = WAL;").execute(&pool).await?;
        sqlx::query("PRAGMA synchronous = NORMAL;").execute(&pool).await?;

        // ✅ Retry si SQLite está ocupado (code 5)
        for attempt in 1..=5 {
            match migrations::apply(&pool).await {
                Ok(_) => break,
                Err(e) if Self::is_sqlite_locked(&e) && attempt < 5 => {
                    crate::logger::warn(&format!(
                        "SQLite locked during migrations (attempt {}/5). Retrying...",
                        attempt
                    ));
                    tokio::time::sleep(Duration::from_millis(250 * attempt as u64)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        // ✅ Blindaje: alinea DB con el modelo real de la app
        schema_guard::ensure_minimum_schema(&pool).await?;

        // ✅ Cargar capabilities al conectar
        let capabilities_cache = crate::guards::create_empty_cache();

        match crate::guards::fetch_capabilities_from_db(&pool).await {
            Ok(caps) => {
                let mut cache_lock = capabilities_cache.write().await;
                *cache_lock = caps;
            }
            Err(e) => {
                crate::logger::warn(&format!(
                    "⚠️ Could not load capabilities: {}. Keeping fail-closed defaults (all disabled).",
                    e
                ));
            }
        }

        let db = Self {
            pool,
            capabilities: capabilities_cache,
        };

        db.repair_integrity().await?;

        // Auto-cleanup de trash (14 días)
        match db.cleanup_old_trash(14).await {
            Ok(count) if count > 0 => {
                crate::logger::info(&format!("🗑️ Auto-cleanup: {} old items removed from trash", count));
            }
            Ok(_) => {}
            Err(e) => {
                crate::logger::warn(&format!("⚠️ Trash auto-cleanup failed: {}", e));
            }
        }

        Ok(db)
    }

    /// Helper para verificar capabilities antes de operaciones
    async fn require_capability(&self, capability: &str) -> Result<(), Box<dyn std::error::Error>> {
        crate::guards::check_capability(&self.capabilities, capability).await
    }

    // Detecta "database is locked" (SQLite code 5)
    fn is_sqlite_locked(e: &sqlx::Error) -> bool {
        match e {
            sqlx::Error::Database(db) => db.code().as_deref() == Some("5")
                || db.message().to_lowercase().contains("database is locked"),
            _ => e.to_string().to_lowercase().contains("database is locked"),
        }
    }

    pub async fn get_schema_version(&self) -> Result<i64, sqlx::Error> {
        migrations::read_schema_version(&self.pool).await
    }

    pub async fn validate_integrity(&self) -> Result<Vec<String>, sqlx::Error> {
        // Verificación real (SQLite): foreign keys
        // Devuelve una lista de strings amigables para UI/debug.
        use sqlx::Row;

        let rows = sqlx::query("PRAGMA foreign_key_check;")
            .fetch_all(&self.pool)
            .await?;

        let mut issues: Vec<String> = Vec::new();

        for r in rows {
            let table: String = r.try_get("table").unwrap_or_else(|_| "<unknown_table>".to_string());
            // SQLite entrega rowid como INTEGER; lo recibimos como i64 para safety.
            let rowid: i64 = r.try_get("rowid").unwrap_or(-1);
            let parent: String =
                r.try_get("parent").unwrap_or_else(|_| "<unknown_parent>".to_string());
            let fkid: i64 = r.try_get("fkid").unwrap_or(-1);

            issues.push(format!(
                "ForeignKey violation: table='{}' rowid={} parent='{}' fkid={}",
                table, rowid, parent, fkid
            ));
        }

        Ok(issues)
    }


    async fn repair_integrity(&self) -> Result<(), sqlx::Error> {
        // Primero asegurarse que existe el universo default
        sqlx::query(
            "INSERT OR IGNORE INTO universes (id, name, description) VALUES \
                    ('u-arhelis-01', 'Arhelis', 'Un mundo fracturado por la magia antigua.')",
        )
            .execute(&self.pool)
            .await?;

        // Luego crear el board principal
        sqlx::query(
            "INSERT OR IGNORE INTO boards (id, name) VALUES \
                    ('board-main', 'Development Roadmap')",
        )
            .execute(&self.pool)
            .await?;

        // Finalmente las columnas (que dependen del board)
        sqlx::query(
            "INSERT OR IGNORE INTO board_columns (id, board_id, name, position) VALUES \
                    ('col-hold', 'board-main', 'On-Hold', 0), \
                    ('col-todo', 'board-main', 'To Do', 1), \
                    ('col-progress', 'board-main', 'In Progress', 2), \
                    ('col-done', 'board-main', 'Done', 3)",
        )
            .execute(&self.pool)
            .await?;

        self.repair_legacy_kanban().await?;
        Ok(())
    }

    async fn repair_legacy_kanban(&self) -> Result<(), sqlx::Error> {
        let expected = vec![
            ("col-hold", "On-Hold", 0),
            ("col-todo", "To Do", 1),
            ("col-progress", "In Progress", 2),
            ("col-done", "Done", 3),
        ];

        for (id, name, pos) in expected {
            sqlx::query("INSERT OR IGNORE INTO board_columns (id, board_id, name, position) VALUES (?, 'board-main', ?, ?)")
                .bind(id)
                .bind(name)
                .bind(pos)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }
}