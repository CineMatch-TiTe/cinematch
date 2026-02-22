# Cinematch Recommendation Engine

Core algorithms for movie recommendations.

## Modules

- **`engine`**: Lower-level recommendation functions.
    - `standard`: Qdrant-based vector similarity (AverageVector strategy).
    - `reviews`: Collaborative filtering using sparse user-movie vectors.
    - `pool`: Recommendation restricted to a specific pool of movie IDs.
- **`ballots`**: Logic for party-based voting ballots.
    - `v1`: Initial weighted shuffle of party and personal pools.
    - `v2`: Round-2 top-3 refinement.

## Main API

| Function | Strategy | Description |
|----------|----------|-------------|
| `recommend_movies()` | Semantic | Qdrant `RecommendPoints` with average positive/negative seeds. |
| `recommend_from_reviews()` | Collaborative | Sparse user-movie vectors to find similar users. |
| `recommend_from_pool()` | Pool | Recommendation restricted to a specific list of IDs. |
| `build_voting_ballots_for_party()` | Round 1 | Constructs personalized ballots for party voting. |
| `build_round2_ballots_for_party()` | Round 2 | Refines voting to the top 3 candidates. |
