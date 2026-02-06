use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use crate::model::Project;
use uuid::Uuid;
use chrono::Utc;

pub struct ProjectManager;

impl ProjectManager {
    fn get_config_dir() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "TitanArchitects", "TAS") {
            proj_dirs.config_dir().to_path_buf()
        } else {
            PathBuf::from("config")
        }
    }

    fn get_data_dir() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "TitanArchitects", "TAS") {
            proj_dirs.data_dir().to_path_buf()
        } else {
            PathBuf::from("data")
        }
    }

    fn get_manifest_path() -> PathBuf {
        Self::get_config_dir().join("projects.json")
    }

    pub fn load_projects() -> Vec<Project> {
        let path = Self::get_manifest_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(projects) = serde_json::from_str::<Vec<Project>>(&content) {
                    let mut sorted = projects;
                    sorted.sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
                    return sorted;
                }
            }
        }
        vec![]
    }

    fn save_projects(projects: &[Project]) -> Result<(), String> {
        let path = Self::get_manifest_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(projects).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn create_project(name: String) -> Result<Project, String> {
        let data_dir = Self::get_data_dir();
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

        let safe_name: String = name.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();

        // Por defecto creamos .universe (para mantener compatibilidad total)
        let filename = format!("{}.universe", safe_name);
        let db_path = data_dir.join(&filename);

        // Creamos el struct TAL CUAL LO TIENES (sin campos nuevos)
        let project = Project {
            id: Uuid::new_v4().to_string(),
            name,
            path: db_path.to_string_lossy().to_string(),
            last_opened: Utc::now(),
            created_at: Utc::now(),
        };

        let mut projects = Self::load_projects();
        projects.insert(0, project.clone());
        Self::save_projects(&projects)?;

        Ok(project)
    }

    pub fn update_last_opened(id: &str) {
        let mut projects = Self::load_projects();
        if let Some(p) = projects.iter_mut().find(|p| p.id == id) {
            p.last_opened = Utc::now();
        }
        let _ = Self::save_projects(&projects);
    }

    pub fn delete_project(id: &str) -> Result<(), String> {
        let mut projects = Self::load_projects();

        if let Some(pos) = projects.iter().position(|p| p.id == id) {
            let p = &projects[pos];
            let path = PathBuf::from(&p.path);

            if path.exists() {
                if let Err(e) = fs::remove_file(path) {
                    return Err(format!("Could not delete file: {}", e));
                }
            }

            projects.remove(pos);
            Self::save_projects(&projects)?;
            Ok(())
        } else {
            Err("Project not found in manifest".to_string())
        }
    }
}