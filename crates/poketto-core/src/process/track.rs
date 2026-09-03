use std::time::{Duration, Instant};
use tokio::process::Child;

pub struct RunTracker {
    pub game_id: String,
    pub title: String,
    pub start: Instant,
}

impl RunTracker {
    pub fn start(game_id: &str, title: &str) -> Self {
        Self {
            game_id: game_id.to_string(),
            title: title.to_string(),
            start: Instant::now(),
        }
    }
    pub async fn wait_for_exit(self, mut child: Child) -> std::io::Result<CompletedRun> {
        child.wait().await?;
        let run = CompletedRun {
            game_id: self.game_id.clone(),
            duration: self.start.elapsed(),
        };
        tracing::info!(game_id = run.game_id.as_str(), minutes = run.play_minutes(), "game exited");
        Ok(run)
    }
}

pub struct CompletedRun {
    pub game_id: String,
    pub duration: Duration,
}

impl CompletedRun {
    pub fn play_minutes(&self) -> u64 {
        self.duration.as_secs() / 60
    }
}

pub struct LocalTimestamps {
    pub rfc3339: String,
    pub date: String,
}

impl LocalTimestamps {
    pub fn now() -> Self {
        let now = chrono::Local::now();
        Self {
            rfc3339: now.to_rfc3339(),
            date: now.format("%Y-%m-%d").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minutes_truncate_like_legacy() {
        let run = CompletedRun {
            game_id: "g1".to_string(),
            duration: Duration::from_secs(119),
        };
        assert_eq!(run.play_minutes(), 1);
    }

    #[tokio::test]
    #[cfg(target_os = "linux")]
    async fn fast_process_completes_with_zero_minutes() {
        let tracker = RunTracker::start("g1", "True");
        let child = tokio::process::Command::new("/bin/true")
            .spawn()
            .expect("spawn true");
        let run = tracker.wait_for_exit(child).await.expect("wait");
        assert_eq!(run.game_id, "g1");
        assert_eq!(run.play_minutes(), 0);
    }
}
