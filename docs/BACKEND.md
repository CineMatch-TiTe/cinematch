# Backend

## Overview

The backend is a Rust workspace with six crates that compile into two binaries: the **API server** and the **data importer CLI**.

## Crate Details

### `cinematch-server`

The main HTTP server built with **Actix Web 4**. Entry point: `main.rs`.

**Modules:**

| Module | Path | Purpose |
|--------|------|---------|
| `auth` | `auth/` | Guest login, GitHub OAuth, JWT guard middleware |
| `party` | `party/` | CRUD, picks, votes, leader ops, user ops |
| `movie` | `movie/` | Search, get movie details, list genres |
| `user` | `user/` | Profile, rename, taste, preferences |
| `recommendation` | `recommendation/` | Proxy to recommendation engine |
| `websocket` | `websocket.rs` | WS upgrade, broadcast helpers |
| `handlers` | `handlers/` | System endpoints (version) |
| `routes` | `routes.rs` | Route registration and scoping |

**Route Groups:**

| Prefix | Scope |
|--------|-------|
| `/api/auth` | Authentication (guest, GitHub, JWT renewal) |
| `/api/user` | User profile and preferences |
| `/api/party` | Party CRUD, membership, picks, votes, leader actions |
| `/api/movie` | Movie search and metadata |
| `/api/recommend` | Personalized recommendations |
| `/api/ws` | WebSocket upgrade |
| `/api/system` | Version info |

### `cinematch-db`

Database layer using **Diesel ORM** with PostgreSQL.

- `schema.rs` — auto-generated Diesel schema (28 tables)
- `domain/` — domain objects with business logic (e.g., `Party`, `User`)
- `repo/` — repository pattern implementations (19 modules)
- `conn/` — connection pool management (PostgreSQL, Redis, Qdrant)

### `cinematch-common`

Shared types and centralized configuration.

- `config.rs` — `LazyLock`-based env var reader for all services
- `models/` — API types: `PartyState`, `VectorType`, `SwipeAction`, `SearchFilter`, `ErrorResponse`
- `models/websocket/` — WebSocket server message types

### `cinematch-abi`

WebSocket session actor and registry.

- `WsSession` — per-connection actor handling ping/pong and message forwarding
- `WsRegistry` — in-memory map of user → WebSocket connections, exposes `send_to_users()`

### `cinematch-recommendation-engine`

ML algorithms for movie recommendations. Three strategies:

| Strategy | Module | Input |
|----------|--------|-------|
| Standard | `engine/standard.rs` | User preferences + Qdrant combined vectors |
| Reviews | `engine/reviews.rs` | User ratings → Qdrant review vectors |
| Pool | `engine/pool.rs` | Party-scoped pool of recommendations |

Also provides:
- `ballots/v1.rs` — round 1 voting ballot generation
- `ballots/v2.rs` — round 2 narrowed ballot
- `utils.rs` — Qdrant filter builders

### `cinematch-importer`

CLI tool for ingesting movie data (TMDB) into PostgreSQL and generating Qdrant embeddings.

## Authentication Middleware

The `Auth` guard extracts the JWT from the `id` cookie, validates it, and injects the authenticated `user_id` into handler arguments. Routes that need auth simply add `auth: Auth` as a parameter.

## API Documentation

The backend auto-generates OpenAPI specs via `utoipa`. Four documentation UIs are served:

- **Swagger UI** — `/swagger-ui/`
- **ReDoc** — `/redoc`
- **Scalar** — `/scalar`
- **RapiDoc** — `/rapidoc`
