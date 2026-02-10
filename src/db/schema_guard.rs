// ========================================
// schema_guard.rs - Guardián de integridad de schema
// ========================================
// Este módulo asegura que el schema de la base de datos esté completo y actualizado,
// independientemente de qué migración se ejecutó. Agrega columnas faltantes, normaliza
// datos legacy y repara problemas conocidos.

use sqlx::SqlitePool;
use super::helpers::{table_exists, column_exists, ensure_column, fix_zero_ts};

pub async fn ensure_minimum_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {

    // --- Universes ---
    ensure_column(pool, "universes", "archived", "INTEGER NOT NULL DEFAULT 0").await?;

    // --- Trash System ---
    ensure_column(pool, "trash_entry", "target_type", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "trash_entry", "target_id", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "trash_entry", "parent_type", "TEXT").await?;
    ensure_column(pool, "trash_entry", "parent_id", "TEXT").await?;
    ensure_column(pool, "trash_entry", "display_name", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "trash_entry", "display_info", "TEXT").await?;

    // Si la tabla fue creada con las columnas viejas (entity_kind/entity_id), debemos limpiarla
    let has_entity_kind: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('trash_entry') WHERE name='entity_kind'"
    )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    if has_entity_kind > 0 {
        // La tabla tiene el schema viejo, recrearla
        sqlx::query("DROP TABLE IF EXISTS trash_entry").execute(pool).await?;
        sqlx::query(r#"
            CREATE TABLE trash_entry (
                id TEXT PRIMARY KEY NOT NULL,
                deleted_at INTEGER NOT NULL DEFAULT (unixepoch()),
                target_type TEXT NOT NULL,
                target_id TEXT NOT NULL,
                parent_type TEXT,
                parent_id TEXT,
                display_name TEXT NOT NULL DEFAULT '',
                display_info TEXT,
                payload_json TEXT NOT NULL DEFAULT '{}'
            )
            "#).execute(pool).await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_trash_deleted_at ON trash_entry(deleted_at)")
            .execute(pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_trash_target ON trash_entry(target_type, target_id)")
            .execute(pool).await?;
    }

    // --- PM Tools ---
    ensure_column(pool, "boards", "kind", "TEXT NOT NULL DEFAULT 'kanban'").await?;
    ensure_column(pool, "cards", "priority", "TEXT NOT NULL DEFAULT ''").await?;

    // --- The Forge ---
    ensure_column(pool, "scenes", "body", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "scenes", "status", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "scenes", "word_count", "INTEGER NOT NULL DEFAULT 0").await?;

    // C2: Canonizar scenes → usar chapter_id (y eliminar dependencia de story_id)
    // Si la DB viene legacy con scenes.story_id, reconstruimos scenes con FK a chapters.
    canonicalize_scenes_to_chapters_if_needed(pool).await?;

    // C8: DB como fuente de verdad → recalcular word_count desde body
    // (corrige data vieja / migraciones legacy / restores / snapshots)
    recalc_scene_word_counts(pool).await?;

    // --- Snapshots (tu código actual usa name + payload_json) ---

    ensure_column(pool, "universe_snapshots", "name", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "universe_snapshots", "payload_json", "TEXT NOT NULL DEFAULT ''").await?;

    // --- Timeline ---
    ensure_column(pool, "timeline_eras", "description", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "timeline_eras", "color", "TEXT NOT NULL DEFAULT ''").await?;

    ensure_column(pool, "timeline_events", "display_date", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "timeline_events", "importance", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "timeline_events", "kind", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "timeline_events", "color", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "timeline_events", "location_id", "TEXT").await?;

    // --- Locations & Bestiary (evita futuros "no such column …") ---
    ensure_column(pool, "locations", "parent_id", "TEXT").await?;
    ensure_column(pool, "locations", "kind", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "locations", "description", "TEXT NOT NULL DEFAULT ''").await?;

    ensure_column(pool, "bestiary_entries", "kind", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "bestiary_entries", "habitat", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "bestiary_entries", "description", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "bestiary_entries", "danger", "TEXT NOT NULL DEFAULT ''").await?;
    ensure_column(pool, "bestiary_entries", "home_location_id", "TEXT").await?;
    ensure_column(pool, "bestiary_entries", "archived", "INTEGER NOT NULL DEFAULT 0").await?;

    // --- CORE TABLES (timestamps) ---
    // Esto evita crashes tipo: "no column named updated_at" en PM/Timeline/Bestiary/Locations.
    ensure_column(pool, "universes", "created_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;
    ensure_column(pool, "universes", "updated_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;

    ensure_column(pool, "locations", "created_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;
    ensure_column(pool, "locations", "updated_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;

    ensure_column(pool, "bestiary_entries", "created_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;
    ensure_column(pool, "bestiary_entries", "updated_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;

    ensure_column(pool, "timeline_eras", "created_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;
    ensure_column(pool, "timeline_eras", "updated_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;

    ensure_column(pool, "timeline_events", "created_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;
    ensure_column(pool, "timeline_events", "updated_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;

    ensure_column(pool, "boards", "created_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;
    ensure_column(pool, "boards", "updated_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;

    ensure_column(pool, "board_columns", "created_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;
    ensure_column(pool, "board_columns", "updated_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;

    ensure_column(pool, "cards", "created_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;
    ensure_column(pool, "cards", "updated_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;

    // --- Novels, Chapters, Scenes (timestamps) ---
    ensure_column(pool, "novels", "created_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;
    ensure_column(pool, "novels", "updated_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;

    ensure_column(pool, "chapters", "created_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;
    ensure_column(pool, "chapters", "updated_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;

    ensure_column(pool, "scenes", "created_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;
    ensure_column(pool, "scenes", "updated_at", "INTEGER NOT NULL DEFAULT (unixepoch())").await?;


    // --- Normalización de valores (para que no queden null/empty raros) ---
    // Boards.kind
    sqlx::query("UPDATE boards SET kind='kanban' WHERE kind IS NULL OR kind=''")
        .execute(pool).await?;

    // Cards.priority
    sqlx::query("UPDATE cards SET priority='' WHERE priority IS NULL")
        .execute(pool).await?;

    // Scenes defaults
    sqlx::query("UPDATE scenes SET body='' WHERE body IS NULL")
        .execute(pool).await?;
    sqlx::query("UPDATE scenes SET status='' WHERE status IS NULL")
        .execute(pool).await?;
    sqlx::query("UPDATE scenes SET word_count=0 WHERE word_count IS NULL")
        .execute(pool).await?;

    // Snapshots defaults
    sqlx::query("UPDATE universe_snapshots SET name='' WHERE name IS NULL")
        .execute(pool).await?;
    sqlx::query("UPDATE universe_snapshots SET payload_json='' WHERE payload_json IS NULL")
        .execute(pool).await?;

    // ✅ Parte pedida: asegurar que ningún timestamp INTEGER se quede en 0
    // (Solo corre si la columna existe; no rompe tablas que no la tengan.)
    let tables = [
        "universes", "locations", "bestiary_entries", "timeline_eras", "timeline_events",
        "boards", "board_columns", "cards", "novels", "chapters", "scenes",
        "universe_snapshots", "db_meta",
    ];

    for t in tables {
        fix_zero_ts(pool, t, "created_at").await?;
        fix_zero_ts(pool, t, "updated_at").await?;
    }

    Ok(())
}

async fn recalc_scene_word_counts(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    use sqlx::Row;

    let mut tx = pool.begin().await?;

    let rows = sqlx::query("SELECT id, body, word_count FROM scenes")
        .fetch_all(&mut *tx)
        .await?;

    for r in rows {
        let id: String = r.try_get("id")?;
        let body: String = r.try_get("body")?;
        let current: i64 = r.try_get("word_count")?;

        // Regla simple y estable: tokens separados por whitespace
        let computed: i64 = body.split_whitespace().count() as i64;

        if computed != current {
            sqlx::query("UPDATE scenes SET word_count = ? WHERE id = ?")
                .bind(computed)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

async fn canonicalize_scenes_to_chapters_if_needed(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Si no hay scenes, nada que hacer
    if !table_exists(pool, "scenes").await? {
        return Ok(());
    }

    let has_story_id = column_exists(pool, "scenes", "story_id").await?;
    let has_chapter_id = column_exists(pool, "scenes", "chapter_id").await?;

    // Si ya está canónico, listo
    if !has_story_id && has_chapter_id {
        return Ok(());
    }

    // chapters debe existir
    if !table_exists(pool, "chapters").await? {
        return Err(sqlx::Error::Protocol(
            "Cannot canonicalize scenes: missing 'chapters' table".into(),
        ));
    }

    // Check ANTES de la tx (table_exists espera &SqlitePool)
    let has_novels = table_exists(pool, "novels").await.unwrap_or(false);

    let mut tx = pool.begin().await?;

    // --- Paso 1: Crear chapters faltantes para novels legacy story-xxx
    if has_novels {
        sqlx::query(
            r#"
            INSERT INTO chapters (id, novel_id, title, position, synopsis, status, created_at, updated_at)
            SELECT
            'chapter-' || substr(n.id, 7),
            n.id,
            'Chapter 1',
            0,
            '',
            COALESCE(n.status, 'draft'),
            COALESCE(n.created_at, unixepoch()),
            COALESCE(n.updated_at, unixepoch())
            FROM novels n
            WHERE n.id LIKE 'story-%'
            AND NOT EXISTS (
                SELECT 1 FROM chapters c
                WHERE c.id = ('chapter-' || substr(n.id, 7))
            )
            "#
        )
            .execute(&mut *tx)
            .await?;
    }

    // --- Paso 2: Crear scenes_new canónica con FK a chapters
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scenes_new (
            id TEXT PRIMARY KEY,
            chapter_id TEXT NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
            title TEXT NOT NULL,
            body TEXT NOT NULL DEFAULT '',
            position INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT '',
            word_count INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            updated_at INTEGER NOT NULL DEFAULT (unixepoch())
        )
        "#
    )
        .execute(&mut *tx)
        .await?;

    // --- Paso 3: Migrar datos desde scenes legacy
    if has_story_id {
        sqlx::query(
            r#"
            INSERT INTO scenes_new (id, chapter_id, title, body, position, status, word_count, created_at, updated_at)
            SELECT
            s.id,
            CASE
            WHEN s.chapter_id IS NOT NULL AND s.chapter_id <> '' THEN s.chapter_id
            ELSE ('chapter-' || substr(s.story_id, 7))
            END AS chapter_id,
            COALESCE(s.title, ''),
            COALESCE(s.body, ''),
            COALESCE(s.position, 0),
            COALESCE(s.status, ''),
            COALESCE(s.word_count, 0),
            COALESCE(s.created_at, unixepoch()),
            COALESCE(s.updated_at, unixepoch())
            FROM scenes s
            WHERE s.id IS NOT NULL
            "#
        )
            .execute(&mut *tx)
            .await?;
    } else {
        sqlx::query(
            r#"
        INSERT INTO scenes_new (id, chapter_id, title, body, position, status, word_count, created_at, updated_at)
        SELECT
        s.id,
        s.chapter_id,
        COALESCE(s.title, ''),
        COALESCE(s.body, ''),
        COALESCE(s.position, 0),
        COALESCE(s.status, ''),
        COALESCE(s.word_count, 0),
        COALESCE(s.created_at, unixepoch()),
        COALESCE(s.updated_at, unixepoch())
        FROM scenes s
        WHERE s.id IS NOT NULL
        "#
        )
            .execute(&mut *tx)
            .await?;
    }

    // --- Paso 4: Swap tablas
    sqlx::query("DROP TABLE IF EXISTS scenes")
        .execute(&mut *tx)
        .await?;

    sqlx::query("ALTER TABLE scenes_new RENAME TO scenes")
        .execute(&mut *tx)
        .await?;

    // --- Paso 5: Índices canónicos
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_scenes_chapter ON scenes(chapter_id)")
        .execute(&mut *tx)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_scenes_chapter_pos ON scenes(chapter_id, position)")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}