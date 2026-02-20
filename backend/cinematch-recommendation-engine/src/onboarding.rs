//! Onboarding entropy-based movie selection.
//!
//! Implements the adaptive onboarding algorithm inspired by 3Blue1Brown's Wordle solver:
//! - Bayesian belief updates over taste clusters
//! - Information-gain-based movie selection
//! - Binary (like/dislike) input mapped to full rating distributions

use serde::{Deserialize, Serialize};

/// Number of rating buckets (0.5, 1.0, 1.5, ..., 5.0)
pub const NUM_BUCKETS: usize = 10;

use cinematch_common::models::SwipeAction;

/// A candidate movie with its conditional rating distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingCandidate {
    pub movie_id: i64,
    /// P(rating_bucket | cluster_k) — shape: [num_clusters][NUM_BUCKETS]
    pub rating_dist: Vec<[f64; NUM_BUCKETS]>,
    /// Popularity score for tie-breaking
    pub popularity: f32,
    /// Release year for filtering
    pub release_year: Option<i32>,
    /// Genre IDs for filtering
    pub genre_ids: Vec<Option<uuid::Uuid>>,
}

/// Perform a Bayesian update on the belief distribution given a swipe action.
///
/// - `belief`: current P(cluster_k) distribution (will be modified in place)
/// - `rating_dist`: P(rating_bucket | cluster_k) for the rated movie — shape [K][10]
/// - `action`: the user's swipe (like or dislike; skip is a no-op)
///
/// "Like" marginalizes over high rating buckets (≥ 3.5, i.e. buckets 6..10),
/// "Dislike" marginalizes over low rating buckets (< 3.5, i.e. buckets 0..6).
pub fn bayesian_update(
    belief: &mut [f64],
    rating_dist: &[[f64; NUM_BUCKETS]],
    action: SwipeAction,
) {
    if action == SwipeAction::Skip {
        return;
    }

    let k = belief.len();
    assert_eq!(
        rating_dist.len(),
        k,
        "rating_dist must have one row per cluster"
    );

    // Bucket indices: 0=0.5, 1=1.0, ..., 5=3.0, 6=3.5, 7=4.0, 8=4.5, 9=5.0
    // Like  = buckets 6..10 (ratings ≥ 3.5)
    // Dislike = buckets 0..6 (ratings < 3.5)
    let (range_start, range_end) = match action {
        SwipeAction::Like => (6, NUM_BUCKETS),
        SwipeAction::SuperLike => (8, NUM_BUCKETS), // Stronger signal: ratings >= 4.5
        SwipeAction::Dislike => (0, 6),
        SwipeAction::Skip => unreachable!(), // skip doesnt update the belief, it just means we show another movie
    };

    // Compute P(action | cluster_k) for each cluster
    let mut total = 0.0;
    for i in 0..k {
        let p_action_given_cluster: f64 = rating_dist[i][range_start..range_end].iter().sum();
        // Clamp to avoid zero probability (Laplace-like smoothing)
        let p = p_action_given_cluster.max(1e-10);
        belief[i] *= p;
        total += belief[i];
    }

    // Normalize
    if total > 0.0 {
        for b in belief.iter_mut() {
            *b /= total;
        }
    }
}

/// Compute the Shannon entropy of a probability distribution.
fn entropy(dist: &[f64]) -> f64 {
    dist.iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.log2())
        .sum()
}

