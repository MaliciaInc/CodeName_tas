use sqlx::SqlitePool;

pub mod bestiary;
pub mod locations;
pub mod timeline;
pub mod pm_tools;

pub async fn run_all(pool: &SqlitePool, universe_id: &str) -> Result<(), sqlx::Error> {
    bestiary::seed(pool, universe_id).await?;
    locations::seed(pool, universe_id).await?;
    timeline::seed(pool, universe_id).await?;
    pm_tools::seed(pool).await?;
    Ok(())
}
