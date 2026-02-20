Cinematch Backend
=================

Rust-based movie recommendation and watch-party platform. Users create parties, swipe on movies, vote collectively, and watch as a group — supported by multi-strategy ML recommendations.

Architecture Overview
---------------------

```mermaid
graph TB
    subgraph Clients
        FE[Next.js Frontend]
    end

    subgraph "cinematch-server (Actix-web)"
        API[REST API]
        WS[WebSocket]
    end

    subgraph "Domain Layer"
        ABI[cinematch-abi]
        REC[cinematch-recommendation-engine]
    end

    subgraph "Data Layer"
        DB[cinematch-db]
    end

    subgraph "Storage"
        PG[(PostgreSQL)]
        RD[(Redis)]
        QD[(Qdrant)]
    end

    subgraph "Offline Pipeline"
        IMP[cinematch-importer]
        OL[Ollama LLM]
    end

    FE -->|HTTP / WS| API
    FE -->|WebSocket| WS
    API --> ABI
    WS --> ABI
    ABI --> REC
    ABI --> DB
    REC --> DB
    DB --> PG
    DB --> RD
    DB --> QD
    IMP -->|CSV + Embeddings| DB
    IMP --> OL
```

Crate Map
---------

| Crate | Description | Documentation |
|-------|-------------|---------------|
| [`cinematch-server`](cinematch-server/) | HTTP API + WebSocket server. | [README](cinematch-server/README.md) |
| [`cinematch-abi`](cinematch-abi/) | Domain logic, app state, scheduler. | [README](cinematch-abi/README.md) |
| [`cinematch-recommendation-engine`](cinematch-recommendation-engine/) | ML recommendation algorithms. | [README](cinematch-recommendation-engine/README.md) |
| [`cinematch-db`](cinematch-db/) | Database access (Postgres, Redis, Qdrant). | [README](cinematch-db/README.md) |
| [`cinematch-common`](cinematch-common/) | Shared types, config, models. | [README](cinematch-common/README.md) |
| [`cinematch-importer`](cinematch-importer/) | Offline data pipeline CLI. | [README](cinematch-importer/README.md) |

Databases
---------

| Database | Port | Description |
|----------|------|-------------|
| **PostgreSQL 15** | 5432 | Primary relational data (users, parties, movies, votes, ratings). |
| **Redis 7** | 6379 | Session store, onboarding state, candidate caching. |
| **Qdrant** | 6333/6334 | Vector similarity search (4 named embedding vectors per movie). |

For schema details, see [docs/databases.md](docs/databases.md).

Recommendation Algorithm
------------------------

The engine implements multiple strategies contingent on user context. For details, see [docs/algorithm.md](docs/algorithm.md).

Quick Start
-----------

```bash
# 1. Start databases
docker compose up -d

# 2. Import movie data (requires Ollama running on port 11434)
cargo run -p cinematch-importer -- update-all

# 3. Run the server
cargo run -p cinematch-server
```

The API server starts on `http://localhost:8085`. Swagger UI is available at `/swagger-ui/`.
