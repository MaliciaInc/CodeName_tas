use std::time::Instant;
use std::collections::{HashMap, HashSet, VecDeque};
use iced::widget::text_editor;

use crate::model::{
    Creature, Universe, Card, KanbanBoardData, Board, Location, TimelineEvent, TimelineEra, Project, UniverseSnapshot,
    Novel, Chapter, Scene, TrashEntry
};
use crate::app::{Route, PmState, PmId};
use crate::editors::{CreatureEditor, LocationEditor, EventEditor, EraEditor};

// ================================
// FASE 13 (PRO): Observabilidad (Debug Overlay)
// ================================
#[derive(Debug, Clone)]
pub enum DebugEventKind {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct DebugEvent {
    pub at: Instant,
    pub kind: DebugEventKind,
    pub msg: String,
}

#[derive(Debug, Clone)]
pub struct DebugInvalidation {
    pub at: Instant,
    pub scope: String,  // "novels" | "chapters" | "scenes" | "core:*" ...
    pub key: String,    // "*" o novel_id / chapter_id / universe_id
    pub reason: String, // texto humano
}

#[derive(Debug, Clone)]
pub struct DebugIgnored {
    pub at: Instant,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoResetScope {
    All,
    Timeline,
    Locations,
    Bestiary,
    PmTools,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DbAction {
    CreateUniverse { id: String, name: String, desc: String },
    InjectDemoData(String),
    ResetDemoDataScoped(String, DemoResetScope),

    SnapshotCreate { universe_id: String, name: String },
    SnapshotDelete { snapshot_id: String },
    SnapshotRestore { snapshot_id: String },

    CreateBoard { id: String, name: String },

    SaveCreature(Creature, String),
    ArchiveCreature(String, bool),

    SaveLocation(Location),

    SaveEvent(TimelineEvent),

    SaveEra(TimelineEra),

    SaveCard(Card),
    MoveCard(String, String, i64),
    RebalanceColumn(String),
    DeleteCard(String),

    // --- THE FORGE ACTIONS ---
    CreateNovel(String, Option<String>, String), // (novel_id, universe_id, title)
    UpdateNovel(Novel),

    CreateChapter(String, String, String), // (chapter_id, novel_id, title)
    UpdateChapter(Chapter),
    ReorderChapter(String, i64),

    CreateScene(String, String, String), // (scene_id, chapter_id, title)
    UpdateScene(Scene),
    ReorderScene(String, i64),


    MoveToTrash {
        target_type: String,
        target_id: String,
        display_name: String,
        display_info: Option<String>,
        parent_type: Option<String>,
        parent_id: Option<String>,
        payload_json: String,
    },
    RestoreFromTrash(String),      // trash_entry_id
    PermanentDelete(String),        // trash_entry_id
    EmptyTrash,
    CleanupOldTrash,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ForgeLoadKey {
    Novels,
    Chapters { novel_id: String },
    Scenes { chapter_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreLoadKey {
    UniversesList,
    BoardsList,
    PmBoard { board_id: String },
    Creatures { universe_id: String },
    Locations { universe_id: String },
    Timeline { universe_id: String },
    Snapshots { universe_id: String },
}

// --- PM (Project Manager) hot-path intern pool ---
// Mantiene DB en String/TEXT, pero en runtime reusa Arc<str> para evitar heap churn.
#[derive(Debug)]
pub struct PmIdPool {
    map: std::collections::HashMap<String, crate::app::PmId>,
}

impl Default for PmIdPool {
    fn default() -> Self {
        Self { map: std::collections::HashMap::new() }
    }
}

impl PmIdPool {
    pub fn rebuild_from_pm(&mut self, pm: &crate::model::KanbanBoardData) {
        self.map.clear();

        // Column IDs
        for c in &pm.columns {
            let arc: crate::app::PmId = std::sync::Arc::from(c.id.as_str());
            self.map.insert(c.id.clone(), arc);
        }

        // Card IDs + column_id (por si acaso)
        for card in pm.cards_by_id.values() {
            if !self.map.contains_key(card.id.as_str()) {
                let arc: crate::app::PmId = std::sync::Arc::from(card.id.as_str());
                self.map.insert(card.id.clone(), arc);
            }
            if !self.map.contains_key(card.column_id.as_str()) {
                let arc: crate::app::PmId = std::sync::Arc::from(card.column_id.as_str());
                self.map.insert(card.column_id.clone(), arc);
            }
        }
    }

    #[inline]
    pub fn get(&self, id: &str) -> crate::app::PmId {
        // En condiciones normales todo está internado tras BoardLoaded.
        // Fallback defensivo: evita panic, pero puede allocar si se usa.
        self.map.get(id).cloned().unwrap_or_else(|| std::sync::Arc::from(id))
    }
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub kind: ToastKind,
    pub created_at: Instant,
    pub ttl_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastKind { Info, Success, Error }

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteUniverse(String),
    DeleteBoard(String),
    DeleteNovel(String),
    DeleteChapter(String),
    DeleteScene(String),
    DeleteLocation(String),
    DeleteCreature(String),
    DeleteEvent(String),
    DeleteEra(String),
}

#[derive(Debug)]
pub struct AppState {
    pub route: Route,

    pub active_project: Option<Project>,
    pub projects: Vec<Project>,
    pub is_creating_project: bool,
    pub new_project_name: String,

    pub universes: Vec<Universe>,
    pub new_universe_name: String,
    pub new_universe_desc: String,

    pub pending_confirm: Option<ConfirmAction>,
    pub dev_panel_open: bool,

    // ================================
    // FASE 13 (PRO): Observabilidad
    // ================================
    pub debug_overlay_open: bool,
    pub debug_schema_version: Option<i64>,
    pub debug_events: Vec<crate::state::DebugEvent>,
    pub debug_last_invalidation: Option<crate::state::DebugInvalidation>,
    pub debug_last_ignored: Option<crate::state::DebugIgnored>,

    pub snapshot_name: String,
    pub snapshots: Vec<UniverseSnapshot>,

    pub integrity_issues: Vec<String>,
    pub integrity_busy: bool,

    pub loaded_creatures_universe: Option<String>,
    pub loaded_locations_universe: Option<String>,
    pub loaded_timeline_universe: Option<String>,
    pub loaded_snapshots_universe: Option<String>,
    pub loaded_forge_universe: Option<String>,

    pub data_dirty: bool,

    pub creatures: Vec<Creature>,
    // ✅ REFACTOR A.3: Cache para búsquedas O(1) por ID
    pub creatures_index: HashMap<String, usize>, // creature_id -> index in Vec
    pub locations: Vec<Location>,
    // ✅ OPTIMIZED: Cache de estructura jerárquica para evitar O(n) en cada render
    pub locations_children_map: HashMap<Option<String>, Vec<String>>, // parent_id -> Vec<child_id>
    pub timeline_events: Vec<TimelineEvent>,
    pub timeline_eras: Vec<TimelineEra>,

    // --- THE FORGE V2 ---
    pub novels: Vec<Novel>,
    pub active_novel_id: Option<String>,

    // ✅ Vista del novel activo (solo para panel/UX)
    pub active_novel_chapters: Vec<Chapter>,

    // ✅ Fuente de verdad: chapters por novel
    pub chapters_by_novel_id: std::collections::HashMap<String, Vec<Chapter>>,

    pub active_chapter_id: Option<String>,

    // ✅ Vista del chapter activo (solo para panel/UX)
    pub active_chapter_scenes: Vec<Scene>,

    // ✅ Fuente de verdad: scenes por chapter
    pub scenes_by_chapter_id: std::collections::HashMap<String, Vec<Scene>>,

    pub expanded_novels: std::collections::HashSet<String>,
    pub expanded_chapters: std::collections::HashSet<String>,


    pub last_novels_reload: std::time::Instant,
    pub last_chapters_reload: std::time::Instant,
    pub last_scenes_reload: std::time::Instant,

    // ✅ NUEVO (PRO): tracking de "ya cargado" por entidad + gating de cargas en progreso
    pub forge_chapters_loaded_for: HashMap<String, std::time::Instant>, // novel_id -> last load time
    pub forge_scenes_loaded_for: HashMap<String, std::time::Instant>,   // chapter_id -> last load time
    pub forge_loading_in_progress: HashSet<ForgeLoadKey>,

    // ================================
    // FASE 9 (PRO): Core fetch lifecycle (gating + throttle)
    // ================================
    pub core_loading_in_progress: std::collections::HashSet<CoreLoadKey>,

    pub last_universes_reload: std::time::Instant,
    pub last_boards_reload: std::time::Instant,

    // ✅ (FASE 10): snapshots también entra al contrato único Core (throttle + gating + loaded_for)
    pub last_snapshots_reload: std::time::Instant,

    pub core_creatures_loaded_for: std::collections::HashMap<String, std::time::Instant>, // universe_id -> last load
    pub core_locations_loaded_for: std::collections::HashMap<String, std::time::Instant>, // universe_id -> last load
    pub core_timeline_loaded_for: std::collections::HashMap<String, std::time::Instant>,  // universe_id -> last load
    pub core_snapshots_loaded_for: std::collections::HashMap<String, std::time::Instant>, // universe_id -> last load
    pub pm_board_loaded_for: std::collections::HashMap<String, std::time::Instant>,       // board_id -> last load

    // ✅ NUEVO: Timestamps para debouncing de acciones
    pub last_create_novel_time: std::time::Instant,
    pub last_create_chapter_time: std::time::Instant,
    pub last_create_scene_time: std::time::Instant,


    pub active_scene_id: Option<String>,
    pub forge_content: text_editor::Content,

    pub forge_last_edit: Option<Instant>,
    pub forge_debounce_task_id: Option<u64>,

    pub forge_renaming_novel_id: Option<String>,
    pub forge_renaming_chapter_id: Option<String>,
    pub forge_renaming_scene_id: Option<String>,

    // ✅ NUEVO: Copias temporales para rename (no dependen de listas que pueden limpiarse)
    pub forge_renaming_novel_temp: Option<Novel>,
    pub forge_renaming_chapter_temp: Option<Chapter>,
    pub forge_renaming_scene_temp: Option<Scene>,
    pub last_forge_novel_click: Option<(String, Instant)>,
    pub last_forge_chapter_click: Option<(String, Instant)>,
    pub last_forge_scene_click: Option<(String, Instant)>,

    pub boards_list: Vec<Board>,
    pub new_board_name: String,
    pub pm_state: PmState,
    pub pm_data: Option<KanbanBoardData>,

    pub hovered_column: Option<PmId>,
    pub hovered_card: Option<PmId>,
    pub last_pm_click: Option<(PmId, Instant)>,

    pub pm_ids: PmIdPool,

    pub creature_editor: Option<CreatureEditor>,
    pub last_bestiary_click: Option<(usize, Instant)>,

    pub location_editor: Option<LocationEditor>,
    pub last_location_click: Option<(String, Instant)>,
    pub expanded_locations: HashSet<String>,
    pub selected_location: Option<String>,

    pub event_editor: Option<EventEditor>,
    pub era_editor: Option<EraEditor>,
    pub last_timeline_click: Option<(String, Instant)>,

    pub db_queue: VecDeque<DbAction>,
    pub db_inflight: Option<DbAction>,

    pub toasts: Vec<Toast>,
    pub toast_counter: u64,
    pub trash_entries: Vec<TrashEntry>,
    pub trash_loaded: bool,
    //search in trash
    pub trash_search_query: String,
    pub trash_selected: HashSet<String>,

    pub forge_outline_version: u32,

}

impl Default for AppState {
    fn default() -> Self {
        Self {
            route: Route::Overview,

            active_project: None,
            projects: vec![],
            is_creating_project: false,
            new_project_name: String::new(),

            universes: vec![],
            new_universe_name: String::new(),
            new_universe_desc: String::new(),

            pending_confirm: None,
            dev_panel_open: true,

            // ================================
            // FASE 13 (PRO): Observabilidad
            // ================================
            debug_overlay_open: false,
            debug_schema_version: None,
            debug_events: Vec::new(),
            debug_last_invalidation: None,
            debug_last_ignored: None,

            snapshot_name: String::new(),
            snapshots: vec![],

            integrity_issues: vec![],
            integrity_busy: false,

            loaded_creatures_universe: None,
            loaded_locations_universe: None,
            loaded_timeline_universe: None,
            loaded_snapshots_universe: None,
            loaded_forge_universe: None,

            data_dirty: false,

            forge_outline_version: 0,

            creatures: vec![],
            creatures_index: HashMap::new(),
            locations: vec![],
            locations_children_map: HashMap::new(),
            timeline_events: vec![],
            timeline_eras: vec![],

            // --- THE FORGE V2 ---
            novels: vec![],
            active_novel_id: None,
            active_novel_chapters: vec![],
            active_chapter_id: None,
            active_chapter_scenes: vec![],

            last_novels_reload: std::time::Instant::now(),
            last_chapters_reload: std::time::Instant::now(),
            last_scenes_reload: std::time::Instant::now(),
            chapters_by_novel_id: std::collections::HashMap::new(),
            scenes_by_chapter_id: std::collections::HashMap::new(),


            // ✅ NUEVO (PRO): tracking de cargas
            forge_chapters_loaded_for: std::collections::HashMap::new(),
            forge_scenes_loaded_for: std::collections::HashMap::new(),
            forge_loading_in_progress: std::collections::HashSet::new(),


            core_loading_in_progress: std::collections::HashSet::new(),

            last_universes_reload: std::time::Instant::now(),
            last_boards_reload: std::time::Instant::now(),
            last_snapshots_reload: std::time::Instant::now(),

            core_creatures_loaded_for: std::collections::HashMap::new(),
            core_locations_loaded_for: std::collections::HashMap::new(),
            core_timeline_loaded_for: std::collections::HashMap::new(),
            core_snapshots_loaded_for: std::collections::HashMap::new(),
            pm_board_loaded_for: std::collections::HashMap::new(),

            // ✅ NUEVO: Timestamps para debouncing de acciones
            last_create_novel_time: std::time::Instant::now(),
            last_create_chapter_time: std::time::Instant::now(),
            last_create_scene_time: std::time::Instant::now(),

            active_scene_id: None,
            forge_content: text_editor::Content::new(),

            forge_last_edit: None,
            forge_debounce_task_id: None,

            forge_renaming_novel_id: None,
            forge_renaming_chapter_id: None,
            forge_renaming_scene_id: None,

            // ✅ NUEVO
            forge_renaming_novel_temp: None,
            forge_renaming_chapter_temp: None,
            forge_renaming_scene_temp: None,
            last_forge_novel_click: None,
            last_forge_chapter_click: None,
            last_forge_scene_click: None,

            boards_list: vec![],
            new_board_name: String::new(),
            pm_state: PmState::Idle,
            pm_data: None,

            pm_ids: PmIdPool::default(),

            hovered_column: None,
            hovered_card: None,
            last_pm_click: None,

            creature_editor: None,
            last_bestiary_click: None,

            location_editor: None,
            last_location_click: None,
            expanded_locations: HashSet::new(),
            selected_location: None,

            event_editor: None,
            era_editor: None,
            last_timeline_click: None,

            db_queue: VecDeque::new(),
            db_inflight: None,

            toasts: vec![],
            toast_counter: 0,

            expanded_novels: std::collections::HashSet::new(),
            expanded_chapters: std::collections::HashSet::new(),

            trash_entries: Vec::new(),
            trash_loaded: false,

            trash_search_query: String::new(),
            trash_selected: HashSet::new(),
        }
    }
}

impl AppState {
    pub fn queue(&mut self, action: DbAction) {
        self.db_queue.push_back(action);
    }

    pub fn show_toast(&mut self, msg: impl Into<String>, kind: ToastKind) {
        self.show_toast_internal(msg.into(), kind);
    }

    // Función interna compartida (privada)
    fn show_toast_internal(
        &mut self,
        message: String,
        kind: ToastKind,
    ) {
        const MAX_TOASTS: usize = 10;

        // 1) Poda inmediata de toasts expirados (evita acumular basura muerta)
        let now = Instant::now();
        self.toasts.retain(|t| {
            now.saturating_duration_since(t.created_at).as_secs() < t.ttl_secs as u64
        });

        // ✅ Observabilidad: si es error, lo registramos como Error (solo si overlay abierto)
        if matches!(kind, ToastKind::Error) {
            // No hot-path: toasts no se emiten por frame. Y debug_push no hace nada si overlay está cerrado.
            self.debug_push(crate::state::DebugEventKind::Error, message.as_str());
        }

        // 2) Crear el nuevo toast
        self.toast_counter += 1;

        self.toasts.push(Toast {
            id: self.toast_counter,
            message,
            kind,
            created_at: now,
            ttl_secs: 4,
        });

        // 3) Cap duro: si se pasa, drena lo más viejo
        if self.toasts.len() > MAX_TOASTS {
            let overflow = self.toasts.len() - MAX_TOASTS;
            self.toasts.drain(0..overflow);
        }
    }

    // ================================
    // FASE 13 (PRO): Observabilidad helpers
    // ================================
    pub fn debug_push(&mut self, kind: crate::state::DebugEventKind, msg: impl Into<String>) {
        // Premium: cero costo si no está abierto
        if !self.debug_overlay_open {
            return;
        }

        const MAX_EVENTS: usize = 80;

        self.debug_events.push(crate::state::DebugEvent {
            at: Instant::now(),
            kind,
            msg: msg.into(),
        });

        if self.debug_events.len() > MAX_EVENTS {
            let overflow = self.debug_events.len() - MAX_EVENTS;
            self.debug_events.drain(0..overflow);
        }
    }

    pub fn debug_record_invalidation(
        &mut self,
        scope: impl Into<String>,
        key: impl Into<String>,
        reason: impl Into<String>,
    ) {
        if !self.debug_overlay_open {
            return;
        }

        self.debug_last_invalidation = Some(crate::state::DebugInvalidation {
            at: Instant::now(),
            scope: scope.into(),
            key: key.into(),
            reason: reason.into(),
        });

        self.debug_push(crate::state::DebugEventKind::Info, "📌 invalidation recorded");
    }

    pub fn debug_record_ignored(&mut self, reason: impl Into<String>) {
        if !self.debug_overlay_open {
            return;
        }

        self.debug_last_ignored = Some(crate::state::DebugIgnored {
            at: Instant::now(),
            reason: reason.into(),
        });

        self.debug_push(crate::state::DebugEventKind::Warn, "🫥 ignored (out-of-order / not relevant)");
    }

    // ============================================
    // FASE 9.x (PRO): Core fetch lifecycle helpers
    // ============================================

    /// Intenta comenzar un load "global" (sin scope), con throttle por Instant.
    /// - Si el key ya está en progreso => false
    /// - Si no pasó el throttle => false
    /// - Si se puede => marca last_reload, inserta key en in_progress y retorna true
    pub fn core_try_begin_global_load(
        &mut self,
        key: CoreLoadKey,
        last_reload: Instant,
        throttle_ms: u128,
    ) -> Option<Instant> {
        if self.core_loading_in_progress.contains(&key) {
            return None;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(last_reload).as_millis() as u128;

        if elapsed < throttle_ms {
            return None;
        }

        self.core_loading_in_progress.insert(key);
        Some(now)
    }

    /// Intenta comenzar un load "scoped" (por universe_id/board_id), con throttle por HashMap(scope_id -> Instant).
    /// - Si el key ya está en progreso => false
    /// - Si no pasó el throttle según loaded_for[scope_id] => false
    /// - Si se puede => inserta key en in_progress y retorna true
    ///
    /// Nota: NO actualiza loaded_for. Eso se hace cuando llega el *Fetched*.
    pub fn core_try_begin_scoped_load(
        &mut self,
        key: CoreLoadKey,
        loaded_at: Option<Instant>,
        throttle_ms: u128,
    ) -> bool {
        if self.core_loading_in_progress.contains(&key) {
            return false;
        }

        let now = Instant::now();

        let allow = loaded_at
            .map(|t| now.duration_since(t).as_millis() as u128 >= throttle_ms)
            .unwrap_or(true);

        if !allow {
            return false;
        }

        self.core_loading_in_progress.insert(key);
        true
    }

    // ============================================
    // REFACTOR A.2: Location hierarchy cache
    // ============================================

    /// Rebuild locations children map - O(n) but only when locations change
    pub fn rebuild_locations_cache(&mut self) {
        self.locations_children_map.clear();

        for loc in &self.locations {
            self.locations_children_map
                .entry(loc.parent_id.clone())
                .or_insert_with(Vec::new)
                .push(loc.id.clone());
        }
    }

    /// Get children IDs for a parent - O(1)
    pub fn get_location_children(&self, parent_id: &Option<String>) -> Vec<&String> {
        self.locations_children_map
            .get(parent_id)
            .map(|ids| ids.iter().collect())
            .unwrap_or_default()
    }
}

impl AppState {
    /// Reconstruye el índice de criaturas - O(n), pero solo cuando cambian las criaturas
    pub fn rebuild_creatures_index(&mut self) {
        self.creatures_index.clear();

        for (idx, creature) in self.creatures.iter().enumerate() {
            self.creatures_index.insert(creature.id.clone(), idx);
        }
    }

    /// Busca criatura por ID - O(1) en vez de O(n)
    pub fn find_creature_by_id(&self, id: &str) -> Option<&Creature> {
        self.creatures_index
            .get(id)
            .and_then(|&idx| self.creatures.get(idx))
    }

}