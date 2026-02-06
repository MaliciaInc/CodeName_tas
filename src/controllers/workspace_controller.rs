use iced::Task;

use crate::app::{AppState, Message, WorkspaceMessage};
use crate::db::Database;
use crate::model::ProjectKind;
use crate::project_manager::ProjectManager;
use crate::state::ToastKind;

/// Workspace controller: handles all workspace concerns (Open/Create/Delete/Close + simple UI state for workspace).
///
/// This keeps ui_controller as mostly wiring/plumbing while still allowing it to own `db: Option<Database>`.
pub fn try_handle(
    state: &mut AppState,
    db: &mut Option<Database>,
    message: &Message,
) -> Option<Vec<Task<Message>>> {
    let mut tasks: Vec<Task<Message>> = Vec::new();

    match message {
        // --- Workspace UI state (pure state mutations) ---
        Message::Workspace(WorkspaceMessage::CreateStart) => {
            state.is_creating_project = true;
            state.new_project_name.clear();
            return Some(tasks);
        }

        Message::Workspace(WorkspaceMessage::CreateCancel) => {
            state.is_creating_project = false;
            return Some(tasks);
        }

        Message::Workspace(WorkspaceMessage::NameChanged(v)) => {
            state.new_project_name = v.clone();
            return Some(tasks);
        }

        // --- Workspace flows (side effects) ---
        Message::Workspace(WorkspaceMessage::Open(id)) => {
            if let Some(proj) = state.projects.iter().find(|p| p.id == *id).cloned() {
                state.active_project = Some(proj.clone());

                // Route by project kind (canonical: Project::get_kind()).
                match proj.get_kind() {
                    ProjectKind::Universe => {
                        state.route = crate::app::Route::Overview;
                    }
                    ProjectKind::Board => {
                        state.route = crate::app::Route::PmList;
                    }
                    ProjectKind::Novel => {
                        state.route = crate::app::Route::Forge;

                        // Keep Forge state consistent on enter.
                        state.novels.clear();
                        state.active_novel_id = None;
                        state.active_novel_chapters.clear();
                        state.active_chapter_id = None;
                        state.active_chapter_scenes.clear();
                        state.active_scene_id = None;
                        state.forge_content = iced::widget::text_editor::Content::new();
                    }
                }

                // Persist last_opened (best-effort).
                let pid = proj.id.clone();
                std::thread::spawn(move || ProjectManager::update_last_opened(&pid));

                // Connect DB.
                let path = std::path::PathBuf::from(proj.path.clone());
                tasks.push(Task::perform(
                    async move { Database::connect(path).await.map_err(|e| e.to_string()) },
                    Message::DbLoaded,
                ));
            }

            return Some(tasks);
        }

        Message::Workspace(WorkspaceMessage::CreateConfirm) => {
            let name = state.new_project_name.clone();
            tasks.push(Task::perform(
                async move { ProjectManager::create_project(name) },
                Message::ProjectCreated,
            ));
            return Some(tasks);
        }

        Message::Workspace(WorkspaceMessage::Delete(id)) => {
            let pid = id.clone();
            tasks.push(Task::perform(
                async move {
                    let _ = ProjectManager::delete_project(&pid);
                    ProjectManager::load_projects()
                },
                Message::ProjectsLoaded,
            ));
            return Some(tasks);
        }
        Message::Workspace(WorkspaceMessage::CloseProject) => {
            // Drop current DB/session and return to launcher mode.
            *db = None;
            *state = crate::state::AppState::default();
            state.route = crate::app::Route::Workspaces;

            tasks.push(Task::perform(
                async { ProjectManager::load_projects() },
                Message::ProjectsLoaded,
            ));
            return Some(tasks);
        }

        // --- Results in the workspace lifecycle ---
        Message::DbLoaded(result) => {
            match result {
                Ok(db_loaded) => {
                    crate::logger::info("Database connected.");
                    *db = Some(db_loaded.clone());

                    // IMPORTANT:
                    // No eager prefetch here.
                    // post_event_tasks_controller decides what to fetch based on route + flags.
                }
                Err(e) => {
                    crate::logger::error(&format!("Failed to connect to DB: {}", e));
                    state.show_toast("Failed to open database", ToastKind::Error);
                }
            }
            return Some(tasks);
        }

        Message::ProjectCreated(result) => {
            match result {
                Ok(_) => {
                    state.is_creating_project = false;

                    tasks.push(Task::perform(
                        async { ProjectManager::load_projects() },
                        Message::ProjectsLoaded,
                    ));

                    state.show_toast("Workspace created", ToastKind::Success);
                }
                Err(e) => {
                    crate::logger::error(&format!("Project create failed: {}", e));
                    state.show_toast(e.clone(), ToastKind::Error);
                }
            }
            return Some(tasks);
        }

        Message::ProjectsLoaded(projects) => {
            crate::logger::info(&format!("📂 Loaded {} projects", projects.len()));

            // ✅ Verificar si cambió la lista
            let projects_changed = state.projects.len() != projects.len() ||
                state.projects.iter().zip(projects.iter())
                    .any(|(a, b)| a.id != b.id);

            if projects_changed {
                crate::logger::info("   📝 Projects list changed, updating...");
                state.projects = projects.clone();
            } else {
                crate::logger::info("   ✅ Projects list unchanged");
                state.projects = projects.clone();
            }

            // ❌ AUTO-OPEN DESHABILITADO - Para quedarse en el launcher
            // Si quieres que auto-abra el último proyecto, descomenta este bloque:
            /*
            if state.active_project.is_none() && !projects.is_empty() {
                let most_recent = &projects[0];
                crate::logger::info(&format!("🚀 Auto-opening most recent project: {}",
                                             most_recent.name));

                let open_msg = Message::Workspace(WorkspaceMessage::Open(most_recent.id.clone()));
                tasks.push(Task::done(open_msg));
            } else if state.active_project.is_some() {
                crate::logger::info(&format!("   ✅ Project already active: {}",
                                             state.active_project.as_ref().unwrap().name));
            }
            */

            // ✅ Solo loguear el estado
            match state.active_project.as_ref() {
                None => {
                    crate::logger::info("   ℹ️ No active project - staying in launcher");
                }
                Some(project) => {
                    crate::logger::info(&format!("   ✅ Project already active: {}", project.name));
                }
            }

            return Some(tasks);
        }

        _ => None,
    }
}
