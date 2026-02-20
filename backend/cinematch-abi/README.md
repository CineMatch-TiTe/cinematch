cinematch-abi
=============

[← Back to main README](../README.md)

Application Business Interface (ABI) layer. Intermediaries between HTTP handlers and the data layer, enforcing business rules and orchestrating cross-cutting concerns.

Structure
---------

```
src/
├── lib.rs           # AppState (AppContext implementation)
├── prelude.rs       # Re-exports
├── domain/
│   ├── recommendation.rs # Recommendation domain model (facade)
│   ├── user.rs          # User domain extensions
│   ├── party/           # Party domain logic & State Machine
│   └── error.rs         # Domain error types
├── scheduler/        # Async timeout scheduler for phase transitions
└── websocket/        # WsRegistry for managing WebSocket connections
```

Responsibilities
----------------

- **`AppState`**: Shared application state injected into Actix handlers. Implements `AppContext` to provide unified access to `Database`, `WsRegistry`, and `Scheduler`.
- **`Recommendation`**: Facade for the `cinematch-recommendation-engine`. Handles strategy selection logic.
- **`Scheduler`**: Manages timeout-based phase transitions (e.g., auto-advancing from Picking to Voting).
- **`WsRegistry`**: Registry of active WebSocket connections for party state broadcasting.

Architecture Constraints
------------------------

Handlers in `cinematch-server` MUST NOT access `cinematch-db` directly. All database interactions MUST traverse `cinematch-abi` domain types or extension traits.
