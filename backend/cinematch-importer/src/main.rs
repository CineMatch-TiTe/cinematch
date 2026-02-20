use anyhow::{Context, Result};
use clap::Parser;

mod cli;
mod commands;
mod csv_loader;
mod ollama;
mod parsing;
pub mod utils;

use cli::{Cli, Command};
use ollama::OllamaService;

use cinematch_db::Database;

// Service configuration
const OLLAMA_HOST: &str = "localhost";
const OLLAMA_PORT: u16 = 11434;

// File paths
const RATINGS_CSV: &str = "data/ratings.csv";
const MOVIES_DB_CSV: &str = "data/movies_DB.csv";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();

    println!("╔════════════════════════════════════════╗");
    println!("║  CINEMATCH EMBEDDING PIPELINE (RUST)  ║");
    println!("╚════════════════════════════════════════╝\n");

    let start = std::time::Instant::now();

    // Initialize services
    println!("🔗 Initializing services...");

    let pg_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://cinematch_user:password@localhost:5432/cinematch".to_string()
    });
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());

    let database =
        Database::new(&pg_url, &redis_url, &qdrant_url).context("Failed to initialize database")?;
    println!("  ✓ Database connections ready");

    let ollama = OllamaService::new(OLLAMA_HOST, OLLAMA_PORT);

    // Run migrations (needed for all commands except maybe remove-all)
    if !matches!(cli.command, Command::RemoveAll) {
        if database.run_migrations(&pg_url).await.is_err() {
            return Err(anyhow::anyhow!("Database migration error"));
        }
        println!("  ✓ Migrations complete");
    }

    // Dispatch command
    match cli.command {
        Command::UpdateAll => {
            ollama
                .check_service()
                .await
                .context("Ollama service not available")?;
            println!("  ✓ Ollama available\n");

            commands::update_movies::run(&database, &ollama, MOVIES_DB_CSV).await?;
            commands::update_ratings::run(&database, RATINGS_CSV, MOVIES_DB_CSV).await?;
        }
        Command::UpdateMovies => {
            ollama
                .check_service()
                .await
                .context("Ollama service not available")?;
            println!("  ✓ Ollama available\n");

            commands::update_movies::run(&database, &ollama, MOVIES_DB_CSV).await?;
        }
        Command::UpdateRatings => {
            println!();
            commands::update_ratings::run(&database, RATINGS_CSV, MOVIES_DB_CSV).await?;
        }
        Command::RemoveAll => {
            println!();
            commands::remove_all::run(&database).await?;
        }
        Command::Download => {
            println!("\n⚠️  Download command is not yet implemented.\n");
            println!("  This will download the required datasets:");
            println!("  - movies_DB.csv");
            println!("  - ratings.csv");
            println!("  - movies.csv");
            println!("  - links.csv\n");
        }
        Command::UpdateOnboarding => {
            println!("🚀 Starting Onboarding Update...");
            let ratings_path = std::path::PathBuf::from(RATINGS_CSV);
            commands::update_onboarding::run(database.clone().into(), ratings_path, None).await?;
        }
    }

    let elapsed = start.elapsed();
    println!("⏱️  Total time: {:.2}s", elapsed.as_secs_f64());
    println!("✅ Done!\n");

    Ok(())
}
