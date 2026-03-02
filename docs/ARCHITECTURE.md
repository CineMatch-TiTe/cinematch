# Architecture

## System Overview

CineMatch is a full-stack application built as a monorepo with two independent deployables:

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Browser                            │
│   Next.js 16 (React 19)  ←──HTTP/WS──→  Rust Backend (Actix)   │
└───────┬──────────────────────────────────────────┬──────────────┘
        │ SSR + Client                             │
        │                                          │
   ┌────▼────┐    ┌──────────┐    ┌──────────┐  ┌──▼──────┐
   │ Next.js │    │ Postgres │    │  Redis   │  │ Qdrant  │
   │  Proxy  │    │   15     │    │   7      │  │ Vector  │
   └─────────┘    └──────────┘    └──────────┘  └─────────┘
```

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Frontend | Next.js 16, React 19, TypeScript | UI, routing, SSR |
| Backend | Rust, Actix Web 4 | REST API, WebSocket, business logic |
| Database | PostgreSQL 15 | Persistent relational storage |
| Cache/Sessions | Redis 7 | Session management, real-time state |
| Vector DB | Qdrant | Movie embeddings for recommendation engine |
| LLM (optional) | Ollama | GPU-accelerated inference (dev) |

## Request Flow

1. The **browser** loads the Next.js application on port `3000`.
2. API calls from the frontend hit `/api/*` routes, which Next.js **rewrites** (proxies) to the backend at `NEXT_PUBLIC_API_BASE` (default `https://api.cinematch.space`).
3. The **Rust backend** authenticates via JWT cookies, processes the request, and interacts with PostgreSQL (via Diesel ORM), Redis, and Qdrant as needed.
4. **WebSocket connections** are established at `/api/ws` for real-time party state updates. The backend uses an actor-based `WsRegistry` to broadcast messages to party members.

## Backend Crate Architecture

The backend is a **Cargo workspace** with six crates:

```
backend/
├── cinematch-server          # HTTP server, routes, handlers, WebSocket
├── cinematch-db              # Diesel schema, domain models, repositories
├── cinematch-common          # Shared config, API models, enums
├── cinematch-abi             # WebSocket session actor, WsRegistry
├── cinematch-recommendation-engine  # ML algorithms, Qdrant queries
└── cinematch-importer        # CLI tool for data ingestion
```

| Crate | Depends On | Purpose |
|-------|-----------|---------|
| `cinematch-server` | all others | Actix Web server, routes, handlers |
| `cinematch-db` | — | Diesel ORM, schema, connection pools |
| `cinematch-common` | — | Config, shared types (`PartyState`, `VectorType`, etc.) |
| `cinematch-abi` | `common`, `db` | WebSocket actor sessions, WsRegistry |
| `cinematch-recommendation-engine` | `common`, `db` | Recommendation strategies, ballot building |
| `cinematch-importer` | `common`, `db` | TMDB data ingestion CLI |

## Frontend Architecture

```
frontend/src/
├── app/              # Next.js App Router pages
│   ├── page.tsx              # Home — join/create party
│   ├── create-party/         # Party creation flow
│   ├── dashboard/            # User dashboard
│   ├── party-room/           # Main party experience
│   └── preferences/          # User taste preferences
├── components/       # React components by domain
│   ├── party/                # Party UI (voting, picking, lobby)
│   ├── preferences/          # Preference forms
│   ├── dashboard/            # Dashboard widgets
│   ├── ui/                   # shadcn/ui primitives
│   └── ...
├── hooks/            # Custom hooks (voting, movie picker, etc.)
├── server/           # Server actions by domain
├── model/            # Orval-generated API client (from OpenAPI)
└── lib/              # Utilities
```

Key choices:
- **SWR** for data fetching with cache-based revalidation
- **Orval** auto-generates a typed API client from the backend's OpenAPI spec
- **Radix UI** primitives with **Tailwind CSS v4** for styling
- **React Compiler** (`babel-plugin-react-compiler`) enabled

## Authentication Flow

```
Guest Login              GitHub OAuth
    │                        │
    ▼                        ▼
POST /auth/login/guest   GET /auth/github/login
    │                        │
    ▼                        ▼
  JWT cookie set          GitHub callback → JWT cookie set
    │                        │
    ▼                        ▼
  Authenticated session (cookie: "id")
```

- **Guest**: instant creation, generates a random or user-provided username. The user is marked `oneshot = true`.
- **GitHub OAuth**: optional, requires `GITHUB_CLIENT_ID` and `GITHUB_CLIENT_SECRET` env vars. Links an `external_accounts` record to the user.
- **JWT** is signed with `SECRET_TOKEN`, expires after `JWT_EXPIRY_SECS` (default 3600s). Can be renewed via `POST /auth/renew`.

## Party Lifecycle (State Machine)

```
Created → Picking → Voting → Watching → Review → (new round or Disbanded)
                      ↑                    │
                      └────────────────────┘
```

| State | Description |
|-------|-------------|
| `Created` | Lobby — members join via 4-char code, set ready |
| `Picking` | Members browse/search movies, each picks favorites |
| `Voting` | Ballot distributed, members vote like/dislike |
| `Watching` | Winning movie selected, timer runs |
| `Review` | Post-watch — leader can start a new round |
| `Disbanded` | Permanent end, retained for history |

Phase transitions happen automatically (when all members ready) or manually (leader force-advance). Configurable timeouts control auto-transitions.
