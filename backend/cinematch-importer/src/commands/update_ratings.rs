use anyhow::{Context, Result};
use cinematch_db::Database;
use indicatif::ProgressBar;
use serde::Deserialize;
use std::collections::HashMap;

const RATINGS_COLLECTION: &str = "ratings";

/// Row from ratings.csv
#[derive(Debug, Deserialize)]
struct RatingRow {
    #[serde(rename = "userId")]
    user_id: i64,
    #[serde(rename = "movieId")]
    movie_id: i64,
    rating: f64,
}

/// Running accumulator for a single (user, movie) pair.
/// Stores (sum, count) instead of Vec<f64> — ~24× less memory per pair.
struct RatingAcc {
    sum: f64,
    count: u32,
}

impl RatingAcc {
    fn new(rating: f64) -> Self {
        Self {
            sum: rating,
            count: 1,
        }
    }

    fn add(&mut self, rating: f64) {
        self.sum += rating;
        self.count += 1;
    }

    fn mean(&self) -> f64 {
        self.sum / self.count as f64
    }
}

use crate::utils::stats::WelfordStats;

/// Run the ratings import pipeline:
/// ratings.csv → z-score normalization → sparse vectors → Qdrant
pub async fn run(database: &Database, ratings_path: &str, movies_db_path: &str) -> Result<()> {
    let start = std::time::Instant::now();

    println!("📂 Loading ratings from CSV...");

    // Step 1: Build movie_id set from movies_DB.csv (only keep ratings for known movies)
    let movie_set = build_movie_set(movies_db_path).await?;
    println!("  ✓ {} movies in reference set", movie_set.len());

    // Step 2: Stream CSV, aggregate per (user, movie) using (sum, count) accumulators,
    //         and compute global mean/std via Welford's algorithm — single pass.
    let (user_ratings, global_mean, global_std) =
        stream_and_aggregate(ratings_path, &movie_set).await?;
    println!(
        "  ✓ {} users with ratings (mean={:.3}, std={:.3})",
        user_ratings.len(),
        global_mean,
        global_std
    );

    // Step 3: Create Qdrant collection
    println!("\n📦 Creating Qdrant ratings collection...");
    database
        .upload_ratings_setup(RATINGS_COLLECTION)
        .await
        .context("Failed to create ratings collection")?;
    println!("  ✓ Collection: ratings (sparse vectors)\n");

    // Step 4: Stream upload — iterate users, normalize, upload in batches
    println!("🚀 Uploading user rating vectors...\n");
    let pb = ProgressBar::new(user_ratings.len() as u64);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("💾 Users   [{bar:40}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("█▓░"),
    );

    let mut total_uploaded = 0u64;
    let batch_size = 100;
    let mut batch_points = Vec::with_capacity(batch_size);

    for (user_id, movies) in &user_ratings {
        let indices: Vec<u32> = movies.keys().map(|&id| id as u32).collect();
        let values: Vec<f32> = movies
            .values()
            .map(|acc| ((acc.mean() - global_mean) / global_std) as f32)
            .collect();
        let movie_ids: Vec<i64> = movies.keys().copied().collect();

        batch_points.push((*user_id, indices, values, movie_ids));

        if batch_points.len() >= batch_size {
            let count = database
                .upload_ratings_batch(RATINGS_COLLECTION, &batch_points)
                .await
                .context("Failed to upload ratings batch")?;
            total_uploaded += count;
            pb.inc(batch_points.len() as u64);
            batch_points.clear();
        }
    }

    // Upload remaining
    if !batch_points.is_empty() {
        let count = database
            .upload_ratings_batch(RATINGS_COLLECTION, &batch_points)
            .await
            .context("Failed to upload remaining ratings")?;
        total_uploaded += count;
        pb.inc(batch_points.len() as u64);
    }

    pb.finish_with_message("✓ All users uploaded");
    println!();

    let elapsed = start.elapsed();
    println!("📊 Ratings Results:");
    println!("  Users uploaded: {}", total_uploaded);
    println!("  ⏱️  Time: {:.2}s\n", elapsed.as_secs_f64());

    Ok(())
}

/// Build a set of valid movie IDs from movies_DB.csv (first column).
async fn build_movie_set(path: &str) -> Result<std::collections::HashSet<i64>> {
    let path = path.to_string();
    tokio::task::spawn_blocking(move || {
        let mut set = std::collections::HashSet::new();
        let file = std::fs::File::open(&path)?;
        let mut reader = csv::Reader::from_reader(std::io::BufReader::new(file));

        for result in reader.records() {
            if let Ok(record) = result
                && let Some(id_str) = record.get(0)
                && let Ok(id) = id_str.parse::<i64>()
            {
                set.insert(id);
            }
        }
        Ok(set)
    })
    .await?
}

/// Single-pass streaming aggregation:
/// - Reads ratings.csv row by row (never loads the whole file)
/// - Aggregates per (user, movie) using (sum, count) accumulators
/// - Computes global mean/std via Welford's online algorithm
///
/// Returns (user_ratings, global_mean, global_std).
async fn stream_and_aggregate(
    path: &str,
    movie_set: &std::collections::HashSet<i64>,
) -> Result<(HashMap<i64, HashMap<i64, RatingAcc>>, f64, f64)> {
    let path = path.to_string();
    let movie_set = movie_set.clone();

    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(&path)?;
        let mut reader = csv::Reader::from_reader(std::io::BufReader::new(file));

        let mut user_ratings: HashMap<i64, HashMap<i64, RatingAcc>> = HashMap::new();
        let mut stats = WelfordStats::new();
        let mut total = 0u64;
        let mut skipped = 0u64;

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("📖 Ratings   {pos} loaded")
                .unwrap(),
        );

        for result in reader.deserialize::<RatingRow>() {
            total += 1;
            if total.is_multiple_of(100_000) {
                pb.set_position(total);
            }

            let row = match result {
                Ok(r) => r,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            if !movie_set.contains(&row.movie_id) {
                skipped += 1;
                continue;
            }

            // Update running accumulator
            let movie_map = user_ratings.entry(row.user_id).or_default();
            match movie_map.get_mut(&row.movie_id) {
                Some(acc) => acc.add(row.rating),
                None => {
                    movie_map.insert(row.movie_id, RatingAcc::new(row.rating));
                }
            }

            // Feed every rating into Welford's online stats
            stats.update(row.rating);
        }

        pb.finish_and_clear();
        println!(
            "  ✓ {} ratings loaded, {} skipped (unknown movies / parse errors)",
            total - skipped,
            skipped
        );

        let (mean, std) = stats.finalize();
        Ok((user_ratings, mean, std))
    })
    .await?
}
