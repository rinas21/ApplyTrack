# Setup

## Verified services (Docker Compose)
| Service | Default URL/port | Verification notes (2026-11-07) |
|---|---|---|
| Frontend (nginx) | http://localhost:4200 | Serves SPA (`/` 200) and `/config.json` (API/Keycloak config). |
| API (Rust/Axum) | http://localhost:8080 | `/health`, `/api/v1/health` 200; OpenAPI JSON `/api-docs/openapi.json`; unauth private routes return 401. |
| Keycloak | http://localhost:8180 | Realm `applytrack` imported; OIDC discovery 200. Admin default `admin/admin` unless changed. |
| PostgreSQL | localhost:5432 | `applytrack` + `keycloak` DBs created; ApplyTrack SQLx migrations applied on startup. |
| Worker | internal | Starts and stays up; Gmail sync polling requires connected account + Google OAuth credentials. |

## Docker startup (recommended)
```bash
cp .env.example .env
# Set a strong TOKEN_ENCRYPTION_KEY. Add Google vars only if testing Gmail.
docker compose up --build -d
docker compose ps
docker compose logs --tail=100 postgres keycloak api worker frontend
```

## Required / notable environment variables
See `.env.example` for the full list.
- `POSTGRES_*`, `DATABASE_URL` — ApplyTrack DB connection.
- `KEYCLOAK_ADMIN*`, `KEYCLOAK_ISSUER`, `KEYCLOAK_JWKS_URL`, `KEYCLOAK_AUDIENCE` — Keycloak and JWT validation. In Compose, API uses internal JWKS URL (`http://keycloak:8080/...`) and issuer `http://localhost:8180/realms/applytrack`.
- `CORS_ORIGIN` — frontend origin (default `http://localhost:4200`).
- `TOKEN_ENCRYPTION_KEY` — required for encrypted email token storage (set a strong value).
- `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_REDIRECT_URI` — required only for live Gmail OAuth/sync.
- `DEV_AUTH_BYPASS` — **default false / keep disabled in production**. Only for explicit local development/testing; not used for normal operation.

## Keycloak
- Realm import: `infrastructure/keycloak/applytrack-realm.json`.
- Web client: `applytrack-web` (public, PKCE S256, redirect `http://localhost:4200/*`).
- Extension client: `applytrack-extension` (public, PKCE S256).
- Issuer: `http://localhost:8180/realms/applytrack`.
- Create a test user in realm `applytrack` (Users → Add user → Credentials, temporary off) before browser login.

## Gmail
Set Google OAuth env vars and connect in Settings. Live Gmail sync requires a Google Cloud OAuth client with Gmail API enabled and matching redirect URI. **Not fully verified in this environment (credentials not configured).**

## Local tests
```bash
cd backend && cargo test
cd frontend && CI=1 npm test -- --watch=false --browsers=ChromeHeadless
cd extension && npm test
```
See `README.md` for the verified 2026-11-07 results.
