use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn seed(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO boards (id, name, kind) VALUES ('board-main', 'Development Roadmap', 'kanban')")
        .execute(pool)
        .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO board_columns (id, board_id, name, position) VALUES \
         ('col-hold', 'board-main', 'On-Hold', 0), \
         ('col-todo', 'board-main', 'To Do', 1), \
         ('col-progress', 'board-main', 'In Progress', 2), \
         ('col-done', 'board-main', 'Done', 3)",
    )
        .execute(pool)
        .await?;

    let (k_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM cards WHERE column_id IN (SELECT id FROM board_columns WHERE board_id = 'board-main')",
    )
        .fetch_one(pool)
        .await?;

    if k_count >= 6 {
        return Ok(());
    }

    let tasks = vec![
        ("Define Magic System", "col-progress", "Hard rules"),
        ("Map Northern Continent", "col-todo", "Mountains and rivers"),
        ("Create Pantheon", "col-todo", "12 gods"),
        ("History of Empire", "col-done", "First emperor details"),
        ("Design Economy", "col-todo", "Gold vs Credits"),
        ("Sketch Main Races", "col-progress", "Elves variation"),
    ];

    for (title, col_id, desc) in tasks {
        sqlx::query("INSERT INTO cards (id, column_id, title, description, position) VALUES (?, ?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(col_id)
            .bind(title)
            .bind(desc)
            .bind(100.0)
            .execute(pool)
            .await?;
    }

    Ok(())
}