/// Compute the expected information gain of showing a movie to a user.
///
/// I(m) = H(belief) - E[H(belief | action)]
///      = H(belief) - Σ_a P(a|belief,m) × H(belief_after_a)
///
/// where a ∈ {like, dislike}.
pub fn expected_info_gain(belief: &[f64], rating_dist: &[[f64; NUM_BUCKETS]]) -> f64 {
    let k = belief.len();
    let h_prior = entropy(belief);

    // For each possible action (like, dislike), compute:
    // 1. P(action | current belief) = Σ_k P(cluster_k) × P(action | cluster_k)
    // 2. posterior belief after observing that action
    // 3. H(posterior)
    let actions = [(6usize, NUM_BUCKETS), (0usize, 6)]; // like, dislike ranges

    let mut expected_posterior_entropy = 0.0;

    for &(range_start, range_end) in &actions {
        // P(action) marginalized over clusters
        let p_action: f64 = (0..k)
            .map(|i| {
                let p_action_cluster: f64 = rating_dist[i][range_start..range_end].iter().sum();
                belief[i] * p_action_cluster.max(1e-10)
            })
            .sum();

        if p_action < 1e-15 {
            continue;
        }

        // Compute posterior belief if this action were observed
        let posterior: Vec<f64> = (0..k)
            .map(|i| {
                let p_action_cluster: f64 = rating_dist[i][range_start..range_end].iter().sum();
                (belief[i] * p_action_cluster.max(1e-10)) / p_action
            })
            .collect();

        expected_posterior_entropy += p_action * entropy(&posterior);
    }

    h_prior - expected_posterior_entropy
}

/// Pick the candidate movie with the highest expected information gain.
///
/// Returns the index into `candidates` and the info gain value.
/// Returns `None` if `candidates` is empty.
pub fn pick_best_movie(belief: &[f64], candidates: &[OnboardingCandidate]) -> Option<(usize, f64)> {
    candidates
        .iter()
        .enumerate()
        .map(|(idx, c)| {
            let ig = expected_info_gain(belief, &c.rating_dist);
            (idx, ig)
        })
        .max_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    // Start with lower ID if gains are equal?
                    // No, max_by wants the "greater" element.
                    // If we want deterministic choice, we should prefer smaller ID?
                    // To prefer smaller ID, we treat smaller ID as "greater" in this comparison context?
                    // Actually, let's just reverse cmp of IDs.
                    let id_a = candidates[a.0].movie_id;
                    let id_b = candidates[b.0].movie_id;
                    id_b.cmp(&id_a)
                })
        })
}

/// Pick the top N candidate movies with the highest expected information gain.
///
/// Returns a list of (index, info_gain) tuples, sorted by info gain descending.
pub fn pick_best_movies(
    belief: &[f64],
    candidates: &[OnboardingCandidate],
    n: usize,
) -> Vec<(usize, f64)> {
    use rayon::prelude::*;

    let mut scored: Vec<(usize, f64)> = candidates
        .par_iter()
        .enumerate()
        .map(|(idx, c)| {
            let ig = expected_info_gain(belief, &c.rating_dist);
            (idx, ig)
        })
        .collect();

    // Sort descending by info gain, then by popularity descending, then by movie_id ascending
    scored.par_sort_unstable_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                // Secondary sort: Popularity descending
                let pop_a = candidates[a.0].popularity;
                let pop_b = candidates[b.0].popularity;
                pop_b
                    .partial_cmp(&pop_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                // Tertiary sort: movie_id ascending for stability
                let id_a = candidates[a.0].movie_id;
                let id_b = candidates[b.0].movie_id;
                id_a.cmp(&id_b)
            })
    });

    scored.into_iter().take(n).collect()
}

