// ========================================
// helpers.rs - Funciones auxiliares para toda la base de datos
// ========================================
// Este módulo contiene utilidades comunes usadas por múltiples módulos de DB:
// - Verificación de existencia de tablas y columnas
// - Agregar columnas dinámicamente
// - Reparar timestamps en 0

use sqlx::SqlitePool;

/// Verifica si una tabla existe en la base de datos
pub async fn table_exists(pool: &SqlitePool, table: &str) -> Result<bool, sqlx::Error> {
    let hit: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name=? LIMIT 1"
    )
        .bind(table)
        .fetch_optional(pool)
        .await?;

    Ok(hit.is_some())
}

/// Verifica si una columna existe en una tabla
pub async fn column_exists(pool: &SqlitePool, table: &str, column: &str) -> Result<bool, sqlx::Error> {
    // pragma_table_info('table') es table-valued function; table va literal en SQL
    let sql = format!(
        "SELECT 1 FROM pragma_table_info('{table}') WHERE name = ? LIMIT 1"
    );

    let hit: Option<i64> = sqlx::query_scalar(&sql)
        .bind(column)
        .fetch_optional(pool)
        .await?;

    Ok(hit.is_some())
}

/// Agrega una columna a una tabla si no existe
pub async fn ensure_column(
    pool: &SqlitePool,
    table: &str,
    column: &str,
    col_def: &str,
) -> Result<(), sqlx::Error> {
    if column_exists(pool, table, column).await? {
        return Ok(());
    }

    let sql = format!("ALTER TABLE {table} ADD COLUMN {column} {col_def}");
    sqlx::query(&sql).execute(pool).await?;
    Ok(())
}

/// Repara timestamps que están en 0 (convierte a timestamp actual)
pub async fn fix_zero_ts(pool: &SqlitePool, table: &str, col: &str) -> Result<(), sqlx::Error> {
    if !column_exists(pool, table, col).await? {
        return Ok(());
    }

    // Solo arregla casos donde el timestamp está en 0 (numérico o texto "0")
    let sql = format!(
        "UPDATE {table} SET {col} = unixepoch() WHERE {col} = 0 OR {col} = '0'"
    );
    sqlx::query(&sql).execute(pool).await?;
    Ok(())
}
