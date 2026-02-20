CineMatch
=========

Collaborative movie watch party platform with real-time voting and ML-based recommendations.

Overview
--------

CineMatch enables users to create and join watch parties, vote on movies, and receive personalized recommendations. The system utilizes a Rust backend for high-performance data processing and a Next.js frontend for the user interface.

Features
--------

- **Party Management**: Create/join parties, real-time membership tracking.
- **Voting System**: Real-time ballot distribution and voting consensus.
- **Access Control**: Party leader administrative capabilities (kick, phase transitions).
- **Authentication**: Guest access and persistent user accounts.
- **Recommendation Engine**: Multi-strategy support (collaborative filtering, content-based, onboarding).
- **Real-time Communication**: WebSocket-based state synchronization.

Technology Stack
----------------

- **Frontend**: Next.js (React), TypeScript.
- **Backend**: Rust (Actix Web), Diesel ORM.
- **Database**: PostgreSQL 15, Redis 7, Qdrant.
- **Infrastructure**: Docker Compose.

Project Structure
-----------------

```
react_hackathon_2026/
├── backend/
│   ├── cinematch-api/               # REST API definition
│   ├── cinematch-db/                # Database interactions
│   ├── cinematch-common/            # Shared types and models
│   ├── cinematch-recommendation-engine/ # ML algorithms
│   └── cinematch-importer/          # Data ingestion CLI
├── frontend/
│   ├── src/                         # React source code
│   └── public/                      # Static assets
└── README.md
```

Getting Started
---------------

### Prerequisites

- Rust (stable)
- Node.js & pnpm
- Docker & Docker Compose
- PostgreSQL (local fallback)

### Development Setup

Clone the repository:

```sh
git clone https://github.com/CineMatch-TiTe/react_hackathon_2026.git
cd react_hackathon_2026
```

#### Backend

Start services using Docker:

```sh
cd backend
docker-compose -f docker-compose.dev.yml up --build
```

#### Frontend

Install dependencies and start the development server:

```sh
cd frontend
pnpm install
pnpm dev
```

### Service Endpoints

- **Frontend**: `http://localhost:3000`
- **Backend API**: `http://localhost:8085`
- **Swagger UI**: `http://localhost:8085/swagger-ui/`
- **ReDoc**: `http://localhost:8085/redoc`

License
-------

MIT License. See LICENSE for details.
