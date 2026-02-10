-- ============================================================
-- 0010_CoreSchema.sql
-- Titan Architect Studio (TAS) - Esquema Base Canónico v10
--
-- Única fuente de verdad para una base de datos nueva.
-- Incluye: core schema + relaciones + audit_log en una sola pasada.
--
-- Flujo esperado:
--   - borrar la DB
--   - app arranca -> corre migrations -> inyecta demo -> listo
-- ============================================================

PRAGMA foreign_keys = ON;

-- ============================================================
-- CORE: UNIVERSOS / LOCATIONS / BESTIARY / TIMELINE / PM
-- (equivalente al estado final de migrations 0001_init_core.sql)
-- ============================================================

CREATE TABLE IF NOT EXISTS universes (
                                         id TEXT PRIMARY KEY,
                                         name TEXT NOT NULL,
                                         description TEXT NOT NULL DEFAULT '',
                                         archived INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS locations (
                                         id TEXT PRIMARY KEY,
                                         universe_id TEXT NOT NULL
                                         REFERENCES universes(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    kind TEXT NOT NULL DEFAULT '',
    parent_id TEXT
    );

CREATE INDEX IF NOT EXISTS idx_locations_universe
    ON locations(universe_id);

CREATE TABLE IF NOT EXISTS bestiary_entries (
                                                id TEXT PRIMARY KEY,
                                                universe_id TEXT NOT NULL
                                                REFERENCES universes(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT '',
    habitat TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    danger TEXT NOT NULL DEFAULT '',
    home_location_id TEXT,
    archived INTEGER NOT NULL DEFAULT 0
    );

CREATE INDEX IF NOT EXISTS idx_bestiary_universe
    ON bestiary_entries(universe_id);

CREATE TABLE IF NOT EXISTS timeline_eras (
                                             id TEXT PRIMARY KEY,
                                             universe_id TEXT NOT NULL
                                             REFERENCES universes(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    start_year INTEGER NOT NULL DEFAULT 0,
    end_year INTEGER,
    color TEXT NOT NULL DEFAULT '#cccccc'
    );

CREATE INDEX IF NOT EXISTS idx_eras_universe
    ON timeline_eras(universe_id);

CREATE TABLE IF NOT EXISTS timeline_events (
                                               id TEXT PRIMARY KEY,
                                               universe_id TEXT NOT NULL
                                               REFERENCES universes(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    year INTEGER NOT NULL DEFAULT 0,
    display_date TEXT NOT NULL DEFAULT '',
    importance TEXT NOT NULL DEFAULT 'normal',
    kind TEXT NOT NULL DEFAULT 'event',
    color TEXT NOT NULL DEFAULT '#3498db',
    location_id TEXT
    );

CREATE INDEX IF NOT EXISTS idx_events_universe
    ON timeline_events(universe_id);

CREATE TABLE IF NOT EXISTS boards (
                                      id TEXT PRIMARY KEY,
                                      name TEXT NOT NULL,
                                      kind TEXT NOT NULL DEFAULT 'kanban'
);

CREATE TABLE IF NOT EXISTS board_columns (
                                             id TEXT PRIMARY KEY,
                                             board_id TEXT NOT NULL
                                             REFERENCES boards(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0
    );

CREATE INDEX IF NOT EXISTS idx_columns_board
    ON board_columns(board_id);

CREATE TABLE IF NOT EXISTS cards (
                                     id TEXT PRIMARY KEY,
                                     column_id TEXT NOT NULL
                                     REFERENCES board_columns(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    position INTEGER NOT NULL DEFAULT 0,
    priority TEXT NOT NULL DEFAULT 'normal'
    );

CREATE INDEX IF NOT EXISTS idx_cards_column
    ON cards(column_id);

-- ============================================================
-- THE FORGE (CANÓNICO): NOVELS / CHAPTERS / SCENES
-- (equivalente al estado final de 0003_forge_stories_scenes.sql)
-- ============================================================

CREATE TABLE IF NOT EXISTS novels (
                                      id TEXT PRIMARY KEY,
                                      universe_id TEXT
                                      REFERENCES universes(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    synopsis TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
    );

CREATE INDEX IF NOT EXISTS idx_novels_universe
    ON novels(universe_id);

CREATE TABLE IF NOT EXISTS chapters (
                                        id TEXT PRIMARY KEY,
                                        novel_id TEXT NOT NULL
                                        REFERENCES novels(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    synopsis TEXT NOT NULL DEFAULT '',
    position INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'draft',
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
    );

CREATE INDEX IF NOT EXISTS idx_chapters_novel
    ON chapters(novel_id);

CREATE INDEX IF NOT EXISTS idx_chapters_novel_pos
    ON chapters(novel_id, position);

CREATE TABLE IF NOT EXISTS scenes (
                                      id TEXT PRIMARY KEY,
                                      chapter_id TEXT NOT NULL
                                      REFERENCES chapters(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    position INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'draft',
    word_count INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
    );

CREATE INDEX IF NOT EXISTS idx_scenes_chapter
    ON scenes(chapter_id);

CREATE INDEX IF NOT EXISTS idx_scenes_chapter_pos
    ON scenes(chapter_id, position);

-- ============================================================
-- SNAPSHOTS: universe_snapshots + name
-- (estado canónico: 0004 + 0010 combinadas en una sola tabla)
-- ============================================================

CREATE TABLE IF NOT EXISTS universe_snapshots (
                                                  id TEXT PRIMARY KEY,
                                                  universe_id TEXT NOT NULL
                                                  REFERENCES universes(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    size_bytes INTEGER NOT NULL DEFAULT 0,
    compressed_b64 TEXT NOT NULL,
    name TEXT NOT NULL DEFAULT ''
    );

CREATE INDEX IF NOT EXISTS idx_universe_snapshots_universe
    ON universe_snapshots(universe_id);

CREATE INDEX IF NOT EXISTS idx_universe_snapshots_created_at
    ON universe_snapshots(created_at);

CREATE INDEX IF NOT EXISTS idx_universe_snapshots_name
    ON universe_snapshots(name);

-- ============================================================
-- SISTEMA DE PAPELERA (TRASH)
-- (equivalente al estado final de 0008_trash.sql)
-- ============================================================

CREATE TABLE IF NOT EXISTS trash_entry (
                                           id TEXT PRIMARY KEY NOT NULL,
                                           deleted_at INTEGER NOT NULL DEFAULT (unixepoch()),
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    parent_type TEXT,
    parent_id TEXT,
    display_name TEXT NOT NULL DEFAULT '',
    display_info TEXT,
    payload_json TEXT NOT NULL DEFAULT '{}'
    );

CREATE INDEX IF NOT EXISTS idx_trash_deleted_at
    ON trash_entry(deleted_at);

CREATE INDEX IF NOT EXISTS idx_trash_target
    ON trash_entry(target_type, target_id);

-- ============================================================
-- DB META (forma canónica de 0009_db_meta_canonicalize.sql)
-- ============================================================

CREATE TABLE IF NOT EXISTS db_meta (
                                       schema_version INTEGER NOT NULL,
                                       container_kind TEXT NOT NULL DEFAULT 'tas_studio',
                                       enabled_capabilities_json TEXT NOT NULL DEFAULT '[
      "worldbuilding",
      "timeline",
      "pm",
      "novel",
      "snapshots",
      "trash"
    ]',
                                       created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    app_version TEXT NOT NULL DEFAULT ''
    );

-- Asegurar que exista al menos una fila
INSERT INTO db_meta (schema_version, container_kind, enabled_capabilities_json, created_at, app_version)
SELECT
    10,
    'tas_studio',
    '[
      "worldbuilding",
      "timeline",
      "pm",
      "novel",
      "snapshots",
      "trash"
    ]',
    unixepoch(),
    ''
    WHERE NOT EXISTS (SELECT 1 FROM db_meta);

-- ============================================================
-- NUEVO: TIPOS DE RELACIÓN + RELACIONES
-- (antes planeado como 0011_relationships.sql)
-- ============================================================

CREATE TABLE IF NOT EXISTS relationship_types (
                                                  id TEXT PRIMARY KEY,
                                                  name TEXT NOT NULL UNIQUE,
                                                  description TEXT NOT NULL DEFAULT '',
                                                  directed INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS relationships (
                                             id TEXT PRIMARY KEY,
                                             relationship_type_id TEXT NOT NULL
                                             REFERENCES relationship_types(id) ON DELETE RESTRICT,
    from_type TEXT NOT NULL,
    from_id TEXT NOT NULL,
    to_type TEXT NOT NULL,
    to_id TEXT NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
    );

CREATE INDEX IF NOT EXISTS idx_relationships_from
    ON relationships(from_type, from_id);

CREATE INDEX IF NOT EXISTS idx_relationships_to
    ON relationships(to_type, to_id);

CREATE INDEX IF NOT EXISTS idx_relationships_type
    ON relationships(relationship_type_id);

-- ============================================================
-- NUEVO: AUDIT LOG
-- (antes planeado como 0012_audit_log.sql)
-- ============================================================

CREATE TABLE IF NOT EXISTS audit_log (
                                         id TEXT PRIMARY KEY,
                                         ts INTEGER NOT NULL DEFAULT (unixepoch()),
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    details_json TEXT NOT NULL DEFAULT ''
    );

CREATE INDEX IF NOT EXISTS idx_audit_log_ts
    ON audit_log(ts);

CREATE INDEX IF NOT EXISTS idx_audit_log_entity
    ON audit_log(entity_type, entity_id);

-- ============================================================
-- FINAL: STAMP DE VERSIÓN (como pediste)
-- ============================================================

UPDATE db_meta SET schema_version = 10;
