// ========================================
// migrations.rs - Sistema de migraciones de base de datos
// ========================================
// Este módulo gestiona las migraciones de la base de datos usando sqlx migrate.
// Maneja tanto bases de datos nuevas como legacy que necesitan ser "stampadas".

use sqlx::{migrate::Migrator, SqlitePool};

// Fuente única de verdad: ./migrations (en la raíz del crate)
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub const CURRENT_SCHEMA_VERSION: i64 = 10;

pub async fn apply(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // 1) Si ya existe la tabla de migraciones de sqlx, solo corremos normal.
    if sqlx_migrations_table_exists(pool).await? {
        MIGRATOR.run(pool).await?;
        sync_db_meta_schema_version(pool).await?;
        return Ok(());
    }

    // 2) Si NO existe, puede ser:
    //    (a) DB nueva/limpia -> correr migraciones normal
    //    (b) DB legacy ya migrada por el sistema anterior -> bootstrap/stamp
    if legacy_db_looks_already_migrated(pool).await? {
        ensure_sqlx_migrations_table(pool).await?;
        stamp_all_migrations_as_applied(pool).await?;
        sync_db_meta_schema_version(pool).await?;
        return Ok(());
    }

    // 3) Caso default: DB nueva (o muy vieja) sin tabla _sqlx_migrations.
    MIGRATOR.run(pool).await?;
    sync_db_meta_schema_version(pool).await?;
    Ok(())
}

pub async fn read_schema_version(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    if !sqlx_migrations_table_exists(pool).await? {
        return Ok(0);
    }

    // MAX(version) donde success=TRUE
    let v: Option<i64> = sqlx::query_scalar(
        "SELECT MAX(version) FROM _sqlx_migrations WHERE success = TRUE"
    )
        .fetch_one(pool)
        .await?;

    Ok(v.unwrap_or(0))
}

async fn sqlx_migrations_table_exists(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'"
    )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    Ok(count > 0)
}

async fn legacy_db_looks_already_migrated(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    // Heurística simple y efectiva:
    // - Tiene novelas/capítulos
    // - scenes tiene chapter_id
    // - (opcional) db_meta existe
    let has_novels: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='novels'"
    )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let has_chapters: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chapters'"
    )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let scenes_has_chapter_id: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('scenes') WHERE name='chapter_id'"
    )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    Ok(has_novels > 0 && has_chapters > 0 && scenes_has_chapter_id > 0)
}

async fn ensure_sqlx_migrations_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Es exactamente el schema que SQLx crea para SQLite
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time BIGINT NOT NULL
        );
        "#,
    )
        .execute(pool)
        .await?;

    Ok(())
}

async fn stamp_all_migrations_as_applied(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Marcamos TODAS las migraciones embebidas como aplicadas con checksum real.
    for m in MIGRATOR.iter() {
        sqlx::query(
            r#"
            INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
            VALUES (?1, ?2, TRUE, ?3, -1)
            ON CONFLICT(version) DO UPDATE SET
            description=excluded.description,
            success=excluded.success,
            checksum=excluded.checksum,
            execution_time=excluded.execution_time;
            "#,
        )
            .bind(m.version)
            .bind(m.description.as_ref())
            .bind(m.checksum.as_ref())
            .execute(pool)
            .await?;
    }

    Ok(())
}

async fn sync_db_meta_schema_version(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Mantén db_meta.schema_version alineado (si existe db_meta).
    let has_db_meta: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='db_meta'"
    )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    if has_db_meta > 0 {
        let v = read_schema_version(pool).await.unwrap_or(CURRENT_SCHEMA_VERSION);
        let _ = sqlx::query("UPDATE db_meta SET schema_version = ?1")
            .bind(v)
            .execute(pool)
            .await;
    }

    Ok(())
}
