use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cinematch-importer",
    about = "Cinematch data import pipeline",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Full pipeline: import movies + generate embeddings + upload ratings
    UpdateAll,
    /// Import movies from CSV, generate embeddings, upload to Qdrant + Postgres
    UpdateMovies,
    /// Import user ratings from CSV, build sparse vectors, upload to Qdrant
    UpdateRatings,
    /// Wipe all imported data from Qdrant and Postgres
    RemoveAll,
    /// Download datasets (WIP)
    Download,
}
