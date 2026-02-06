use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn seed(pool: &SqlitePool, universe_id: &str) -> Result<(), sqlx::Error> {
    // Ensure 5 eras exist with stable IDs
    let eras = vec![
        ("era-myth", "Age of Myth", -10000, Some(-5000), "#8B5CF6"),
        ("era-golden", "The Golden Era", -4999, Some(-1000), "#F59E0B"),
        ("era-war", "The Great War", -999, Some(0), "#EF4444"),
        ("era-silence", "The Silence", 1, Some(500), "#6B7280"),
        ("era-rebirth", "The Rebirth", 501, Some(2000), "#10B981"),
    ];

    for (id, name, start, end, color) in &eras {
        sqlx::query(
            "INSERT OR IGNORE INTO timeline_eras (id, universe_id, name, start_year, end_year, color) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
            .bind(*id)
            .bind(universe_id)
            .bind(*name)
            .bind(*start)
            .bind(*end)
            .bind(*color)
            .execute(pool)
            .await?;
    }

    // Ensure at least 15 events (3 per era)
    let (ev_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM timeline_events WHERE universe_id = ?")
            .bind(universe_id)
            .fetch_one(pool)
            .await?;

    if ev_count >= 15 {
        return Ok(());
    }

    let per_era = vec![
        ("Age of Myth", -10000, -5000, "#8B5CF6"),
        ("The Golden Era", -4999, -1000, "#F59E0B"),
        ("The Great War", -999, 0, "#EF4444"),
        ("The Silence", 1, 500, "#6B7280"),
        ("The Rebirth", 501, 2000, "#10B981"),
    ];

    for (era_name, start, end, col) in per_era {
        let y1 = start + ((end - start) / 5);
        let y2 = start + ((end - start) / 2);
        let y3 = start + (((end - start) * 4) / 5);

        let events = vec![
            (format!("The First Omen — {}", era_name), y1, "Major", "Omen"),
            (format!("A Turning Point — {}", era_name), y2, "Normal", "Political"),
            (format!("Echoes and Aftermath — {}", era_name), y3, "Normal", "Myth"),
        ];

        for (title, year, importance, kind) in events {
            let display_date = year.to_string(); // keep UI stable
            sqlx::query(
                "INSERT INTO timeline_events (id, universe_id, title, description, year, display_date, importance, kind, color) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
                .bind(Uuid::new_v4().to_string())
                .bind(universe_id)
                .bind(title)
                .bind("")
                .bind(year)
                .bind(display_date)
                .bind(importance)
                .bind(kind)
                .bind(col)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}