/// Convert a final belief distribution into a genre preference vector.
///
/// Multiplies the belief weights by the cluster centroids to get a
/// weighted-average genre preference vector for the user.
///
/// centroids: [K][num_genres] — the cluster centroids from K-Means.
/// Returns: [num_genres] — the user's estimated genre preferences.
pub fn belief_to_genre_preferences(belief: &[f64], centroids: &[Vec<f64>]) -> Vec<f64> {
    let k = belief.len();
    assert_eq!(centroids.len(), k);

    let num_genres = centroids[0].len();
    let mut result = vec![0.0; num_genres];

    for (i, &weight) in belief.iter().enumerate() {
        for (j, &val) in centroids[i].iter().enumerate() {
            result[j] += weight * val;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dist_2clusters() -> Vec<[f64; NUM_BUCKETS]> {
        // Cluster 0: loves this movie (high ratings)
        // Cluster 1: hates this movie (low ratings)
        vec![
            // cluster 0: 90% rates ≥3.5
            [0.01, 0.01, 0.01, 0.01, 0.01, 0.05, 0.10, 0.20, 0.30, 0.30],
            // cluster 1: 90% rates <3.5
            [0.15, 0.20, 0.20, 0.15, 0.10, 0.10, 0.03, 0.03, 0.02, 0.02],
        ]
    }

    #[test]
    fn test_bayesian_update_like() {
        let mut belief = vec![0.5, 0.5];
        let dist = make_dist_2clusters();

        bayesian_update(&mut belief, &dist, SwipeAction::Like);

        // After liking, cluster 0 (which loves this movie) should dominate
        assert!(
            belief[0] > 0.8,
            "cluster 0 should dominate after like: {:?}",
            belief
        );
        assert!(
            (belief[0] + belief[1] - 1.0).abs() < 1e-10,
            "should sum to 1"
        );
    }

    #[test]
    fn test_bayesian_update_dislike() {
        let mut belief = vec![0.5, 0.5];
        let dist = make_dist_2clusters();

        bayesian_update(&mut belief, &dist, SwipeAction::Dislike);

        // After disliking, cluster 1 (which hates this movie) should dominate
        assert!(
            belief[1] > 0.8,
            "cluster 1 should dominate after dislike: {:?}",
            belief
        );
    }

    #[test]
    fn test_bayesian_update_skip() {
        let mut belief = vec![0.5, 0.5];
        let dist = make_dist_2clusters();

        bayesian_update(&mut belief, &dist, SwipeAction::Skip);

        // Skip should not change belief
        assert_eq!(belief, vec![0.5, 0.5]);
    }

    #[test]
    fn test_info_gain_polarizing_movie() {
        let belief = vec![0.5, 0.5];

        // Polarizing movie: clusters disagree strongly
        let polarizing = make_dist_2clusters();

        // Boring movie: all clusters rate similarly
        let boring = vec![
            [0.05, 0.05, 0.10, 0.10, 0.10, 0.10, 0.10, 0.15, 0.15, 0.10],
            [0.05, 0.05, 0.10, 0.10, 0.10, 0.10, 0.10, 0.15, 0.15, 0.10],
        ];

        let ig_polarizing = expected_info_gain(&belief, &polarizing);
        let ig_boring = expected_info_gain(&belief, &boring);

        assert!(
            ig_polarizing > ig_boring,
            "polarizing movie should have higher info gain: {} vs {}",
            ig_polarizing,
            ig_boring
        );
    }

    #[test]
    fn test_pick_best_movie() {
        let belief = vec![0.5, 0.5];
        let candidates = vec![
            OnboardingCandidate {
                movie_id: 1,
                rating_dist: vec![
                    [0.05, 0.05, 0.10, 0.10, 0.10, 0.10, 0.10, 0.15, 0.15, 0.10],
                    [0.05, 0.05, 0.10, 0.10, 0.10, 0.10, 0.10, 0.15, 0.15, 0.10],
                ], // boring
                popularity: 10.0,
                release_year: Some(2020),
                genre_ids: vec![],
            },
            OnboardingCandidate {
                movie_id: 42,
                rating_dist: make_dist_2clusters(), // polarizing
                popularity: 5.0,
                release_year: Some(2021),
                genre_ids: vec![],
            },
        ];

        let (idx, _ig) = pick_best_movie(&belief, &candidates).unwrap();
        assert_eq!(idx, 1, "should pick the polarizing movie (index 1)");
        assert_eq!(candidates[idx].movie_id, 42);
    }

    #[test]
    fn test_belief_to_genre_preferences() {
        let belief = vec![0.7, 0.3];
        let centroids = vec![
            vec![4.5, 2.0, 3.0], // cluster 0: loves action, dislikes comedy
            vec![2.0, 4.5, 3.5], // cluster 1: loves comedy, dislikes action
        ];

        let prefs = belief_to_genre_preferences(&belief, &centroids);

        // Expected: 0.7*4.5 + 0.3*2.0 = 3.75, 0.7*2.0 + 0.3*4.5 = 2.75, ...
        assert!((prefs[0] - 3.75).abs() < 1e-10);
        assert!((prefs[1] - 2.75).abs() < 1e-10);
    }
}
