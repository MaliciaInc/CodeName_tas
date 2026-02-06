use iced::{event, mouse, Element, Event, Size, Subscription, Task, Theme};

use crate::app::{AppState, Message, APP_ACRONYM, APP_NAME};
use crate::db::Database;

use std::time::{Duration, Instant};

pub fn run() -> iced::Result {
    let _ = crate::logger::init();

    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .window_size(Size::new(1600.0, 950.0))
        .run()
}

struct App {
    state: AppState,
    db: Option<Database>,

    // 🔥 FIX: “boost” corto de redraw cuando cambie el outline.
    // Iced 0.14 (reactive) puede decidir no repintar texto en Windows/wgpu
    // aunque el view() ya tenga el valor nuevo.
    last_forge_outline_version: u32,
    redraw_boost_until: Instant,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        crate::logger::info("App starting in Workspaces Mode...");

        let mut app = Self {
            state: AppState::default(),
            db: None,

            last_forge_outline_version: 0,
            redraw_boost_until: Instant::now(),
        };

        // Mantengo tu comportamiento original (según tu repo): arrancar en Workspaces
        app.state.route = crate::app::Route::Workspaces;

        // Capturamos el valor inicial para no disparar boost de gratis
        app.last_forge_outline_version = app.state.forge_outline_version;

        let load_projs = Task::perform(
            async { crate::project_manager::ProjectManager::load_projects() },
            Message::ProjectsLoaded,
        );

        (app, load_projs)
    }

    fn title(&self) -> String {
        if let Some(p) = &self.state.active_project {
            format!("{} - {} ({})", p.name, APP_NAME, APP_ACRONYM)
        } else {
            format!("{} ({})", APP_NAME, APP_ACRONYM)
        }
    }

    fn theme(&self) -> Theme {
        crate::app::app_theme(&self.state)
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = Vec::new();

        // 1) Dragging tracking (PM)
        if let crate::app::PmState::Dragging { .. } = self.state.pm_state {
            subs.push(event::listen_with(|event, _status, _window| match event {
                Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    Some(Message::GlobalEvent(Event::Mouse(
                        mouse::Event::CursorMoved { position },
                    )))
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    Some(Message::GlobalEvent(Event::Mouse(
                        mouse::Event::ButtonReleased(mouse::Button::Left),
                    )))
                }
                _ => None,
            }));
        }

        // 2) Toast TTL tick (lo dejás como estaba, 1 Hz)
        if !self.state.toasts.is_empty() {
            subs.push(iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick));
        }

        // 3) 🔥 Redraw boost tick (solo cuando se necesita, 60Hz aprox)
        // Esto no “hace trabajo”; solo hace que el runtime procese frames.
        if Instant::now() < self.redraw_boost_until {
            subs.push(iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick));
        }

        Subscription::batch(subs)
    }

    fn view(&self) -> Element<'_, Message> {
        crate::app::view(&self.state)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        // ✅ Atajo de rendimiento:
        // Si este Tick fue solo para “redraw boost” y no hay toasts,
        // no corremos controllers ni lógica pesada.
        if matches!(message, Message::Tick)
            && self.state.toasts.is_empty()
            && Instant::now() < self.redraw_boost_until
        {
            return Task::none();
        }

        let mut tasks: Vec<Task<Message>> = Vec::new();

        // 1) Workspace lifecycle / side-effects
        if let Some(mut t) = crate::controllers::workspace_controller::try_handle(
            &mut self.state,
            &mut self.db,
            &message,
        ) {
            tasks.append(&mut t);
        }

        // 2) Requests de carga (The Forge)
        match &message {
            Message::ForgeRequestLoadNovels => {
                if let Some(db) = self.db.as_ref() {
                    tasks.extend(crate::controllers::forge_data_controller::load_novels_if_needed(
                        &mut self.state,
                        db,
                    ));
                }
            }
            Message::ForgeRequestLoadChapters(novel_id) => {
                if let Some(db) = self.db.as_ref() {
                    tasks.extend(crate::controllers::forge_data_controller::load_chapters_if_needed(
                        &mut self.state,
                        db,
                        novel_id.clone(),
                    ));
                }
            }
            Message::ForgeRequestLoadScenes(chapter_id) => {
                if let Some(db) = self.db.as_ref() {
                    tasks.extend(crate::controllers::forge_data_controller::load_scenes_if_needed(
                        &mut self.state,
                        db,
                        chapter_id.clone(),
                    ));
                }
            }
            _ => {}
        }

        // 3) ActionDone side effects
        if let Message::ActionDone(result) = &message {
            crate::controllers::action_done_controller::handle_action_done(&mut self.state, result);
        }

        // 4) Handle fetch results (delegación a navigation_controller)
        match &message {
            Message::UniversesFetched(result) => {
                if let Ok(universes) = result {
                    crate::logger::info(&format!("✅ Loaded {} universes from DB", universes.len()));
                    self.state.universes = universes.clone();
                    self.state.data_dirty = false;
                } else if let Err(e) = result {
                    crate::logger::error(&format!("❌ Failed to fetch universes: {}", e));
                }
            }

            Message::BoardsFetched(result) => {
                if let Ok(boards) = result {
                    crate::logger::info(&format!("✅ Loaded {} boards from DB", boards.len()));
                    self.state.boards_list = boards.clone();
                } else if let Err(e) = result {
                    crate::logger::error(&format!("❌ Failed to fetch boards: {}", e));
                }
            }

            Message::NovelsFetched(result) => {
                crate::controllers::navigation_controller::handle_novels_fetched(
                    &mut self.state,
                    result.clone(),
                );
            }

            Message::ChaptersFetched(result) => {
                crate::controllers::navigation_controller::handle_chapters_fetched(
                    &mut self.state,
                    result.clone(),
                );
            }

            Message::ForgeScenesFetched { chapter_id, result } => {
                crate::controllers::navigation_controller::handle_forge_scenes_fetched(
                    &mut self.state,
                    chapter_id.clone(),
                    result.clone(),
                );
            }

            Message::ScenesFetched => {
                // Unit variant obsoleto - no hace nada
            }

            _ => {}
        }

        // 5) Messages controller
        let mut msg_tasks =
            crate::controllers::messages_controller::update(&mut self.state, message.clone());
        tasks.append(&mut msg_tasks);

        // 6) Global input translation
        if let Some(extra) =
            crate::controllers::input_controller::translate_global_event(&self.state, &message)
        {
            let mut extra_tasks =
                crate::controllers::messages_controller::update(&mut self.state, extra);
            tasks.append(&mut extra_tasks);
        }

        // 7) Post-event scheduler
        if self.state.active_project.is_some() {
            tasks.extend(crate::controllers::post_event_tasks_controller::post_event_tasks(
                &mut self.state,
                &self.db,
            ));
        }

        // 🔥 FIX REAL: Detectar cambios del outline y abrir ventana de “boost” de repintado.
        // Esto NO toca tu data ni DB: solo asegura que el runtime procese frames.
        let curr: u32 = self.state.forge_outline_version;
        if curr != self.last_forge_outline_version {
            self.last_forge_outline_version = curr;
            self.redraw_boost_until = Instant::now() + Duration::from_millis(200);
        }

        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }
}
