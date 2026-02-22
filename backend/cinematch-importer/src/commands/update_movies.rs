use anyhow::{Context, Result};
use cinematch_db::conn::qdrant::models::MovieData;
use cinematch_db::{BATCH_SIZE, Database};
use indicatif::ProgressBar;
use std::collections::HashMap;

use crate::csv_loader::load_and_preprocess_movies;
use crate::ollama::OllamaService;

const MOVIES: &str = "movies";
const EMBEDDING_BATCH_SIZE: usize = 10;

/// Run the full movies import pipeline:
/// CSV → embeddings → Qdrant + Postgres
pub async fn run(database: &Database, ollama: &OllamaService, movies_path: &str) -> Result<()> {
    let start = std::time::Instant::now();

    // Load CSV
    println!("📂 Loading movies with async streaming iterator...");
    let mut movies_stream = load_and_preprocess_movies(movies_path)
        .await
        .context("Failed to load movies from CSV")?;
    println!("  ✓ Async streaming iterator ready\n");

    // Create Qdrant collection
    println!("📦 Creating Qdrant movies collection...");
    database
        .vector
        .setup(MOVIES)
        .await
        .context("Failed to create collections")?;
    println!("  ✓ Collection: movies (with 4 named vectors)\n");

    // Stream process movies in batches
    println!("🚀 Processing movies with streaming embeddings...\n");

    let m = indicatif::MultiProgress::new();

    let pb_movies = m.add(ProgressBar::new_spinner());
    pb_movies.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("📚 Movies    {pos} processed")
            .unwrap(),
    );

    let pb_uploads = m.add(ProgressBar::new_spinner());
    pb_uploads.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("💾 Uploads   {pos} points (4 vectors each)")
            .unwrap(),
    );

    let mut total_points_uploaded = 0u64;
    let mut total_pg_inserted = 0u64;
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    while let Some(movie) = movies_stream.next_movie().await.ok().flatten() {
        batch.push(movie);

        if batch.len() >= BATCH_SIZE {
            let batch_to_process = std::mem::take(&mut batch);
            let (qdrant, pg) =
                process_batch(ollama, database, &pb_movies, &batch_to_process).await?;
            total_points_uploaded += qdrant;
            total_pg_inserted += pg;
            pb_uploads.inc(qdrant);
        }
    }

    // Process remaining
    if !batch.is_empty() {
        let (qdrant, pg) = process_batch(ollama, database, &pb_movies, &batch).await?;
        total_points_uploaded += qdrant;
        total_pg_inserted += pg;
        pb_uploads.inc(qdrant);
    }

    pb_movies.finish_with_message("✓ All movies processed");
    pb_uploads.finish_with_message("✓ All uploads complete");
    println!();

    let elapsed = start.elapsed();
    println!("📊 Movies Results:");
    println!("  Qdrant points uploaded: {}", total_points_uploaded);
    println!("  PG rows inserted:      {}", total_pg_inserted);
    println!("  Vectors per point:     4 (plot, cast_crew, reviews, combined)");
    println!("  Total vectors:         ~{}", total_points_uploaded * 4);
    if total_points_uploaded > 0 {
        let throughput = (total_points_uploaded * 4) as f64 / elapsed.as_secs_f64();
        println!("  ⚡ Throughput: {:.2} vectors/sec", throughput);
    }
    println!("  ⏱️  Time: {:.2}s\n", elapsed.as_secs_f64());

    Ok(())
}

/// Process a batch of movies: generate embeddings, then upload.
async fn process_batch(
    ollama: &OllamaService,
    database: &Database,
    pb_movies: &ProgressBar,
    batch_movies: &[MovieData],
) -> Result<(u64, u64)> {
    let mut plot_texts = Vec::new();
    let mut cast_crew_texts = Vec::new();
    let mut reviews_texts = Vec::new();
    let mut combined_texts = Vec::new();

    for movie in batch_movies {
        let plot_text = movie.get_plot_text();
        let cast_crew_text = movie.get_cast_crew_text();
        let reviews_text = movie.get_reviews_text();
        let combined_text = movie.get_combined_text();

        if !plot_text.trim().is_empty() {
            plot_texts.push(plot_text);
        }
        if !cast_crew_text.trim().is_empty() {
            cast_crew_texts.push(cast_crew_text);
        }
        if !reviews_text.trim().is_empty() {
            reviews_texts.push(reviews_text);
        }
        if !combined_text.trim().is_empty() {
            combined_texts.push(combined_text);
        }

        pb_movies.inc(1);
    }

    // Generate all 4 embeddings concurrently (I/O-bound → async concurrency)
    let (plot_embeddings, cast_crew_embeddings, reviews_embeddings, combined_embeddings) = tokio::join!(
        generate_embeddings_batched(ollama, plot_texts),
        generate_embeddings_batched(ollama, cast_crew_texts),
        generate_embeddings_batched(ollama, reviews_texts),
        generate_embeddings_batched(ollama, combined_texts),
    );

    // Build embeddings map: movie_id → { vector_name → embedding }
    let mut embeddings_map: HashMap<i64, HashMap<String, Vec<f32>>> = HashMap::new();

    for (idx, movie) in batch_movies.iter().enumerate() {
        let mut vectors = HashMap::new();
        if let Some(emb) = plot_embeddings.get(idx) {
            vectors.insert("plot_vector".to_string(), emb.clone());
        }
        if let Some(emb) = cast_crew_embeddings.get(idx) {
            vectors.insert("cast_crew_vector".to_string(), emb.clone());
        }
        if let Some(emb) = reviews_embeddings.get(idx) {
            vectors.insert("reviews_vector".to_string(), emb.clone());
        }
        if let Some(emb) = combined_embeddings.get(idx) {
            vectors.insert("combined_vector".to_string(), emb.clone());
        }
        if !vectors.is_empty() {
            embeddings_map.insert(movie.movie_id, vectors);
        } else {
            eprintln!(
                "⚠️  No embeddings for movie {}: {}",
                movie.movie_id, movie.title
            );
        }
    }

    // Delegate storage to the database crate
    let (qdrant_uploaded, pg_inserted) = database
        .upload_movies(batch_movies, &embeddings_map, MOVIES)
        .await
        .context("Failed to upload movie batch")?;

    Ok((qdrant_uploaded, pg_inserted))
}

/// Generate embeddings in sub-batches.
async fn generate_embeddings_batched(ollama: &OllamaService, texts: Vec<String>) -> Vec<Vec<f32>> {
    let mut all_embeddings = Vec::new();
    let filtered: Vec<String> = texts.into_iter().filter(|t| !t.trim().is_empty()).collect();

    if filtered.is_empty() {
        return Vec::new();
    }

    for text_batch in filtered.chunks(EMBEDDING_BATCH_SIZE) {
        match ollama.embed_batch(text_batch).await {
            Ok(batch_embeddings) => {
                let validated: Vec<Vec<f32>> = batch_embeddings
                    .into_iter()
                    .filter(|emb| emb.iter().all(|v| v.is_finite()))
                    .collect();
                all_embeddings.extend(validated);
            }
            Err(e) => {
                eprintln!("⚠️  Failed to generate embeddings for batch: {}", e);
                return all_embeddings;
            }
        }
    }

    all_embeddings
}
