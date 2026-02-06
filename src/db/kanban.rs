// ========================================
// kanban.rs - Gestión de boards Kanban
// ========================================
// Este módulo maneja boards, columnas y cards del sistema Kanban

use crate::model::{Board, Card, KanbanBoardData};
use crate::db::Database;

impl Database {
    pub async fn get_all_boards(&self) -> Result<Vec<Board>, sqlx::Error> {
        crate::logger::info("🔍 DB: Querying boards...");

        let result = sqlx::query_as::<_, Board>(
            "SELECT id, name, kind
                        FROM boards
                        ORDER BY name ASC"
        )
            .fetch_all(&self.pool)
            .await?;

        crate::logger::info(&format!("✅ DB: Found {} boards", result.len()));

        for b in &result {
            crate::logger::info(&format!("  - {} ({})", b.name, b.id));
        }

        Ok(result)
    }

    pub async fn create_board(&self, id: String, name: String) -> Result<(), Box<dyn std::error::Error>> {
        // ✅ Guard de capability
        self.require_capability("boards").await?;

        let mut tx = self.pool.begin().await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // 1) Insert board
        sqlx::query(
            r#"
                    INSERT INTO boards (id, name)
                    VALUES (?, ?)
                    "#,
        )
            .bind(&id)
            .bind(&name)
            .execute(&mut *tx)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // 2) Insert default columns
        let defaults: [(&str, i64); 4] = [
            ("On Hold", 0),
            ("To Do", 1),
            ("In Progress", 2),
            ("Done", 3),
        ];

        for (col_name, pos) in defaults {
            let col_id = format!("col-{}-{}", &id, col_name.to_lowercase().replace(' ', "-"));

            sqlx::query(
                r#"
                    INSERT INTO board_columns (id, board_id, name, position)
                    VALUES (?, ?, ?, ?)
                    "#,
            )
                .bind(col_id)
                .bind(&id)
                .bind(col_name)
                .bind(pos)
                .execute(&mut *tx)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        }

        tx.commit().await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        Ok(())
    }

    pub async fn delete_board(&self, board_id: String) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Defensive deletes
        sqlx::query("DELETE FROM cards WHERE column_id IN (SELECT id FROM board_columns WHERE board_id = ?)")
            .bind(&board_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM board_columns WHERE board_id = ?")
            .bind(&board_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM boards WHERE id = ?")
            .bind(&board_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_kanban_data(&self, board_id: String) -> Result<KanbanBoardData, sqlx::Error> {
        let board: Board = sqlx::query_as("SELECT id, name, kind FROM boards WHERE id = ?")
            .bind(&board_id)
            .fetch_one(&self.pool)
            .await?;

        let cols: Vec<(String, String, i32)> = sqlx::query_as("SELECT id, name, position FROM board_columns WHERE board_id = ? ORDER BY position ASC")
            .bind(&board_id)
            .fetch_all(&self.pool)
            .await?;

        let mut out: Vec<(crate::model::BoardColumn, Vec<Card>)> = Vec::new();

        for (cid, cname, pos) in cols {
            let cards: Vec<Card> = sqlx::query_as("SELECT id, column_id, title, description, position, priority FROM cards WHERE column_id = ? ORDER BY position ASC")
                .bind(&cid)
                .fetch_all(&self.pool)
                .await?;

            out.push((
                crate::model::BoardColumn {
                    id: cid,
                    board_id: board_id.clone(),
                    name: cname,
                    position: pos,
                },
                cards,
            ));
        }

        Ok(KanbanBoardData { board, columns: out })
    }

    pub async fn upsert_card(&self, c: Card) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO cards (id, column_id, title, description, position, priority, updated_at) VALUES (?, ?, ?, ?, ?, ?, unixepoch()) ON CONFLICT(id) DO UPDATE SET column_id=excluded.column_id, title=excluded.title, description=excluded.description, position=excluded.position, priority=excluded.priority, updated_at=unixepoch()")
            .bind(c.id).bind(c.column_id).bind(c.title).bind(c.description).bind(c.position).bind(c.priority)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn move_card(&self, card_id: String, column_id: String, pos: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE cards SET column_id = ?, position = ? WHERE id = ?")
            .bind(column_id)
            .bind(pos)
            .bind(card_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_card(&self, card_id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM cards WHERE id = ?").bind(card_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn rebalance_column(&self, column_id: String) -> Result<(), sqlx::Error> {
        let cards: Vec<(String,)> = sqlx::query_as("SELECT id FROM cards WHERE column_id = ? ORDER BY position ASC").bind(&column_id).fetch_all(&self.pool).await?;
        let mut tx = self.pool.begin().await?;
        for (i, (id,)) in cards.into_iter().enumerate() {
            let new_pos = (i as f64 + 1.0) * 1000.0;
            sqlx::query("UPDATE cards SET position = ? WHERE id = ?").bind(new_pos).bind(id).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}