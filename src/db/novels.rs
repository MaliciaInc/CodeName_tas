// ========================================
// novels.rs - Gestión de novelas, capítulos y escenas
// ========================================
// Este módulo maneja el sistema completo de escritura: novels, chapters, scenes

use crate::model::{Novel, Chapter, Scene};
use crate::db::Database;

impl Database {
    // --- NOVELS ---

    pub async fn get_novels(&self, universe_id: Option<String>) -> Result<Vec<Novel>, sqlx::Error> {
        crate::logger::info(&format!("🔍 DB: Querying novels for universe: {:?}", universe_id));

        let result = match universe_id {
            Some(uid) => {
                sqlx::query_as::<_, Novel>(
                    "SELECT * FROM novels WHERE universe_id = ? ORDER BY created_at DESC"
                )
                    .bind(&uid)
                    .fetch_all(&self.pool)
                    .await?
            }
            None => {
                sqlx::query_as::<_, Novel>(
                    "SELECT * FROM novels WHERE universe_id IS NULL ORDER BY created_at DESC"
                )
                    .fetch_all(&self.pool)
                    .await?
            }
        };

        crate::logger::info(&format!("✅ DB: Found {} novels", result.len()));

        for n in &result {
            crate::logger::info(&format!("  - {} ({})", n.title, n.id));
        }

        Ok(result)
    }

