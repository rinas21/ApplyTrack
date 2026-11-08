# ApplyTrack Architecture

## Overview
ApplyTrack is a modular monolith SaaS:
- **Angular frontend** (`frontend/`): SPA using Keycloak OIDC PKCE, calling the Rust API.
- **Rust API** (`backend/src/bin/api.rs`): Axum REST API, SQLx/PostgreSQL, JWT validation, domain modules.
- **Rust worker** (`backend/src/bin/worker.rs`): background Gmail sync/reminder processing foundation.
- **PostgreSQL**: normalized multi-tenant schema with ownership via `user_id` linked to Keycloak `sub`.
- **Keycloak**: identity provider, realm `applytrack`, public PKCE clients for web and extension.
- **Browser extension** (`extension/`): Chrome MV3 capture tool calling `/api/v1/extension/*`.

## Backend modules
- `auth`: JWKS fetch/cache, JWT validation, `AuthUser` extractor (upserts internal user).
- `users`: profile/settings endpoints.
- `companies`, `jobs`, `applications`: core CRUD with per-user ownership checks.
- `analytics`: DB-backed metrics and CSV export.
- `email`: Gmail OAuth, encrypted refresh tokens, sync, classification, matching, correction.
- `reminders`, `notifications`, `subscriptions`: scheduling and plan/limit models.
- `common`: errors, response envelope, pagination, crypto.

## Security model
- Every private resource query is scoped to authenticated `user.id`.
- Frontend-supplied user IDs are never trusted.
- Keycloak public clients use Authorization Code + PKCE.
- Email refresh tokens encrypted server-side; never sent to browser/extension.

## Deployment & verification
Docker Compose for local/dev (`postgres`, `keycloak`, `api`, `worker`, `frontend`). Verified 2026-11-07: images build, stack starts, migrations/realm import run, auth is enforced (401 without token), and core CRUD/status/analytics/extension-capture/isolation workflows pass against the running stack (functional checks used a temporary local dev-auth override only). Production should use managed Postgres/Keycloak or hardened containers, TLS, secret management, rate limiting, and CI/CD. Live Gmail sync requires external Google OAuth credentials.
