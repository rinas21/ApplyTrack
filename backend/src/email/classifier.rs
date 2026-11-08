// Simple rule-based classifier for recruitment emails.
// In production, combine with sender reputation, user corrections, and optional ML.
#[derive(Debug, Clone)]
pub struct Classification {
    pub event_type: String,
    pub confidence: f64,
}

pub fn classify(subject: &str, snippet: &str, from: &str) -> Classification {
    let text = format!("{} {} {}", subject, snippet, from).to_lowercase();
    let rules: &[(&[&str], &str, f64)] = &[
        (&["interview", "schedule a call", "invite you to interview", "phone screen"], "interview", 0.88),
        (&["assessment", "coding challenge", "take-home", "hackerrank", "codility"], "assessment", 0.86),
        (&["unfortunately", "not moving forward", "regret to inform", "we will not be proceeding", "position has been filled"], "rejection", 0.90),
        (&["offer", "pleased to offer", "job offer", "compensation package"], "offer", 0.84),
        (&["application received", "we received your application", "thank you for applying", "application confirmation"], "confirmation", 0.90),
        (&["recruiter", "opportunity", "your profile", "talent acquisition"], "recruiter", 0.55),
    ];
    for (needles, kind, conf) in rules {
        if needles.iter().any(|n| text.contains(n)) {
            return Classification { event_type: (*kind).into(), confidence: *conf };
        }
    }
    Classification { event_type: "unknown".into(), confidence: 0.2 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_interview() {
        let c = classify("Interview invitation", "We'd like to schedule a call", "hr@example.com");
        assert_eq!(c.event_type, "interview");
        assert!(c.confidence >= 0.8);
    }
    #[test]
    fn detects_rejection() {
        let c = classify("Update on your application", "Unfortunately we will not be proceeding", "jobs@example.com");
        assert_eq!(c.event_type, "rejection");
    }
    #[test]
    fn detects_confirmation() {
        let c = classify("Application received", "Thank you for applying", "no-reply@example.com");
        assert_eq!(c.event_type, "confirmation");
    }
    #[test]
    fn unknown_when_no_signal() {
        let c = classify("Newsletter", "Weekly updates", "news@example.com");
        assert_eq!(c.event_type, "unknown");
    }
}
