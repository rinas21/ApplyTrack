// Subscription/usage-limit models are persisted in DB (subscriptions, usage_records).
// Billing provider integration is intentionally isolated for future implementation.
pub const FREE_ACTIVE_APPLICATION_LIMIT: i64 = 50;
pub const PRO_ACTIVE_APPLICATION_LIMIT: i64 = 5000;