    pub async fn create_novel_with_id(
        &self,
        novel_id: String,
        universe_id: Option<String>,
        title: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // ✅ Guard de capability
        self.require_capability("forge").await?;

        if title.trim().is_empty() {
            return Err("Title cannot be empty".into());
        }

        sqlx::query("INSERT INTO novels (id, universe_id, title) VALUES (?, ?, ?)")
            .bind(&novel_id)
            .bind(universe_id)
            .bind(title)
            .execute(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        Ok(())
    }

    pub async fn update_novel(&self, n: Novel) -> Result<(), sqlx::Error> {
        if n.title.trim().is_empty() {
            return Err(sqlx::Error::Protocol("Title cannot be empty".into()));
        }

        sqlx::query(
            "UPDATE novels
                    SET title = ?, synopsis = ?, status = ?, updated_at = unixepoch()
                    WHERE id = ?"
        )
            .bind(n.title)
            .bind(n.synopsis)
            .bind(n.status)
            .bind(n.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_novel(&self, novel_id: String) -> Result<(), sqlx::Error> {
        // CASCADE DELETE: Esto borrará automáticamente chapters y scenes
        crate::logger::info(&format!("Deleting novel {} (cascade to chapters and scenes)", novel_id));

        sqlx::query("DELETE FROM novels WHERE id = ?")
            .bind(novel_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // --- CHAPTERS ---

    pub async fn get_chapters(&self, novel_id: String) -> Result<Vec<Chapter>, sqlx::Error> {
        sqlx::query_as::<_, Chapter>(
            "SELECT id, novel_id, title, position, synopsis, status, created_at, updated_at
                    FROM chapters
                    WHERE novel_id = ?
                    ORDER BY position ASC"
        )
            .bind(novel_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn create_chapter_with_id(
        &self,
        chapter_id: String,
        novel_id: String,
        title: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // ✅ Guard de capability
        self.require_capability("forge").await?;

        if title.trim().is_empty() {
            return Err("Title cannot be empty".into());
        }

        // Verificar que el novel existe
        let (exists,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM novels WHERE id = ?")
            .bind(&novel_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        if exists == 0 {
            return Err(format!("Novel {} not found", novel_id).into());
        }

        // Obtener la siguiente posición
        let (max_pos,): (Option<i64>,) = sqlx::query_as("SELECT MAX(position) FROM chapters WHERE novel_id = ?")
            .bind(&novel_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let pos = max_pos.unwrap_or(-1) + 1;

        sqlx::query("INSERT INTO chapters (id, novel_id, title, position) VALUES (?, ?, ?, ?)")
            .bind(&chapter_id)
            .bind(novel_id)
            .bind(title)
            .bind(pos)
            .execute(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        Ok(())
    }

    pub async fn update_chapter(&self, c: Chapter) -> Result<(), sqlx::Error> {
        if c.title.trim().is_empty() {
            return Err(sqlx::Error::Protocol("Title cannot be empty".into()));
        }

        sqlx::query(
            "UPDATE chapters
                    SET title = ?, synopsis = ?, status = ?, updated_at = unixepoch()
                    WHERE id = ?"
        )
            .bind(c.title)
            .bind(c.synopsis)
            .bind(c.status)
            .bind(c.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_chapter(&self, chapter_id: String) -> Result<(), sqlx::Error> {
        // CASCADE DELETE: Esto borrará automáticamente todas las scenes del chapter
        crate::logger::info(&format!("Deleting chapter {} (cascade to scenes)", chapter_id));

        sqlx::query("DELETE FROM chapters WHERE id = ?")
            .bind(chapter_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn reorder_chapter(&self, chapter_id: String, new_position: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE chapters SET position = ?, updated_at = unixepoch() WHERE id = ?")
            .bind(new_position)
            .bind(chapter_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // --- SCENES ---

    pub async fn get_scenes(&self, chapter_id: String) -> Result<Vec<Scene>, sqlx::Error> {
        sqlx::query_as::<_, Scene>(
            "SELECT id, chapter_id, title, body, position, status, word_count, created_at, updated_at
                    FROM scenes
                    WHERE chapter_id = ?
                    ORDER BY position ASC"
        )
            .bind(chapter_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn create_scene_with_id(
        &self,
        scene_id: String,
        chapter_id: String,
        title: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // ✅ Guard de capability
        self.require_capability("forge").await?;

        if title.trim().is_empty() {
            return Err("Title cannot be empty".into());
        }

        // Verificar que el chapter existe
        let (exists,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chapters WHERE id = ?")
            .bind(&chapter_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        if exists == 0 {
            return Err(format!("Chapter {} not found", chapter_id).into());
        }

        // Obtener la siguiente posición
        let (max_pos,): (Option<i64>,) = sqlx::query_as("SELECT MAX(position) FROM scenes WHERE chapter_id = ?")
            .bind(&chapter_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let pos = max_pos.unwrap_or(-1) + 1;

        sqlx::query("INSERT INTO scenes (id, chapter_id, title, position) VALUES (?, ?, ?, ?)")
            .bind(&scene_id)
            .bind(chapter_id)
            .bind(title)
            .bind(pos)
            .execute(&self.pool)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        Ok(())
    }

    pub async fn update_scene(&self, s: Scene) -> Result<(), sqlx::Error> {
        if s.title.trim().is_empty() {
            return Err(sqlx::Error::Protocol("Title cannot be empty".into()));
        }

        // Recalcular word_count desde el body (DB como última fuente de verdad)
        let computed_word_count: i64 = s.body.split_whitespace().count() as i64;

        crate::logger::info(&format!(
            "Updating scene {} (words: {})",
            s.id, computed_word_count
        ));

        sqlx::query(
            "UPDATE scenes
                        SET title = ?, body = ?, status = ?, word_count = ?, updated_at = unixepoch()
                        WHERE id = ?"
        )
            .bind(s.title)
            .bind(s.body)
            .bind(s.status)
            .bind(computed_word_count)
            .bind(s.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_scene(&self, scene_id: String) -> Result<(), sqlx::Error> {
        crate::logger::info(&format!("Deleting scene {}", scene_id));

        sqlx::query("DELETE FROM scenes WHERE id = ?")
            .bind(scene_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn reorder_scene(&self, scene_id: String, new_position: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scenes SET position = ?, updated_at = unixepoch() WHERE id = ?")
            .bind(new_position)
            .bind(scene_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}