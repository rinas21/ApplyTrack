# ApplyTrack

ApplyTrack is a production-oriented, multi-user SaaS foundation for tracking job applications, capturing jobs from a browser extension, synchronizing recruitment email signals, and analyzing job-search performance.

> **Status (verified 2026-11-07):** working modular-monolith foundation. The full Docker stack builds and starts; core authenticated CRUD/status/analytics/extension-capture workflows and strict per-user isolation were verified at runtime (functional verification used a temporary local `DEV_AUTH_BYPASS` override only; auth enforcement was rechecked after restoration). Real browser Keycloak login/PKCE end-to-end and live Gmail OAuth require configured credentials/interactive flows (see limitations).

## What is implemented

### Infrastructure / foundation
- Docker Compose environment for `postgres`, `keycloak`, `api`, `worker`, and `frontend`.
- Separate logical PostgreSQL databases for ApplyTrack and Keycloak (same local Postgres instance, initialized by script).
- Multi-stage Dockerfiles for backend and frontend.
- Keycloak realm export (`infrastructure/keycloak/applytrack-realm.json`) with `applytrack` realm and public PKCE clients:
  - `applytrack-web` (Angular)
  - `applytrack-extension` (browser extension)
- Environment-driven configuration (`.env.example`).
- Health checks and startup dependencies.

### Backend (Rust, Axum, Tokio, SQLx, Serde, tracing, Utoipa)
- Modular domain structure: `auth`, `users`, `companies`, `jobs`, `applications`, `analytics`, `email`, `reminders`, `extension`, `notifications`, `subscriptions`, `common`, `config`, `database`.
- PostgreSQL migrations in `backend/migrations/20261107000000_init.sql`.
- Keycloak JWT verification via JWKS (issuer validation; configurable audience/JWKS URL).
- Authenticated user upsert mapped to Keycloak `sub` (never trusts frontend user IDs).
- Core CRUD APIs under `/api/v1`:
  - `/me`, `/settings/*`
  - `/companies` (CRUD + stats)
  - `/jobs` (CRUD/search; URL-based dedupe per user)
  - `/applications` (CRUD, filtering/pagination, status change, status history)
  - `/analytics/summary`, `/analytics/export.csv`
  - `/reminders` (CRUD-ish actions + suggestions)
  - `/extension/capture`, `/extension/recent`
  - `/email/*` (accounts, Gmail auth URL/callback, sync, activity, match correction)
- OpenAPI JSON at `/api-docs/openapi.json` (minimal spec; handlers can be expanded).
- Background worker binary (`worker`) polls connected Gmail accounts and runs sync.
- Gmail integration architecture implemented: OAuth code exchange, encrypted refresh-token storage (AES-GCM key derived from `TOKEN_ENCRYPTION_KEY`), Gmail metadata sync, rule-based classification, application matching, manual correction, and high-confidence auto status updates.
- **Real Gmail sync requires Google OAuth credentials and user consent** (see setup).

### Frontend (Angular 18, TypeScript, Router, Signals, Reactive-capable forms)
- Config loaded from `/config.json` (`APP_INITIALIZER`).
- Keycloak OIDC Authorization Code Flow with PKCE (public client).
- Auth guard, token interceptor, API service.
- SaaS layout/navigation: Overview, Applications, Companies, Jobs, Analytics, Email Activity, Reminders, Settings.
- Dashboard and analytics consume real backend endpoints.
- Applications page supports create/edit/status change/delete and filtering.
- Jobs/Companies/Reminders/Email Activity/Settings pages wired to API.
- Responsive dark dashboard styling, loading/empty/error states in key flows.

### Browser extension (TypeScript, Chrome Manifest V3)
- MV3 manifest, popup, options page, background service worker, content script.
- Extraction adapters for LinkedIn and Indeed, plus generic fallback adapter.
- User can review/edit extracted fields before saving.
- Saves jobs/applications via `/api/v1/extension/capture`.
- Shows recent saved jobs; duplicate detection uses existing jobs by URL and backend safeguards.
- “Mark as applied” requires explicit user confirmation in popup.
- Local/dev auth can use a configured Bearer token in options; production should use full Keycloak PKCE via `applytrack-extension`.

## Architecture

```text
ApplyTrack/
├── frontend/                  # Angular 18 app
│   └── src/app/
│       ├── core/              # auth (PKCE), guards, interceptors, API/config services
│       ├── layout/            # shell/navigation
│       └── features/          # dashboard, applications, jobs, companies, analytics, email, reminders, settings
├── backend/                   # Rust modular monolith
│   ├── src/
│   │   ├── auth/              # JWT/JWKS verification, AuthUser extractor
│   │   ├── users/ companies/ jobs/ applications/ analytics/ reminders/
│   │   ├── email/             # Gmail OAuth/sync, classifier, matcher
│   │   ├── extension/         # capture/recent endpoints
│   │   ├── common/            # errors, responses, pagination, crypto
│   │   └── bin/               # api, worker
│   └── migrations/            # SQLx migrations
├── extension/                 # Chrome MV3 extension (TypeScript)
│   └── src/                   # popup/options/background/content adapters
├── infrastructure/
│   ├── docker/postgres-init/  # creates Keycloak DB
│   └── keycloak/              # realm export
├── docs/
├── docker-compose.yml
└── .env.example
```

