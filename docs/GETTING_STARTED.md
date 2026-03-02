# Getting Started

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | stable (2024 edition) | Backend compilation |
| Node.js | 18+ | Frontend runtime |
| pnpm | latest | Frontend package manager |
| Docker & Docker Compose | latest | Database and service containers |

## Quick Start

### 1. Clone the Repository

```sh
git clone https://github.com/CineMatch-TiTe/react_hackathon_2026.git
cd react_hackathon_2026
```

### 2. Start Backend Services

```sh
cd backend
cp .env.example .env    # Edit values as needed
docker-compose -f docker-compose.dev.yml up --build
```

This starts:
- **Rust API server** on port `8085`
- **PostgreSQL 15** on port `5432`
- **Redis 7** on port `6379`
- **Qdrant** on ports `6333` (HTTP) and `6334` (gRPC)
- **Ollama** on port `11434` (requires NVIDIA GPU)

### 3. Start Frontend

```sh
cd frontend
cp .env.example .env    # Set NEXT_PUBLIC_API_BASE if needed
pnpm install
pnpm dev
```

The frontend runs on http://localhost:3000.

### 4. Verify

| Service | URL |
|---------|-----|
| Frontend | http://localhost:3000 |
| Backend API | http://localhost:8085 |
| Swagger UI | http://localhost:8085/swagger-ui/ |
| ReDoc | http://localhost:8085/redoc |
| Scalar | http://localhost:8085/scalar |
| RapiDoc | http://localhost:8085/rapidoc |

## Environment Variables

### Backend (Required)

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | — (required) |
| `REDIS_URL` | Redis connection string | — (required) |
| `QDRANT_URL` | Qdrant gRPC endpoint | — (required) |
| `SECRET_TOKEN` | JWT/session signing key (≥ 64 bytes) | — (required) |

### Backend (Optional)

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_HOST` | Bind address | `0.0.0.0` |
| `SERVER_PORT` | HTTP port | `8080` |
| `JWT_EXPIRY_SECS` | Token lifetime (seconds) | `3600` |
| `RUST_LOG` | Log level | `info` |
| `GITHUB_CLIENT_ID` | GitHub OAuth app ID | — (disables OAuth) |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth secret | — |
| `VOTING_R1_TIMEOUT_SECS` | Round 1 voting timeout | `30` |
| `VOTING_R2_TIMEOUT_SECS` | Round 2 voting timeout | `20` |
| `WATCHING_TIMEOUT_SECS` | Watching phase timeout | `900` |
| `READY_COUNTDOWN_SECS` | All-ready countdown | `5.0` |

### Frontend

| Variable | Description | Default |
|----------|-------------|---------|
| `NEXT_PUBLIC_API_BASE` | Backend API base URL | `https://api.cinematch.space` |

## Development Tips

- **API client regeneration:** run `pnpm orval` in `frontend/` after backend OpenAPI changes.
- **Database migrations:** managed by Diesel CLI — run `diesel migration run` inside `backend/`.
- **Hot reload:** the Rust server must be rebuilt manually; the Next.js frontend supports HMR.
