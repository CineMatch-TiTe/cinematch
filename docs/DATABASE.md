# Database

## Overview

CineMatch uses **PostgreSQL 15** as its primary data store, accessed via the **Diesel ORM** (Rust). The schema contains 28 tables organized into five domains: users, movies, parties, recommendations, and scheduling.

## Entity-Relationship Diagram

```
┌──────────┐     ┌──────────────────┐     ┌─────────┐
│  users   │────┤  party_members   ├────│ parties │
│          │     └──────────────────┘     │         │
│ id (PK)  │     ┌──────────────────┐     │ id (PK) │
│ username │────┤  user_ratings    │     │ state   │
│ oneshot  │     └──────────────────┘     │ leader  │
└────┬─────┘     ┌──────────────────┐     └────┬────┘
     │          ┤  user_preferences │          │
     │           └──────────────────┘     ┌────▼──────┐
     │           ┌──────────────────┐     │party_codes│
     └──────────┤  external_accts  │     │ code (4ch)│
                 └──────────────────┘     └───────────┘
                                          ┌───────────┐
┌──────────┐     ┌──────────────────┐     │party_picks│
│  movies  │────┤  movie_genres   ├────│ votes     │
│          │     └──────────────────┘     └───────────┘
│ movie_id │     ┌──────────────────┐
│ title    │────┤  movie_cast     │
│ runtime  │     └──────────────────┘
│ rating   │     ┌──────────────────┐
└──────────┘────┤  movie_keywords  │
                 └──────────────────┘
```

## Tables by Domain

### Users

| Table | Primary Key | Description |
|-------|-------------|-------------|
| `users` | `id` (UUID) | User accounts. `oneshot` = guest user |
| `external_accounts` | `id` (UUID) | OAuth provider links (GitHub). FK → `users` |
| `user_preferences` | `user_id` (UUID) | Year target, flexibility, TiTe flag. FK → `users` |
| `user_ratings` | `rating_id` (UUID) | Movie taste data (liked/rating). FK → `users`, `movies` |
| `prefs_include_genre` | `(user_id, genre_id)` | Genre whitelist. FK → `users`, `genres` |
| `prefs_exclude_genre` | `(user_id, genre_id)` | Genre blacklist. FK → `users`, `genres` |

### Parties

| Table | Primary Key | Description |
|-------|-------------|-------------|
| `parties` | `id` (UUID) | Party state, leader, selected movie, voting round |
| `party_codes` | `code` (CHAR(4)) | 4-char join code → party mapping |
| `party_members` | `(user_id, party_id)` | Membership with `is_ready` flag |
| `party_picks` | `taste_id` (UUID) | Member movie picks with like/dislike |
| `votes` | `(party_id, user_id, movie_id)` | Individual votes (bool) |
| `shown_movies` | `(party_id, user_id, movie_id)` | Tracks which movies were shown to whom |

### Movies

| Table | Primary Key | Description |
|-------|-------------|-------------|
| `movies` | `movie_id` (INT8) | Core movie data (title, runtime, TMDB IDs, etc.) |
| `genres` | `genre_id` (UUID) | Genre catalog |
| `movie_genres` | `(movie_id, genre_id)` | Movie ↔ genre mapping |
| `cast_members` | `cast_id` (UUID) | Actor catalog |
| `movie_cast` | `(movie_id, cast_id)` | Movie ↔ actor mapping |
| `directors` | `director_id` (UUID) | Director catalog |
| `movie_directors` | `(movie_id, director_id)` | Movie ↔ director mapping |
| `keywords` | `keyword_id` (UUID) | Keyword/tag catalog |
| `movie_keywords` | `(movie_id, keyword_id)` | Movie ↔ keyword mapping |
| `trailers` | `trailer_id` (UUID) | Trailer video keys |
| `movie_trailers` | `(movie_id, trailer_id)` | Movie ↔ trailer mapping |
| `production_countries` | `country_code` (CHAR(3)) | Country catalog |
| `movie_production_countries` | `(movie_id, country_code)` | Movie ↔ country mapping |

### Recommendations

| Table | Primary Key | Description |
|-------|-------------|-------------|
| `onboarding_movies` | `movie_id` (INT8) | High-info-gain movies for new user onboarding |
| `onboarding_clusters` | `cluster_id` (INT2) | User taste clusters with centroids (JSONB) |

### Scheduling

| Table | Primary Key | Description |
|-------|-------------|-------------|
| `schedules` | `id` (UUID) | Timed events (voting start/end, watching end, ready timeouts) |

## Custom Types (PostgreSQL ENUMs)

| Type | Values | Used In |
|------|--------|---------|
| `auth_provider` | Provider identifiers | `external_accounts.provider` |
| `party_state` | `Created`, `Picking`, `Voting`, `Watching`, `Review`, `Disbanded` | `parties.state` |
| `timeout_type` | `VotingStarting`, `VotingEnding`, `WatchingEnding`, `ReadyTimeout` | `schedules.timeout_type` |

## Key Relationships

- A **user** can be in multiple **parties** (via `party_members`)
- Each **party** has exactly one **leader** (`party_leader_id` → `users`)
- **Party codes** are recycled — only active parties hold codes
- **Votes** are scoped to `(party, user, movie)` — one vote per movie per user per party
- **User ratings** (`user_ratings`) persist across parties and feed the recommendation engine