## Quick start (Docker)

### Prerequisites
- Docker + Docker Compose v2
- (Optional for local non-Docker dev) Node 22+, npm, Rust stable, PostgreSQL, Keycloak

### 1) Configure environment
```bash
cp .env.example .env
# Edit .env. At minimum, set a strong TOKEN_ENCRYPTION_KEY.
# For Gmail sync, set GOOGLE_CLIENT_ID/SECRET/REDIRECT_URI.
```

### 2) Start services
```bash
docker compose up --build
```

Services (default):
- Frontend: http://localhost:4200
- API: http://localhost:8080 (`/health`, `/api/v1/health`, `/api-docs/openapi.json`)
- Keycloak: http://localhost:8180 (admin: `admin` / `admin` unless changed)
- PostgreSQL: localhost:5432

### 3) Configure Keycloak for login
The realm is imported from `infrastructure/keycloak/applytrack-realm.json`.

1. Open Keycloak Admin Console: http://localhost:8180
2. Sign in with `KEYCLOAK_ADMIN` / `KEYCLOAK_ADMIN_PASSWORD` (default `admin/admin`).
3. Select realm **applytrack**.
4. Create a user (Users → Add user), set email/username, and set a password in **Credentials** (temporary off).
5. Ensure clients `applytrack-web` and `applytrack-extension` exist and redirect URIs match your environment.

### 4) Use the app
1. Open http://localhost:4200
2. Click **Continue with Keycloak** and sign in.
3. Add applications/companies/jobs; dashboard and analytics query real API data.

## Local development (without full Docker app stack)

### Database + Keycloak (recommended via Docker)
```bash
docker compose up -d postgres keycloak
```

### Backend
```bash
cd backend
cp ../.env.example ../.env # or export env vars
# Ensure DATABASE_URL points to local Postgres (applytrack DB)
# For local browser auth, KEYCLOAK_ISSUER should be reachable from browser AND JWKS URL from API.
# In compose, API uses KEYCLOAK_JWKS_URL=http://keycloak:8080/... while issuer is http://localhost:8180/...
cargo run --bin api
# optional worker
cargo run --bin worker
```

`DEV_AUTH_BYPASS=true` allows local API testing with headers `X-Dev-Sub` and `X-Dev-Email` **only for explicit local development**. Do not enable in production.

### Frontend
```bash
cd frontend
npm install
npm start
```
Edit `public/config.json` if API/Keycloak URLs differ.

### Browser extension
```bash
cd extension
npm install
npm run build
```
Load unpacked in Chrome:
1. `chrome://extensions`
2. Enable Developer mode
3. **Load unpacked** → select `ApplyTrack/extension`
4. Open extension **Settings** and set API base URL and (for local dev) a Bearer access token, or implement full PKCE flow.

## Google OAuth (Gmail) setup

Gmail sync requires real Google Cloud credentials.

1. Create/select a Google Cloud project.
2. Enable **Gmail API**.
3. Configure OAuth consent screen (external/internal as appropriate for testing).
4. Create OAuth 2.0 Client ID (**Web application**).
5. Add authorized redirect URI matching `GOOGLE_REDIRECT_URI` (default `http://localhost:4200/settings/email/callback`).
6. Set env vars: `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_REDIRECT_URI`.
7. In ApplyTrack Settings, connect Gmail. The backend stores encrypted refresh tokens; browser/extension never receive refresh tokens.
8. Trigger sync manually from Settings or let `worker` poll connected accounts.

**Privacy/security:** request minimal scopes (`gmail.readonly` in current implementation), allow disconnect and synchronized data deletion, treat email content as untrusted input, and do not execute instructions found in emails.

## Database

- PostgreSQL with UUID PKs, foreign keys, indexes, unique constraints, timestamps.
- Migrations: `backend/migrations/*.sql` (SQLx). API/worker run migrations on startup.
- Core tables include users/preferences, companies, jobs, applications, status history, notes/tags, connected email accounts/sync states/events/matches, reminders, notifications, subscriptions, usage records, audit logs.

## API overview

