use anyhow::{Context, Result};
use indicatif::ProgressBar;
use std::time::Instant;

mod deserializer;
mod ollama;

mod csv_loader;

use csv_loader::load_and_preprocess_movies;
use ollama::OllamaService;

pub use cinematch_db::vector::models::*;

use cinematch_db::Database;

const EMBEDDING_BATCH_SIZE: usize = 10; // Generate embeddings for 10 texts at once
use cinematch_db::BATCH_SIZE;

// Service configuration
const OLLAMA_HOST: &str = "localhost";
const OLLAMA_PORT: u16 = 11434;

const MOVIES: &str = "movies";

use cinematch_db::vector::qdrant::QdrantService;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    println!("╔════════════════════════════════════════╗");
    println!("║  CINEMATCH EMBEDDING PIPELINE (RUST)  ║");
    println!("║        Streaming Processing Init      ║");
    println!("╚════════════════════════════════════════╝\n");

    let start_time = Instant::now();

    // Load and preprocess CSV dataset with async streaming iterator
    println!("📂 Loading movies with async streaming iterator...");
    let mut movies_stream = load_and_preprocess_movies("movies_DB.csv")
        .await
        .context("Failed to load movies from CSV")?;

    // Initialize services first before consuming iterator
    println!("✓ Async streaming iterator ready\n");

    // Initialize services
    println!("🔗 Initializing services...");
    let ollama = OllamaService::new(OLLAMA_HOST, OLLAMA_PORT);

    // read from env
    let pg_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://cinematch_user:password@localhost:5432/cinematch".to_string()
    });
    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());

    let database = Database::new(&pg_url, &qdrant_url).context("Failed to initialize database")?;
    println!("  ✓ Postgres connected");
    println!("  ✓ Qdrant connected");

    // Run database migrations
    if database.run_migrations(&pg_url).await.is_err() {
        println!("  ✗ Failed to run database migrations");
        return Err(anyhow::anyhow!("Database migration error"));
    }
    println!("  ✓ Database migrations complete");

    // Check Ollama
    ollama
        .check_service()
        .await
        .context("Ollama service not available")?;
    println!("  ✓ Ollama available");

    // Create collections
    println!("📦 Creating Qdrant collections...");
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
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    // Process movies in streaming batches (async)
    while let Some(movie) = movies_stream.next_movie().await.ok().flatten() {
        batch.push(movie);

        // When batch is full, process it
        if batch.len() >= BATCH_SIZE {
            let batch_to_process = std::mem::take(&mut batch);
            process_batch(
                &ollama,
                &database,
                &pb_movies,
                &pb_uploads,
                &batch_to_process,
                &mut total_points_uploaded,
            )
            .await?;
        }
    }

    // Process any remaining movies
    if !batch.is_empty() {
        process_batch(
            &ollama,
            &database,
            &pb_movies,
            &pb_uploads,
            &batch,
            &mut total_points_uploaded,
        )
        .await?;
    }

    pb_movies.finish_with_message("✓ All movies processed");
    pb_uploads.finish_with_message("✓ All uploads complete");
    println!();

    let elapsed = start_time.elapsed();

    // Print final stats
    println!("╔════════════════════════════════════════╗");
    println!("║          EMBEDDING SUMMARY             ║");
    println!("╚════════════════════════════════════════╝");
    println!();
    println!("📊 Results:");
    println!("  Points uploaded:     {}", total_points_uploaded);
    println!("  Vectors per point:   4 (plot, cast_crew, reviews, combined)");
    println!("  Total vectors:       ~{}", total_points_uploaded * 4);
    println!();
    println!("⏱️  Total time: {:.2}s", elapsed.as_secs_f64());

    if total_points_uploaded > 0 {
        let throughput = (total_points_uploaded * 4) as f64 / elapsed.as_secs_f64();
        println!("⚡ Throughput: {:.2} vectors/sec", throughput);
    }
    println!();
    println!("✅ Pipeline completed successfully!");

    Ok(())
}

