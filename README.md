<div align="center">
	<h1>CineMatch</h1>
	<p><strong>Watch Parties & Movie Recommendations, Together</strong></p>
</div>

---

## Overview

CineMatch is a collaborative movie night platform that lets users create and join watch parties, vote on movies, and receive personalized recommendations. Built with a modern stack (Rust backend, Next.js frontend), CineMatch enables real-time party management and interactive experiences for groups.

# CineMatch

CineMatch is a platform for creating and joining movie watch parties. Users can vote on movies, manage party membership, and get group-based recommendations. The project uses a Rust backend and a Next.js frontend.

## Features

- Create/join parties
- Real-time party room and voting
- Party leader controls (kick, transfer, advance phase)
- Guest and authenticated users
- Movie recommendations
- WebSocket updates

## Tech Stack

- Frontend: Next.js (React), TypeScript
- Backend: Rust (Actix Web), Diesel ORM
- Database: PostgreSQL
- Recommendation Engine: Rust
- WebSockets: Real-time updates
- Docker: Dev and deployment

## Structure

```
react_hackathon_2026/
├── backend/
│   ├── cinematch-api/
│   ├── cinematch-db/
│   ├── cinematch-common/
│   ├── cinematch-recommendation-engine/
├── frontend/
│   ├── src/
│   ├── public/
└── README.md
```

## Getting Started

### Prerequisites

- Rust
- Node.js & pnpm
- Docker (optional)
- PostgreSQL (if not using Docker)

### Local Development

Clone the repository:

```sh
git clone https://github.com/CineMatch-TiTe/react_hackathon_2026.git
cd react_hackathon_2026
```

Start backend with Docker (recommended):

```sh
cd backend
docker-compose -f docker-compose.dev.yml up --build
```

Start frontend:

```sh
cd frontend
pnpm install
pnpm dev
```

App URLs:

- Frontend: http://localhost:3000
- Backend API: http://localhost:8085

- Swagger UI: http://localhost:8085/swagger-ui/
- ReDoc: http://localhost:8085/redoc
- Scalar: http://localhost:8085/scalar
- RapiDoc: http://localhost:8085/rapidoc

#### Manual (no Docker)

- Set up PostgreSQL and update backend configs
- Run DB migrations: `diesel migration run` in backend/cinematch-db
- Build/run Rust services with `cargo run`
- Start frontend as above

## License

MIT License. See LICENSE for details.
