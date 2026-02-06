use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn seed(pool: &SqlitePool, universe_id: &str) -> Result<(), sqlx::Error> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM locations WHERE universe_id = ?")
            .bind(universe_id)
            .fetch_one(pool)
            .await?;

    if count >= 7 {
        return Ok(());
    }

    let locs = vec![
        ("Silver Spire", "City"),
        ("Dread Marsh", "Region"),
        ("Sunken Temple", "Landmark"),
        ("Iron Peaks", "Region"),
        ("The Void Gate", "Landmark"),
        ("Emerald Forest", "Region"),
        ("Kings Landing", "City"),
    ];

    for (name, kind) in locs {
        sqlx::query("INSERT INTO locations (id, universe_id, name, kind) VALUES (?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(universe_id)
            .bind(name)
            .bind(kind)
            .execute(pool)
            .await?;
    }

    Ok(())
}
