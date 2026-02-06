use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn seed(pool: &SqlitePool, universe_id: &str) -> Result<(), sqlx::Error> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM bestiary_entries WHERE universe_id = ?")
            .bind(universe_id)
            .fetch_one(pool)
            .await?;

    if count >= 7 {
        return Ok(());
    }

    let creatures = vec![
        ("Astral Whale", "Celestial", "Space", "Swims between stars.", "Low"),
        ("Swamp Hag", "Humanoid", "Marsh", "Deceptive witch.", "High"),
        ("Iron Golem", "Construct", "Mountains", "Guardian of the mines.", "Medium"),
        ("Void Leech", "Aberration", "Void", "Drains magic.", "High"),
        ("Crystal Spider", "Beast", "Caves", "Weaves glass webs.", "Medium"),
        ("Phoenix Hatchling", "Elemental", "Volcano", "Reborn from ashes.", "High"),
        ("Shadow Stalker", "Demon", "Ruins", "Hunts in darkness.", "Extreme"),
    ];

    for (name, kind, habitat, description, danger) in creatures {
        sqlx::query(
            "INSERT INTO bestiary_entries (id, universe_id, name, kind, habitat, description, danger) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
            .bind(Uuid::new_v4().to_string())
            .bind(universe_id)
            .bind(name)
            .bind(kind)
            .bind(habitat)
            .bind(description)
            .bind(danger)
            .execute(pool)
            .await?;
    }

    Ok(())
}
