use sqlx::PgPool;
use uuid::Uuid;

pub async fn suggest_application(
    pool: &PgPool,
    user_id: Uuid,
    subject: &str,
    from: &str,
    snippet: &str,
) -> anyhow::Result<Option<(Uuid, f64)>> {
    let text = format!("{} {} {}", subject, from, snippet).to_lowercase();
    let domain = from.split('@').nth(1).unwrap_or("").trim_matches(|c| c == '>' || c == ' ' || c == '"');
    let apps = sqlx::query_as::<_, (Uuid, String, String, Option<String>)>(
        "SELECT id, lower(company_name), lower(job_title), job_url FROM applications WHERE user_id=$1 AND archived=false ORDER BY updated_at DESC LIMIT 200"
    ).bind(user_id).fetch_all(pool).await?;
    let mut best: Option<(Uuid, f64)> = None;
    for (id, company, title, _url) in apps {
        let mut score: f64 = 0.0;
        if !company.is_empty() && (text.contains(&company) || (!domain.is_empty() && domain.contains(&company.replace(' ', "")))) { score += 0.55; }
        if !title.is_empty() && text.split_whitespace().filter(|w| w.len()>3 && title.contains(*w)).count() >= 2 { score += 0.25; }
        if score > 0.0 {
            if best.as_ref().map(|b| score > b.1).unwrap_or(true) { best = Some((id, score.min(0.99))); }
        }
    }
    Ok(best.filter(|b| b.1 >= 0.5))
}

pub fn status_for_event(event_type: &str) -> Option<&'static str> {
    match event_type {
        "confirmation" => Some("Applied"),
        "assessment" => Some("Assessment"),
        "interview" => Some("Interview"),
        "rejection" => Some("Rejected"),
        "offer" => Some("Offer"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_events_to_statuses() {
        assert_eq!(status_for_event("confirmation"), Some("Applied"));
        assert_eq!(status_for_event("interview"), Some("Interview"));
        assert_eq!(status_for_event("rejection"), Some("Rejected"));
        assert_eq!(status_for_event("offer"), Some("Offer"));
        assert_eq!(status_for_event("recruiter"), None);
        assert_eq!(status_for_event("unknown"), None);
    }
}
