use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;

pub async fn upsert_user(
    pool: &PgPool,
    sub: &str,
    email: Option<&str>,
    display_name: Option<&str>,
) -> Result<AuthUser> {
    let row = sqlx::query_as::<_, (Uuid, String, Option<String>, Option<String>)>(
        r#"
        INSERT INTO users (keycloak_sub, email, display_name)
        VALUES ($1, $2, $3)
        ON CONFLICT (keycloak_sub) DO UPDATE
        SET email = COALESCE(EXCLUDED.email, users.email),
            display_name = COALESCE(EXCLUDED.display_name, users.display_name),
            updated_at = now()
        RETURNING id, keycloak_sub, email, display_name
        "#,
    )
    .bind(sub)
    .bind(email)
    .bind(display_name)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        "INSERT INTO user_preferences (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING",
    )
    .bind(row.0)
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO subscriptions (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING",
    )
    .bind(row.0)
    .execute(pool)
    .await?;

    Ok(AuthUser {
        id: row.0,
        keycloak_sub: row.1,
        email: row.2,
        display_name: row.3,
    })
}