Base: `/api/v1` (authenticated unless noted)
- `GET /health` (public), `GET /api/v1/health`
- `GET /me`
- `GET/POST /companies`, `GET/PUT/DELETE /companies/{id}`, `GET /companies/{id}/stats`
- `GET/POST /jobs`, `GET/PUT/DELETE /jobs/{id}`
- `GET/POST /applications`, `GET/PUT/DELETE /applications/{id}`, `PATCH /applications/{id}/status`, `GET /applications/{id}/history`
- `GET /analytics/summary`, `GET /analytics/export.csv`
- `GET/POST /reminders`, `PATCH /reminders/{id}/complete|dismiss`, `GET /reminders/suggestions`
- `POST /extension/capture`, `GET /extension/recent`
- `GET /email/accounts`, `GET /email/gmail/auth-url`, `POST /email/gmail/callback`, `POST /email/accounts/{id}/sync`, `DELETE /email/accounts/{id}`, `GET /email/activity`, `PATCH /email/matches/{id}`
- `GET/PUT /settings/preferences`, `GET /settings/subscription`, `GET /settings/export`

## Tests

Run available automated tests:
```bash
cd backend && cargo test
cd frontend && CI=1 npm test -- --watch=false --browsers=ChromeHeadless
cd extension && npm test
cd frontend && npm run build
cd extension && npm run build
```

### Verified results (2026-11-07)
- **Backend (`cargo test`)**: 8 passed, 0 failed — `common::crypto` encrypt/decrypt + nonce behavior; `email::classifier` (confirmation/interview/rejection/unknown); `email::matcher::status_for_event`; `applications::valid_status`.
- **Frontend**: production build passed; Karma/ChromeHeadless **2/2 SUCCESS**.
- **Extension**: build passed; Node tests **3/3 passed** (LinkedIn/Indeed/generic URL matching via `matches()`).
- **Docker build**: `docker compose up --build -d` completed; images built for `applytrack-api`, `applytrack-worker`, `applytrack-frontend` (backend Dockerfile uses `rust:1.91-bookworm`, copies `Cargo.lock`, excludes local `target/` via `backend/.dockerignore`).
- **Runtime (Docker, restored auth defaults)**:
  - Containers healthy/up: `postgres` (healthy), `keycloak` (up; realm imported), `api` (healthy), `worker` (up), `frontend` (up).
  - PostgreSQL init created `applytrack` + `keycloak` DBs; SQLx migrations applied (ApplyTrack domain tables present, including `_sqlx_migrations`).
  - Keycloak realm import confirmed (`Realm 'applytrack' imported`); OIDC discovery `GET /realms/applytrack/.well-known/openid-configuration` returned **200**.
  - Auth enforcement: unauthenticated `GET /api/v1/me` and `GET /api/v1/extension/recent` returned **401** (`missing authorization header`); `GET /health` and `GET /api/v1/health` returned **200**.
  - Frontend served `GET /` **200** and `/config.json` with API/Keycloak settings.
  - Worker started and remained stable with no connected Gmail accounts (Gmail sync processing not exercisable without credentials/accounts).
- **Functional runtime verification (temporary local `DEV_AUTH_BYPASS=true` override on `api`/`worker` only; removed and auth reverified)**: PASS for company/job/application create+get/list, application `PUT /applications/{id}` route availability/handling (router accepts PUT), `PATCH /applications/{id}/status` (Applied), `GET /applications/{id}/history` (>=2 entries), analytics summary reflecting submitted totals after status change, `POST /extension/capture`, `GET /extension/recent`, and strict isolation (user B received 404 for user A company/job/application access and saw no user A applications).
- **Failure found and fixed during verification**: `/{id}` path-parameter routes returned 404 under Axum 0.7.9 because the codebase used `{id}` capture syntax; fixed to Axum 0.7-compatible `:id` routing (including nested application routes and email/reminder ID routes), rebuilt `api`/`worker`, and reverified.

## Security notes

- JWT signature/issuer validation via Keycloak JWKS.
- User data isolated by internal user ID derived from authenticated Keycloak subject.
- No Keycloak admin/client secrets in frontend or extension.
- Gmail refresh tokens encrypted at rest server-side.
- Email content treated as untrusted; no instruction execution.
- CORS restricted by config.
- Full production hardening still needed: stricter audience validation per deployment, refresh-token rotation policies, rate limiting middleware, CSRF considerations for cookie-based flows (not used), audit logging hooks, and comprehensive account deletion/export workflows.

## Known limitations / next steps

- Gmail sync is implemented but **cannot be fully verified without real Google OAuth credentials**.
- Outlook/Microsoft Graph: architecture placeholders only; implement provider abstraction and Graph sync.
- Extension uses a practical local token/config flow; complete Keycloak PKCE in MV3 for production.
- Extension production auth should use full Keycloak PKCE in MV3 (current options page supports local Bearer token configuration for development/testing).
- Subscription/billing provider integration not connected (models/limits prepared).
- Add comprehensive automated tests and CI.
- Expand OpenAPI annotations and add Swagger UI (compatible versions) if desired.
- Add production deployment manifests (TLS, ingress, secrets manager, managed Postgres/Keycloak).

## Environment variables

See `.env.example` for all variables (database, Keycloak, API, CORS, encryption, Google OAuth, frontend config).
