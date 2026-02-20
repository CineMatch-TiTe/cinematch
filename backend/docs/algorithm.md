Recommendation Algorithm
========================

[← Back to main README](../README.md)

Cinematch uses three recommendation strategies, selected automatically based on user context (solo vs. party).

Strategy Selection
------------------

```mermaid
flowchart TD
    REQ["/api/recommend"] --> METHOD{method param?}
    METHOD -->|"reviews"| REV[Review-Based: Collaborative Filtering]
    METHOD -->|"semantic"| SEM[Semantic: Vector Similarity]
    METHOD -->|default| SEM
    
    PARTY["Party Voting Phase"] --> BALLOT[Pool-Based: Ballot Builder]
```

1. Semantic Recommendation (Standard Strategy)
--------------------------------------------

Utilizes Qdrant's `RecommendPoints` API with the `AverageVector` strategy.

### Mechanism

```mermaid
flowchart LR
    subgraph "User Profile"
        POS[Liked Movies]
        NEG[Disliked Movies]
        PREFS[Preferences: Genres, Year, etc.]
    end

    subgraph "Qdrant"
        AVG["Average positive vectors - negative vectors"]
        FILT[Filters: Genre, Year, Exclude seen]
        NN[Nearest neighbors]
    end

    POS --> AVG
    NEG --> AVG
    AVG --> FILT
    PREFS --> FILT
    FILT --> NN
    NN --> RES[Ranked Movie IDs]
```

### Vector Types

Each movie has 4 named embedding vectors (1024-dim, generated via Ollama) bge-m3 model:

| Vector | Description |
|--------|-------------|
| `plot_vector` | Plot similarity |
| `cast_crew_vector` | Cast/Crew similarity |
| `reviews_vector` | Critical reception similarity |
| `combined_vector` | General purpose (default) |

### Fallback

If no positive ratings exist, the engine uses the top 5 most popular movies as positive seeds.

2. Review-Based Recommendation (Collaborative Filtering)
------------------------------------------------------

**Condition**: User has explicitly requested `method=reviews`.

Uses sparse user-movie vectors in Qdrant's `ratings` collection to identify similar users.

### Mechanism

```mermaid
flowchart TD
    A[Build sparse vector from user ratings: liked=+1.0, disliked=-1.0] --> B[Query 'ratings' collection: Find 200 similar users]
    B --> C[Aggregate movie scores from similar users' profiles]
    C --> D[Filter by preferences: Genre, year, exclusions]
    D --> E[Return top N movies]
    
    E -->|Empty?| F[Fallback to Semantic Strategy]
```

### Sparse Vector Format

```
user_vector = { movie_1: 1.0, movie_5: -1.0, movie_12: 1.0, ... }
```

Matches users with similar rating patterns and recommends movies liked by those users.

3. Pool-Based Recommendation (Party Voting)
-------------------------------------------

**Condition**: Party transitions from Picking → Voting phase.

Constructs personalized voting ballots for each party member from the shared pool of picked movies.

### Round 1: Initial Ballot

```mermaid
flowchart TD
    subgraph "Inputs"
        PP[Party Pool: All liked picks from all members]
        OP[Own Pool: User's own liked picks]
    end

    PP -->|Recommend 3| REC1[Qdrant recommend_from_pool]
    OP -->|Recommend 2| REC2[Qdrant recommend_from_pool]
    REC1 --> MERGE[Merge + Deduplicate]
    REC2 --> MERGE
    MERGE --> SHUFFLE[Shuffle]
    SHUFFLE --> PAD{< 5 movies?}
    PAD -->|Yes| POP[Pad from Popular Movies]
    PAD -->|No| BALLOT[Final 5-Movie Ballot]
    POP --> BALLOT
```

- **3 from party pool**: Group favorites, ranked by personal preference.
- **2 from own pool**: Personal favorites, for diversity.
- Shuffled and padded to exactly 5 movies per ballot.

### Round 2: Top-3 Refinement

After Round 1 tallying, determining the top 3 movies. Each member receives a new ballot of 3 movies from this subset.

Pipeline: Data Ingestion to Recommendation
------------------------------------------

```mermaid
flowchart LR
    subgraph "Offline (Importer CLI)"
        CSV[MovieLens CSVs] --> PARSE[Parse & Clean]
        PARSE --> EMB[Generate Embeddings via Ollama]
        EMB --> UPLOAD[Upload to Qdrant: 4 vectors per movie]
        PARSE --> PG_INS[Insert into Postgres: Movies, Genres, Cast]
        
        CSV --> SPARSE[Build sparse user vectors from ratings]
        SPARSE --> QDRANT_RAT[Upload to Qdrant: 'ratings' collection]
    end

    subgraph "Online (Server)"
        REQ2[API Request] --> STRAT[Strategy Selection]
        STRAT --> QD2[(Qdrant)]
        STRAT --> PG2[(PostgreSQL)]
        STRAT --> RD2[(Redis Cache)]
    end

    UPLOAD --> QD2
    PG_INS --> PG2
    QDRANT_RAT --> QD2
```

### Importer Commands

| Command | Description |
|---------|-------------|
| `update-all` | Runs `update-movies` + `update-ratings` |
| `update-movies` | CSV → Ollama embeddings → Qdrant `movies` + Postgres |
| `update-ratings` | CSV → Sparse vectors → Qdrant `ratings` collection |
| `remove-all` | Wipe all Qdrant collections |
