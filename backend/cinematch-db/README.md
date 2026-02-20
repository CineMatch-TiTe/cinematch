cinematch-db
============

[← Back to main README](../README.md)

Database access layer for PostgreSQL, Redis, and Qdrant. Provides repository abstractions and lazy-loading domain types.

For schema details, see [docs/databases.md](../docs/databases.md).

Structure
---------

```
src/
├── lib.rs          # Database (PG pool + Redis pool + Qdrant client)
├── schema.rs       # Diesel schema
├── models.rs       # Re-exports
├── conn/
│   ├── postgres/   # PG connection helpers
│   ├── redis/      # Redis utilities
│   └── qdrant/     # QdrantService
├── repo/           # Repository layer
│   ├── movie/      # Movie CRUD + search
│   ├── party/      # Party CRUD
│   ├── user/       # User CRUD
│   ├── vote/       # Vote operations
│   ├── taste/      # User taste profile
│   └── schedules/  # Timeout schedules
├── domain/         # Domain types (Extension traits)
│   ├── party.rs    # Party entity logic
│   ├── user.rs     # User entity logic
│   └── movie.rs    # Movie entity logic
└── prelude.rs      # Imports
```

Database Struct
---------------

```rust
pub struct Database {
    pub pool: Pool<AsyncPgConnection>,
    pub redis: RedisPool,
    pub vector: QdrantService,
}
```

Exposes `conn()` and `redis_conn()` for connection acquisition, and `run_migrations()` for startup initialization.

Architecture Constraints
------------------------

- **Raw SQL** MUST be encapsulated within `repo/` modules.
- **No direct DB calls** permitted from `cinematch-server` handlers; `cinematch-abi` domain types MUST be used.
- **Migrations** are embedded via Diesel's `embed_migrations!`.
