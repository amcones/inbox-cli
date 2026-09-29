use jiff::Timestamp;

pub fn score(created: Timestamp, views: u64, now: Timestamp) -> f64 {
    let age = (now.as_second() - created.as_second()).max(0) as f64 / 86_400.0;
    let recency = 2.0_f64.powf(-age / 14.0);
    let frequency = views as f64 / (views as f64 + 5.0);
    0.65 * recency + 0.35 * frequency
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranking_rewards_recency_and_views_with_diminishing_returns() {
        let now: Timestamp = "2026-09-29T00:00:00Z".parse().unwrap();
        let old: Timestamp = "2026-09-15T00:00:00Z".parse().unwrap();
        assert!((score(old, 0, now) - 0.325).abs() < 1e-10);
        assert!(score(now, 0, now) > score(old, 0, now));
        assert!(score(old, 5, now) > score(old, 0, now));
        assert!(score(old, 5, now) - score(old, 0, now) > score(old, 10, now) - score(old, 5, now));
        assert!(score(old, u64::MAX, now).is_finite());
        assert_eq!(score(now, 0, old), score(now, 0, now));
    }
}
