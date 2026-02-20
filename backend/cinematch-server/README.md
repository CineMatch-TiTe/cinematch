cinematch-server
================

[← Back to main README](../README.md)

Actix-web HTTP server and WebSocket gateway. Default workspace entry point.

Endpoints
---------

| Scope | Endpoints | Description |
|-------|-----------|-------------|
| `/api/auth` | `POST /login/guest`, `POST /logout` | Guest login, logout. |
| `/api/user` | `GET /me`, `PATCH /rename`, `PATCH /taste`, `GET /pref`, `PUT /pref` | User profile and preferences. |
| `/api/party` | `POST /create`, `GET /`, `POST /join`, `POST /leave`, `GET /members`, `PATCH /ready`, `POST /advance-phase`, `POST /disband` | Party lifecycle management. |
| `/api/party` | `GET /picks`, `POST /pick`, `DELETE /pick`, `GET /vote`, `POST /vote` | Movie picking and voting. |
| `/api/movie` | `GET /:id`, `GET /genres`, `POST /search` | Movie catalog queries. |
| `/api/recommend` | `GET /` | Personalized recommendations. |
| `/api/ws` | WebSocket upgrade | Real-time party updates. |
| `/api/onboarding` | `POST /start`, `POST /rate` | Onboarding session flow. |

Features
--------

- **Session Authentication**: `actix-identity` + `actix-session` (Redis-backed).
- **OpenAPI**: Generated via `utoipa`. Available at `/swagger-ui/`.
- **CORS**: Configured for development environment.
- **Compression**: Default middleware enabled.
