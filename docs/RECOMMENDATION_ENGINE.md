# Recommendation Engine

## Overview

The `cinematch-recommendation-engine` crate provides ML-based movie recommendations using **Qdrant** vector search. It supports multiple strategies and generates voting ballots for the party flow.

## Vector Types

Movies are represented by four embedding types stored in Qdrant:

| Vector | Field | Source |
|--------|-------|--------|
| Plot | `plot_vector` | Movie plot/overview text embeddings |
| Cast & Crew | `cast_crew_vector` | Actor/director combination embeddings |
| Reviews | `reviews_vector` | Aggregated user review embeddings |
| Combined | `combined_vector` | Weighted blend of all vectors |

## Recommendation Strategies

### 1. Standard (`engine/standard.rs`)

The default strategy for personalized recommendations.

- Builds a user taste profile from their `user_ratings` (likes/dislikes)
- Queries Qdrant with the `combined_vector` for nearest-neighbor search
- Applies filters from user preferences (genre include/exclude, year range, runtime)
- Returns ranked movie IDs

### 2. Reviews-Based (`engine/reviews.rs`)

Uses review vectors for users with sufficient rating history.

- Computes a centroid from movies the user has liked
- Searches Qdrant's `reviews_vector` space
- Better for users with strong taste signals

### 3. Pool-Based (`engine/pool.rs`)

Party-scoped recommendations.

- Aggregates preferences from all party members
- Finds movies that satisfy the group's combined taste profile
- Used during the Picking phase to suggest movies the whole party might enjoy

## Ballot Generation

### Round 1 (`ballots/v1.rs`)

`build_voting_ballots_for_party()`:
- Takes all party picks (movies selected by members during Picking)
- Enriches with movie metadata
- Returns the ballot for the Voting phase

### Round 2 (`ballots/v2.rs`)

`build_round2_ballots_for_party()`:
- Narrows the ballot based on Round 1 results
- Eliminates movies with net-negative votes
- Creates a tighter ballot for the final vote

## Filter Utilities (`utils.rs`)

Qdrant filter builders that translate user preferences into vector search constraints:

- Genre inclusion/exclusion filters
- Year range filters
- Runtime range filters
- Already-shown movie exclusion (`shown_movies` table)

## Integration

The recommendation engine is called from `cinematch-server` handlers:

```
GET /api/recommend  →  recommendation::handlers::get_recommendations
                          → recommend_movies() / recommend_from_reviews()
```

During party phase transitions, ballot functions are called directly by the party handlers.

## Dependencies

- **Qdrant Client** (`qdrant-client 1.16`) — vector database queries
- **Linfa** (`linfa 0.8`) — ML toolkit for clustering (onboarding clusters)
- **ndarray** — numerical array operations
- **rayon** — parallel iteration for batch processing
