# Deployment

## Docker Compose (Development)

The development stack is defined in `backend/docker-compose.dev.yml`:

```yaml
services:
  cinematch-server    # Rust API (port 8085)
  cinematch-db        # PostgreSQL 15 (port 5432)
  cinematch-redis     # Redis 7 (port 6379)
  qdrant              # Qdrant vector DB (ports 6333, 6334)
  ollama              # Ollama LLM server (port 11434, GPU)
```

### Networks

| Network | Type | Purpose |
|---------|------|---------|
| `db_network` | bridge, internal | Database connectivity (no external access) |
| `internet` | bridge | External access for API server |

### Volumes

| Volume | Mounted To |
|--------|-----------|
| `cinematch_db_data` | PostgreSQL data directory |
| `cinematch_redis_data` | Redis persistence |
| `qdrant_storage` | Qdrant vector storage |
| `ollama_data` | Ollama model cache |

### Start Development Stack

```sh
cd backend
docker-compose -f docker-compose.dev.yml up --build
```

## Dockerfiles

### Backend (`backend/Dockerfile`)

Multi-stage Rust build:
1. **Build stage** — compiles release binary with cargo
2. **Runtime stage** — minimal image with the compiled binary

### Frontend (`frontend/Dockerfile`)

Multi-stage Next.js build:
1. **Dependencies** — installs node_modules
2. **Build** — runs `next build` with `standalone` output
3. **Runtime** — serves from the standalone directory

## CI/CD (GitHub Actions)

Located in `.github/workflows/`:

| Workflow | File | Trigger |
|----------|------|---------|
| Backend CI | `backend.yml` | Rust build, test, lint on PRs |
| Docker Backend | `docker-backend.yml` | Build & push backend container image |
| Docker Frontend | `docker-frontend.yml` | Build & push frontend container image |
| PR Image Cleanup | `cleanup-pr-images.yml` | Remove temporary container images on PR close |

## Production Configuration

### Backend Environment

Set the following in production:

```env
DATABASE_URL=postgresql://user:pass@host:5432/cinematch
REDIS_URL=redis://host:6379
QDRANT_URL=http://host:6334
SECRET_TOKEN=<64+ byte random string>
SERVER_HOST=0.0.0.0
SERVER_PORT=8085
JWT_EXPIRY_SECS=3600

# Optional: GitHub OAuth
GITHUB_CLIENT_ID=<client_id>
GITHUB_CLIENT_SECRET=<client_secret>

# Optional: Phase timeouts
VOTING_R1_TIMEOUT_SECS=30
VOTING_R2_TIMEOUT_SECS=20
WATCHING_TIMEOUT_SECS=900
READY_COUNTDOWN_SECS=5.0
```

### Frontend Environment

```env
NEXT_PUBLIC_API_BASE=https://api.cinematch.space
```

### Service Endpoints (Production)

| Service | Default URL |
|---------|-------------|
| Frontend | `https://cinematch.space` |
| Backend API | `https://api.cinematch.space` |

## Next.js Standalone Output

The frontend is configured with `output: 'standalone'` in `next.config.ts`, producing a self-contained Node.js server for container deployment without needing the full `node_modules`.