async fn process_batch(
    ollama: &OllamaService,
    database: &Database,
    pb_movies: &indicatif::ProgressBar,
    pb_uploads: &indicatif::ProgressBar,
    batch_movies: &[MovieData],
    total_points_uploaded: &mut u64,
) -> Result<()> {
    let mut plot_texts = Vec::new();
    let mut cast_crew_texts = Vec::new();
    let mut reviews_texts = Vec::new();
    let mut combined_texts = Vec::new();
    let mut movie_ids = Vec::new();

    // Prepare all 4 semantic text variants for this batch
    for movie in batch_movies {
        movie_ids.push(movie.movie_id);

        let plot_text = movie.get_plot_text();
        let cast_crew_text = movie.get_cast_crew_text();
        let reviews_text = movie.get_reviews_text();
        let combined_text = movie.get_combined_text();

        // Only include non-empty texts
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

    // Generate all 4 embeddings for this batch (with error handling)
    let plot_embeddings = if !plot_texts.is_empty() {
        match generate_embeddings_batched(ollama, plot_texts).await {
            Ok(emb) => emb,
            Err(e) => {
                eprintln!("⚠️  Failed to generate plot embeddings: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let cast_crew_embeddings = if !cast_crew_texts.is_empty() {
        match generate_embeddings_batched(ollama, cast_crew_texts).await {
            Ok(emb) => emb,
            Err(e) => {
                eprintln!("⚠️  Failed to generate cast_crew embeddings: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let reviews_embeddings = if !reviews_texts.is_empty() {
        match generate_embeddings_batched(ollama, reviews_texts).await {
            Ok(emb) => emb,
            Err(e) => {
                eprintln!("⚠️  Failed to generate reviews embeddings: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let combined_embeddings = if !combined_texts.is_empty() {
        match generate_embeddings_batched(ollama, combined_texts).await {
            Ok(emb) => emb,
            Err(e) => {
                eprintln!("⚠️  Failed to generate combined embeddings: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // Build points with multiple named vectors and keep mapping to movie index
    let mut points = Vec::new();
    let mut failed_movies = 0;

    for (idx, movie) in batch_movies.iter().enumerate() {
        let mut vectors = std::collections::HashMap::new();
        if !plot_embeddings.is_empty() && idx < plot_embeddings.len() {
            vectors.insert("plot_vector".to_string(), plot_embeddings[idx].clone());
        }
        if !cast_crew_embeddings.is_empty() && idx < cast_crew_embeddings.len() {
            vectors.insert(
                "cast_crew_vector".to_string(),
                cast_crew_embeddings[idx].clone(),
            );
        }
        if !reviews_embeddings.is_empty() && idx < reviews_embeddings.len() {
            vectors.insert(
                "reviews_vector".to_string(),
                reviews_embeddings[idx].clone(),
            );
        }
        if !combined_embeddings.is_empty() && idx < combined_embeddings.len() {
            vectors.insert(
                "combined_vector".to_string(),
                combined_embeddings[idx].clone(),
            );
        }
        if !vectors.is_empty() {
            let point = QdrantService::create_point_with_vectors(movie, vectors);
            points.push(point);
        } else {
            failed_movies += 1;
            eprintln!(
                "⚠️  Failed to generate embeddings for movie {}: {}",
                movie.movie_id, movie.title
            );
        }
    }

    // Upload batch to Qdrant
    if !points.is_empty() {
        match database.vector.upload_batch(MOVIES, &points).await {
            Ok(_) => {
                let vector_count = points
                    .iter()
                    .map(|p| {
                        if let Some(qdrant_client::qdrant::Vectors {
                            vectors_options:
                                Some(qdrant_client::qdrant::vectors::VectorsOptions::Vectors(nv)),
                        }) = &p.vectors
                        {
                            nv.vectors.len() as u64
                        } else {
                            0
                        }
                    })
                    .sum::<u64>();
                pb_uploads.inc(vector_count);
                *total_points_uploaded += points.len() as u64;
            }
            Err(e) => {
                eprintln!("⚠️  Failed to upload batch to Qdrant: {}", e);
                // Continue processing next batch instead of failing completely
            }
        }
    }

    // Batch insert to Postgres (movie_id is always the qdrant id)
    let movies_for_pg = batch_movies.to_vec();
    if let Err(e) = database.insert_movie_data_batch(&movies_for_pg).await {
        eprintln!("⚠️  Failed to insert batch to Postgres: {}", e);
    }

    if failed_movies > 0 {
        eprintln!(
            "⚠️  Skipped {} movies in this batch due to embedding errors",
            failed_movies
        );
    }

    Ok(())
}

async fn generate_embeddings_batched(
    ollama: &OllamaService,
    texts: Vec<String>,
) -> Result<Vec<Vec<f32>>> {
    let mut all_embeddings = Vec::new();

    // Filter out empty or whitespace-only texts
    let filtered_texts: Vec<String> = texts.into_iter().filter(|t| !t.trim().is_empty()).collect();

    if filtered_texts.is_empty() {
        return Ok(Vec::new());
    }

    // Process texts in smaller batches to avoid overwhelming Ollama
    for text_batch in filtered_texts.chunks(EMBEDDING_BATCH_SIZE) {
        match ollama.embed_batch(text_batch).await {
            Ok(batch_embeddings) => {
                // Validate embeddings - skip any with NaN values
                let validated: Vec<Vec<f32>> = batch_embeddings
                    .into_iter()
                    .filter(|emb| {
                        let valid = emb.iter().all(|v| v.is_finite());
                        if !valid {
                            eprintln!(
                                "⚠️  Skipping embedding with invalid values (NaN/Inf detected)"
                            );
                        }
                        valid
                    })
                    .collect();

                all_embeddings.extend(validated);
            }
            Err(e) => {
                // Log the error but continue processing
                eprintln!("⚠️  Failed to generate embeddings for batch: {}", e);
                // Return what we have so far - the batch will be skipped
                return Ok(all_embeddings);
            }
        }
    }

    Ok(all_embeddings)
}
