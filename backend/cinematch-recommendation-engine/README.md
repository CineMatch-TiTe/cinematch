cinematch-recommendation-engine
===============================

[← Back to main README](../README.md)

Core recommendation algorithms and logic.

For detailed architecture, see [docs/algorithm.md](../docs/algorithm.md).

API Reference
-------------

| Function | Strategy | Description |
|----------|----------|-------------|
| `recommend_movies()` | Semantic | Qdrant vector similarity using liked/disliked movies as seeds. |
| `recommed_movies_from_reviews()` | Collaborative | Sparse user-movie vectors to find similar users. |
| `recommend_from_pool()` | Pool-based | Recommend from a constrained set of movie IDs. |
| `build_voting_ballots_for_party()` | Voting R1 | Build 5-movie ballots (3 party + 2 personal). |
| `build_round2_ballots_for_party()` | Voting R2 | Build 3-movie ballots from top-3 finalists. |

Onboarding Logic (`onboarding.rs`)
----------------------------------

Implements entropy-based information gain maximization.

| Function | Description |
|----------|-------------|
| `bayesian_update()` | Updates cluster belief distribution given a swipe action. |
| `expected_info_gain()` | Computes Shannon entropy reduction for a candidate movie. |
| `pick_best_movie()` | Selects the single highest-info-gain candidate. |
| `pick_best_movies()` | Selects top N candidates (parallel implementation). |
| `belief_to_genre_preferences()` | Converts final belief into a genre preference vector. |

Dependencies
------------

- `qdrant-client`
- `rayon`
- `linfa` / `ndarray`
