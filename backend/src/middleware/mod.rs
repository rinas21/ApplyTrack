// Middleware layer (rate limiting/audit hooks can be added via tower layers).
// Authentication is enforced by the AuthUser extractor on protected handlers.
pub struct RateLimitConfig;
