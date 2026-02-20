use anyhow::Result;
use cinematch_db::Database;
use cinematch_db::repo::onboarding::models::{NewOnboardingCluster, NewOnboardingMovie};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use linfa::Dataset;
use linfa::traits::{Fit, Predict};
use linfa_clustering::KMeans;
use ndarray::Array2;
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Minimum ratings a movie must have to be considered for onboarding.
const MIN_MOVIE_RATINGS: usize = 50;
/// Minimum ratings a user must have to be included in clustering.
const MIN_USER_RATINGS: usize = 20;
/// Number of clusters (taste archetypes).
const NUM_CLUSTERS: usize = 200;
/// Regularization term for Laplace smoothing.
const LAPLACE_SMOOTHING: f64 = 0.1;

#[derive(Debug, Deserialize)]
struct RatingRow {
    #[serde(rename = "userId")]
    user_id: u32,
    #[serde(rename = "movieId")]
    movie_id: i64,
    rating: f32,
}

use crate::utils::stats::WelfordStats;

pub async fn run(db: Arc<Database>, ratings_path: PathBuf, _pool: Option<PathBuf>) -> Result<()> {
    log::info!("Starting onboarding data update...");

    // 1. Load Genre Map (MovieID -> Vec<GenreUUID>)
    log::info!("Loading movie genres...");
    let (movie_genres_map, all_genres) = db.get_all_movie_genres_data().await?;
    let genre_index: HashMap<uuid::Uuid, usize> = all_genres
        .iter()
        .enumerate()
        .map(|(i, &g)| (g, i))
        .collect();
    let num_genres = all_genres.len();
    log::info!(
        "Found {} movies with genres, totaling {} unique genres.",
        movie_genres_map.len(),
        num_genres
    );

    // 2. First Pass: Compute User Stats & Movie Rating Counts
    log::info!("Pass 1: Analyzing ratings (User Stats & Movie Counts)...");
    let mut user_stats: HashMap<u32, WelfordStats> = HashMap::new();
    let mut movie_rating_counts: HashMap<i64, usize> = HashMap::new();

    let mut user_genre_sums: HashMap<u32, Vec<f32>> = HashMap::new();
    let mut user_genre_counts: HashMap<u32, Vec<usize>> = HashMap::new();

    let m = MultiProgress::new();
    let pb = m.add(ProgressBar::new_spinner());
    pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg} {pos}")?);
    pb.set_message("Reading ratings CSV...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(&ratings_path)?;

    for result in rdr.deserialize() {
        let row: RatingRow = result?;

        *movie_rating_counts.entry(row.movie_id).or_default() += 1;
        user_stats
            .entry(row.user_id)
            .or_default()
            .update(row.rating as f64);

        if let Some(genres) = movie_genres_map.get(&row.movie_id) {
            let sums = user_genre_sums
                .entry(row.user_id)
                .or_insert_with(|| vec![0.0; num_genres]);
            let counts = user_genre_counts
                .entry(row.user_id)
                .or_insert_with(|| vec![0; num_genres]);

            for &genre_id in genres {
                if let Some(&idx) = genre_index.get(&genre_id) {
                    sums[idx] += row.rating;
                    counts[idx] += 1;
                }
            }
        }

        if user_stats.len().is_multiple_of(10000) {
            pb.set_message(format!("Processed {} users...", user_stats.len()));
            pb.set_position(user_stats.len() as u64);
        }
    }
    pb.finish_with_message("Rating analysis complete.");

    // Filter movies
    let valid_movies: HashSet<i64> = movie_rating_counts
        .iter()
        .filter(|&(&id, &count)| count >= MIN_MOVIE_RATINGS && movie_genres_map.contains_key(&id))
        .map(|(&id, _)| id)
        .collect();
    log::info!(
        "Found {} valid movies with >= {} ratings and known genres.",
        valid_movies.len(),
        MIN_MOVIE_RATINGS
    );

    // 3. Build User Vectors (Normalized)
    log::info!("Building normalized user vectors for clustering...");
    let mut valid_users = Vec::new();
    let mut data_matrix = Vec::new();

    for (user_id, stats) in &user_stats {
        if stats.count() < MIN_USER_RATINGS as u64 {
            continue;
        }
        let (user_mean, std_dev) = stats.finalize();
        let std_dev = std_dev as f32;
        let user_mean = user_mean as f32;

        if std_dev < 0.2 {
            continue;
        }

        let sums = match user_genre_sums.get(user_id) {
            Some(s) => s,
            None => continue,
        };
        let counts = match user_genre_counts.get(user_id) {
            Some(c) => c,
            None => continue,
        };

        let mut vector = Vec::with_capacity(num_genres);

        for i in 0..num_genres {
            if counts[i] > 0 {
                let raw_avg = sums[i] / counts[i] as f32;
                let normalized = (raw_avg - user_mean) / std_dev;
                vector.push(normalized);
            } else {
                vector.push(0.0);
            }
        }

        valid_users.push(*user_id);
        data_matrix.extend(vector);
    }

    let num_samples = valid_users.len();
    log::info!("Clustering {} valid users...", num_samples);

    if num_samples < NUM_CLUSTERS {
        anyhow::bail!("Not enough data to cluster.");
    }

    // 4. Run K-Means
    // Using rayon for K-Means internally if supported, but linfa-clustering might be single-threaded.
    // However, the main cost here is fitting.
    let dataset = Array2::from_shape_vec((num_samples, num_genres), data_matrix)?;
    let dataset = Dataset::from(dataset);

    log::info!("Running K-Means (K={})...", NUM_CLUSTERS);
    let model = KMeans::params(NUM_CLUSTERS)
        .max_n_iterations(200)
        .tolerance(1e-5)
        .fit(&dataset)
        .map_err(|e| anyhow::anyhow!("Clustering failed: {}", e))?;

    let centroids = model.centroids();
    let predicted = model.predict(&dataset);
    let user_clusters: HashMap<u32, usize> = valid_users
        .iter()
        .zip(predicted.iter())
        .map(|(&uid, &cid)| (uid, cid))
        .collect();

    let mut onboarding_clusters = Vec::new();
    let mut cluster_user_counts = vec![0; NUM_CLUSTERS];

    for (i, count_slot) in cluster_user_counts
        .iter_mut()
        .enumerate()
        .take(NUM_CLUSTERS)
    {
        let centroid_vec: Vec<f64> = centroids.row(i).iter().map(|&v| v as f64).collect();
        let count = predicted.iter().filter(|&&c| c == i).count();
        *count_slot = count;

        onboarding_clusters.push(NewOnboardingCluster {
            cluster_id: i as i16,
            centroid: json!(centroid_vec),
            user_count: count as i32,
        });
    }
    log::info!("Cluster sizes: {:?}", cluster_user_counts);
    db.store_onboarding_clusters(&onboarding_clusters).await?;

    // 5. Pass 2: Rating Distributions
    log::info!("Pass 2: Computing probability distributions...");

    // We can't parallelize reading CSV easily without chunking, but we can do it later.
    // For now, let's read sequentially but optimize the accumulation.
    // Actually, avoiding DashMap overhead if we can just use a single thread for reading might be better
    // BUT we can use thread-local accumulators if we want to parallelize parsing.
    // For simplicity and correctness with the CSV crate, let's keep sequential read but optimize the lookups.

    // Pre-allocate movie distributions
    // Use DashMap for parallel updates? No, if reading is sequential, DashMap adds overhead.
    // Regular HashMap is fine.

    let mut movie_dists: HashMap<i64, Vec<[f64; 10]>> = valid_movies
        .iter()
        .map(|&mid| (mid, vec![[0.0; 10]; NUM_CLUSTERS]))
        .collect();

    let pb_pass2 = m.add(ProgressBar::new(0)); // We don't know exact lines unless we count first
    pb_pass2.set_style(ProgressStyle::default_bar().template("{spinner} {msg} {pos} lines")?);
    pb_pass2.set_message("Processing ratings for distributions...");
    pb_pass2.enable_steady_tick(Duration::from_millis(100));

    let mut rdr2 = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(&ratings_path)?;

    // Optimization: Pre-fetch cluster IDs for all users to avoid HashMap lookup per row?
    // user_clusters is HashMap<u32, usize>.
    // That's fast enough.

    let mut lines_processed = 0u64;
    for result in rdr2.deserialize() {
        let row: RatingRow = result?;
        lines_processed += 1;
        if lines_processed % 100_000 == 0 {
            pb_pass2.set_position(lines_processed);
        }

        if valid_movies.contains(&row.movie_id)
            && let Some(&cluster_id) = user_clusters.get(&row.user_id)
        {
            let bucket_idx = rating_to_bucket(row.rating);
            // safe unwrap because we initialized movie_dists with all valid_movies
            if let Some(dists) = movie_dists.get_mut(&row.movie_id) {
                dists[cluster_id][bucket_idx] += 1.0;
            }
        }
    }
    pb_pass2.finish_with_message("Distributions computed.");

    // 6. Compute Info Gain & Rank (Parallelized)
    log::info!("Computing information gain and ranking candidates...");
    let prior_belief = vec![1.0 / NUM_CLUSTERS as f64; NUM_CLUSTERS];

    // Convert movie_dists to a Vec for parallel iteration
    let dists_vec: Vec<(i64, Vec<[f64; 10]>)> = movie_dists.into_iter().collect();

    // Parallel processing with Rayon
    let pb_gain = m.add(ProgressBar::new(dists_vec.len() as u64));
    pb_gain
        .set_style(ProgressStyle::default_bar().template("{spinner} {msg} {bar:40} {pos}/{len}")?);
    pb_gain.set_message("Calculating Entropy...");

    // We need thread-safe access to movie_rating_counts and movie_genres_map (read-only)
    // they are already shared referneces if we use par_iter

    let movie_rating_counts_ref = &movie_rating_counts;
    let movie_genres_map_ref = &movie_genres_map;
    let prior_belief_ref = &prior_belief;
    let pb_gain_ref = &pb_gain;

    let mut onboarding_movies: Vec<NewOnboardingMovie> = dists_vec
        .par_iter()
        .map(|(mid, counts)| {
            let mut probs = vec![[0.0; 10]; NUM_CLUSTERS];
            for k in 0..NUM_CLUSTERS {
                let total_cluster_ratings: f64 = counts[k].iter().sum();
                let denom = total_cluster_ratings + (10.0 * LAPLACE_SMOOTHING);
                for b in 0..10 {
                    probs[k][b] = (counts[k][b] + LAPLACE_SMOOTHING) / denom;
                }
            }

            let gain = cinematch_recommendation_engine::onboarding::expected_info_gain(
                prior_belief_ref,
                &probs,
            );
            let rating_count = *movie_rating_counts_ref.get(mid).unwrap_or(&0) as i32;

            pb_gain_ref.inc(1);

            let genre_ids = if let Some(genres) = movie_genres_map_ref.get(mid) {
                genres.iter().map(|&g| Some(g)).collect()
            } else {
                Vec::new()
            };

            NewOnboardingMovie {
                movie_id: *mid,
                info_gain: gain as f32,
                rating_dist: json!(probs),
                rating_count,
                genre_ids,
            }
        })
        .collect();

    pb_gain.finish_with_message("Entropy calculation complete.");

    // Sort and filter
    log::info!(
        "Sorting and filtering {} candidates...",
        onboarding_movies.len()
    );

    // Sort by info gain descending
    onboarding_movies.par_sort_unstable_by(|a, b| b.info_gain.partial_cmp(&a.info_gain).unwrap());

    // Take all valid candidates
    let final_movies: Vec<NewOnboardingMovie> = onboarding_movies;

    log::info!("Storing {} onboarding candidates...", final_movies.len());
    db.store_onboarding_movies(&final_movies).await?;

    log::info!("Done!");
    Ok(())
}

fn rating_to_bucket(r: f32) -> usize {
    let bucket = (r * 2.0).round() as isize - 1;
    bucket.clamp(0, 9) as usize
}
