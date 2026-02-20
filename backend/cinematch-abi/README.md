cinematch-abi
=============

[← Back to main README](../README.md)

Application Business Interface (ABI) layer. Intermediaries between HTTP handlers and the data layer, enforcing business rules and orchestrating cross-cutting concerns.

Structure
---------

```
src/
├── lib.rs          # AppState (Arc<Database> + Arc<WsRegistry> + Arc<Scheduler>)
├── prelude.rs      # Re-exports
├── domain/
│   ├── onboarding.rs    # OnboardingService + OnboardingCache (L1 RAM + L2 Redis)
│   ├── recommendation.rs # Recommendation domain model
│   ├── user.rs          # User domain extensions
│   ├── party/           # Party domain logic
│   └── error.rs         # Domain error types
├── scheduler/       # Async timeout scheduler for phase transitions
└── websocket/       # WsRegistry for managing WebSocket connections
```

Responsibilities
----------------

- **`AppState`**: Shared application state injected into Actix handlers (`Arc<Database>`, `Arc<WsRegistry>`, `Arc<Scheduler>`).
- **`OnboardingService`**: Manages onboarding sessions. Implements a 3-tier cache (RAM → Redis → Postgres) for candidate data.
- **`Recommendation`**: Recommendation strategy selection logic.
- **`Scheduler`**: Manages timeout-based phase transitions.
- **`WsRegistry`**: Registry of active WebSocket connections for party state broadcasting.

Architecture Constraints
------------------------

Handlers in `cinematch-server` MUST NOT access `cinematch-db` directly. All database interactions MUST traverse `cinematch-abi` domain types.
