use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct RateLimiter {
    interval: Duration,
    last: Mutex<Option<Instant>>,
}

impl RateLimiter {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last: Mutex::new(None),
        }
    }

    pub fn vndb() -> Self {
        Self::new(Duration::from_secs(1))
    }

    pub async fn wait(&self) {
        let sleep_for = {
            let mut last = self.last.lock().await;
            let now = Instant::now();
            let sleep_for = match *last {
                Some(previous) => self.interval.checked_sub(now.duration_since(previous)),
                None => None,
            };
            *last = Some(now + sleep_for.unwrap_or(Duration::ZERO));
            sleep_for
        };
        if let Some(delay) = sleep_for {
            tokio::time::sleep(delay).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn first_request_passes_immediately() {
        let limiter = RateLimiter::new(Duration::from_secs(10));
        limiter.wait().await;
    }

    #[tokio::test]
    async fn second_request_waits_for_interval() {
        let limiter = RateLimiter::new(Duration::from_millis(120));
        limiter.wait().await;
        let start = Instant::now();
        limiter.wait().await;
        assert!(start.elapsed() >= Duration::from_millis(100));
    }

    #[tokio::test]
    async fn burst_debt_accumulates() {
        let limiter = RateLimiter::new(Duration::from_millis(80));
        limiter.wait().await;
        let start = Instant::now();
        limiter.wait().await;
        limiter.wait().await;
        assert!(start.elapsed() >= Duration::from_millis(140));
    }
}
