# Security & Privacy

- **Auth:** Keycloak OIDC/OAuth2 Authorization Code + PKCE. Rust API validates JWT signature, issuer, expiration (audience configurable).
- **Authorization:** all user data queries filtered by authenticated internal user ID.
- **Secrets:** no Keycloak secrets/admin creds in browser code; no email refresh/access tokens exposed to frontend/extension.
- **Email tokens:** refresh tokens encrypted at rest with `TOKEN_ENCRYPTION_KEY` (AES-256-GCM, derived key).
- **Email safety:** treat email content/headers as untrusted input; never execute instructions from emails; use minimal scopes; allow disconnect/data deletion.
- **Input validation:** request DTO validation and parameterized SQLx queries.
- **Verified (2026-11-07):** unauthenticated access to protected endpoints (`/api/v1/me`, extension endpoints) returns 401 in the Docker runtime with default config; strict user-level data isolation was verified (cross-user company/job/application access denied/not visible). `DEV_AUTH_BYPASS` defaults to false and was only used as a temporary local override during functional verification, then removed.
- **Hardening TODO:** rate limiting, comprehensive audit-log writes on sensitive actions, production CORS/origins, secure token storage strategy for extension (full PKCE), monitoring, dependency scanning.
