use anyhow::{Context, Result};
use cinematch_db::Database;

/// Wipe all imported data from Qdrant and Postgres.
pub async fn run(database: &Database) -> Result<()> {
    println!("⚠️  Removing all imported data...\n");

    database.wipe_all().await.context("Failed to wipe data")?;

    println!("✅ All data removed.\n");
    Ok(())
}
