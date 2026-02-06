use iced::widget::text_editor;
use crate::app::Route;
use crate::model::{Creature, Universe, Card, KanbanBoardData, Board, Location, TimelineEvent, TimelineEra, Project, UniverseSnapshot, Novel, Chapter, Scene, TrashEntry};
use crate::state::DemoResetScope;

#[derive(Debug, Clone)]
pub enum PmMessage {
    BoardNameChanged(String), CreateBoard, DeleteBoard(String), OpenBoard(String),
    BoardLoaded(KanbanBoardData), DragStart(String), ColumnHovered(String), CardHovered(String),
    OpenCreate(String), OpenGlobalCreate, OpenEdit(Card), TitleChanged(String),
    DescChanged(text_editor::Action), PriorityChanged(String), Save, Delete, Cancel,
}
#[derive(Debug, Clone)]
pub enum BestiaryMessage {
    Open(String), CardClicked(usize), EditorOpenCreate, EditorCancel, EditorSave,
    NameChanged(String), KindChanged(String), HabitatChanged(String),
    DescriptionChanged(text_editor::Action), DangerChanged(String), LocationChanged(Option<Location>),
    Delete(String), Archive(String), Restore(String),
}

#[derive(Debug, Clone)]
pub enum UniverseMessage {
    NameChanged(String), DescChanged(String), Create, Delete(String), Open(String),
    InjectDemoData(String),
    ResetDemoPrompt(String, DemoResetScope),
    ToggleDeveloperPanel,
    ToggleDebugOverlay,
    SnapshotNameChanged(String),
    SnapshotCreate(String),
    SnapshotRefresh(String),
    SnapshotRestore(String),
    SnapshotDelete(String),
    ValidateUniverse(String),
}

#[derive(Debug, Clone)]
pub enum LocationsMessage {
    Open(String), EditorOpenCreate(Option<String>), CardClicked(String), EditorCancel, EditorSave,
    Delete(String), NameChanged(String), KindChanged(String), DescriptionChanged(text_editor::Action),
    ToggleExpand(String), Select(String), CardDoubleClicked(String),
}

#[derive(Debug, Clone)]
pub enum TimelineMessage {
    Open(String),
    EditorOpenCreateEvent(Option<i64>), EditorOpenCreateEra,
    EditEvent(String), EditEra(String),
    CardClicked(String), EraBannerClicked(String),
    EditorCancel, EditorSaveEvent, EditorSaveEra,
    DeleteEvent(String), DeleteEra(String),
    TitleChanged(String), YearChanged(String), DisplayDateChanged(String), ImportanceChanged(String),
    KindChanged(String), ColorChanged(String), LocationChanged(Option<Location>), DescriptionChanged(text_editor::Action),
    EraNameChanged(String), EraStartChanged(String), EraEndChanged(String), EraColorChanged(String), EraDescChanged(text_editor::Action),
}

#[derive(Debug, Clone)]
pub enum WorkspaceMessage {
    CreateStart, CreateCancel, NameChanged(String), CreateConfirm,
    Open(String), CloseProject,
    Delete(String),
}

#[derive(Debug, Clone)]
pub enum TheForgeMessage {
    // --- NAVIGATION ---
    Open(Option<String>),        // universe_id opcional
    UniverseChanged(String),

    // --- NOVEL ACTIONS ---
    CreateNovel,
    DeleteNovel(String),
    SelectNovel(String),
    NovelTitleChanged(String),

    // --- CHAPTER ACTIONS ---
    CreateChapter(String),       // novel_id
    DeleteChapter(String),       // chapter_id
    SelectChapter(String),
    ChapterTitleChanged(String),

    // --- SCENE ACTIONS ---
    CreateScene(String),         // chapter_id
    DeleteScene(String),         // scene_id
    SelectScene(String),
    SceneTitleChanged(String),
    SceneBodyChanged(text_editor::Action),

    // --- AUTO-SAVE ---
    SaveCurrentScene,
    DebounceComplete(u64),

    // --- INLINE RENAME ---
    EndRename,

    // NUEVO: EXPAND/COLLAPSE
    ToggleNovel(String),      // novel_id - expande/colapsa chapters
    ToggleChapter(String),    // chapter_id - expande/colapsa scenes

    // NUEVO: DRAG & DROP
    ChapterDragged(String, usize),  // chapter_id, new_position
    SceneDragged(String, usize),    // scene_id, new_position
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(Route), MouseMoved(iced::Point), MouseReleased,
    Tick,
    ToastDismiss(u64),

    // ✅ NUEVO (FASE 2): intenciones de carga (sin DB en state)
    ForgeRequestLoadNovels,
    ForgeRequestLoadChapters(String), // novel_id
    ForgeRequestLoadScenes(String),   // chapter_id

    Pm(PmMessage), Bestiary(BestiaryMessage), Universe(UniverseMessage), Locations(LocationsMessage),
    Timeline(TimelineMessage), Workspace(WorkspaceMessage), TheForge(TheForgeMessage),

    BoardsFetched(Result<Vec<Board>, String>),

    UniversesFetched(Result<Vec<Universe>, String>),

    // ✅ FASE 9/10: identidad + resultado (evita out-of-order y libera gating siempre)
    CreaturesFetched {
        universe_id: String,
        result: Result<Vec<Creature>, String>,
    },

    // ✅ FASE 9/10: identidad + resultado (evita out-of-order y libera gating siempre)
    PmBoardFetched {
        board_id: String,
        result: Result<KanbanBoardData, String>,
    },
    LocationsFetched {
        universe_id: String,
        result: Result<Vec<Location>, String>,
    },

    TimelineFetched {
        universe_id: String,
        result: Result<(Vec<TimelineEvent>, Vec<TimelineEra>), String>,
    },

    NovelsFetched(Result<Vec<Novel>, String>),
    ChaptersFetched(Result<Vec<Chapter>, String>),
    ScenesFetched,

// ✅ NUEVO (FASE 11 hardening): chapters con identidad (evita responses out-of-order)
    ForgeChaptersFetched {
        novel_id: String,
        result: Result<Vec<Chapter>, String>,
    },

    ForgeScenesFetched {
        chapter_id: String,
        result: Result<Vec<Scene>, String>,
    },

    SnapshotsFetched {
        universe_id: String,
        result: Result<Vec<UniverseSnapshot>, String>,
    },
    SchemaVersionFetched(Result<i64, String>),
    IntegrityFetched(Result<Vec<String>, String>),

    ProjectsLoaded(Vec<Project>),
    ProjectCreated(Result<Project, String>),
    DbLoaded(Result<crate::db::Database, String>),

    ActionDone(Result<(), String>),

    GlobalEvent(iced::Event),

    BackToUniverses, BackToUniverse(String), OpenTimeline(String), GoToLocation(String, String),
    ConfirmDelete,
    CancelConfirm,

    TrashFetched(Result<Vec<TrashEntry>, String>),
    RestoreFromTrash(String),
    PermanentDelete(String),
    EmptyTrash,
    TrashSearchChanged(String),
    ToggleTrashSelection(String),      // Toggle un item
    SelectAllTrash,                     // Seleccionar todos
    DeselectAllTrash,                   // Deseleccionar todos
    RestoreSelected,                    // Restaurar seleccionados
    DeleteSelectedForever,
    CleanupOldTrash,
}