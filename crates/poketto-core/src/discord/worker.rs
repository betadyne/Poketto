use discord_rich_presence::{
    activity::{self, ActivityType, StatusDisplayType},
    DiscordIpc, DiscordIpcClient,
};
use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

pub const DISCORD_CLIENT_ID: &str = "1454731999637147732";
pub const DEFAULT_STATE_TEXT: &str = "Playing Visual Novel";
const MAX_BUTTONS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresenceUpdate {
    pub title: String,
    pub state: String,
    pub cover_url: Option<String>,
    pub buttons: Vec<(String, String)>,
    pub start_timestamp: u64,
}

impl PresenceUpdate {
    pub fn playing(
        title: &str,
        developer: Option<&str>,
        cover_url: Option<&str>,
        buttons: Vec<(String, String)>,
        start_timestamp: u64,
    ) -> Self {
        Self {
            title: title.to_string(),
            state: developer.unwrap_or(DEFAULT_STATE_TEXT).to_string(),
            cover_url: cover_url.map(|url| url.to_string()),
            buttons: buttons.into_iter().take(MAX_BUTTONS).collect(),
            start_timestamp,
        }
    }
}

pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

enum WorkerCommand {
    Set(PresenceUpdate),
    Clear,
    Shutdown,
}

fn should_send(last: &Option<PresenceUpdate>, next: &PresenceUpdate) -> bool {
    last.as_ref() != Some(next)
}

fn send_update(client: &mut DiscordIpcClient, update: &PresenceUpdate) -> bool {
    let mut activity = activity::Activity::new()
        .details(update.title.as_str())
        .state(update.state.as_str())
        .activity_type(ActivityType::Playing)
        .status_display_type(StatusDisplayType::Details)
        .timestamps(activity::Timestamps::new().start(update.start_timestamp as i64));

    let mut assets = activity::Assets::new().large_text(update.title.as_str());
    if let Some(url) = &update.cover_url {
        assets = assets.large_image(url.as_str());
    }
    activity = activity.assets(assets);

    if !update.buttons.is_empty() {
        let buttons: Vec<activity::Button> = update
            .buttons
            .iter()
            .map(|(label, url)| activity::Button::new(label.as_str(), url.as_str()))
            .collect();
        activity = activity.buttons(buttons);
    }

    client.set_activity(activity).is_ok()
}

fn worker_loop(client_id: String, rx: mpsc::Receiver<WorkerCommand>) {
    let mut client: Option<DiscordIpcClient> = None;
    let mut connected = false;
    let mut last: Option<PresenceUpdate> = None;

    let ensure_connected = |client: &mut Option<DiscordIpcClient>, connected: &mut bool| {
        if *connected {
            return;
        }
        let mut fresh = DiscordIpcClient::new(&client_id);
        match fresh.connect() {
            Ok(()) => {
                tracing::info!("connected to Discord Rich Presence");
                *client = Some(fresh);
                *connected = true;
            }
            Err(e) => {
                tracing::warn!("discord connect failed: {e}");
            }
        }
    };

    for command in rx {
        match command {
            WorkerCommand::Shutdown => break,
            WorkerCommand::Clear => {
                last = None;
                if connected {
                    if let Some(client) = client.as_mut() {
                        if let Err(e) = client.clear_activity() {
                            tracing::warn!("discord clear failed: {e}");
                            connected = false;
                        }
                    }
                }
            }
            WorkerCommand::Set(update) => {
                if !should_send(&last, &update) {
                    continue;
                }
                ensure_connected(&mut client, &mut connected);
                if !connected {
                    continue;
                }
                if let Some(client) = client.as_mut() {
                    if send_update(client, &update) {
                        tracing::info!(title = update.title.as_str(), "discord activity set");
                        last = Some(update);
                    } else {
                        tracing::warn!(title = update.title.as_str(), "discord send failed");
                        connected = false;
                    }
                }
    }
        }
    }

    if let Some(mut client) = client {
        let _ = client.close();
    }
}

pub struct PresenceHandle {
    tx: Sender<WorkerCommand>,
}

impl PresenceHandle {
    pub fn set_playing(&self, update: PresenceUpdate) {
        let _ = self.tx.send(WorkerCommand::Set(update));
    }

    pub fn clear(&self) {
        let _ = self.tx.send(WorkerCommand::Clear);
    }
}

impl Drop for PresenceHandle {
    fn drop(&mut self) {
        let _ = self.tx.send(WorkerCommand::Shutdown);
    }
}

pub fn spawn_presence_worker(client_id: &str) -> (PresenceHandle, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let client_id = client_id.to_string();
    let handle = std::thread::Builder::new()
        .name("poketto-discord".to_string())
        .spawn(move || worker_loop(client_id, rx))
        .expect("discord worker thread spawns");
    (PresenceHandle { tx }, handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> PresenceUpdate {
        PresenceUpdate::playing(
            "Muv-Luv",
            Some("Age"),
            Some("https://example.com/cover.jpg"),
            vec![("View on VNDB".to_string(), "https://vndb.org/v17".to_string())],
            1700000000,
        )
    }

    #[test]
    fn identical_update_is_skipped() {
        let update = sample();
        assert_eq!(should_send(&None, &update), true);
        assert_eq!(should_send(&Some(update.clone()), &update), false);
        let mut changed = update.clone();
        changed.title = "Other".to_string();
        assert_eq!(should_send(&Some(update), &changed), true);
    }

    #[test]
    fn playing_defaults_state_text() {
        let update = PresenceUpdate::playing("Title", None, None, Vec::new(), 0);
        assert_eq!(update.state, DEFAULT_STATE_TEXT);
        assert_eq!(update.buttons.len(), 0);
    }

    #[test]
    fn buttons_cap_at_two() {
        let buttons = vec![
            ("a".to_string(), "https://a".to_string()),
            ("b".to_string(), "https://b".to_string()),
            ("c".to_string(), "https://c".to_string()),
        ];
        let update = PresenceUpdate::playing("Title", None, None, buttons, 0);
        assert_eq!(update.buttons.len(), 2);
    }

    #[test]
    fn worker_survives_without_discord() {
        let (handle, worker) = spawn_presence_worker("000000000000000000");
        handle.set_playing(sample());
        handle.clear();
        drop(handle);
        worker.join().expect("worker exits");
    }
}
