use discord_rich_presence::{
    activity::{self, ActivityType, StatusDisplayType},
    DiscordIpc, DiscordIpcClient,
};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::{CustomPresence, PresenceActivityType};

pub struct PresenceContext<'a> {
    pub title: &'a str,
    pub state_fallback: &'a str,
    pub cover_url: Option<&'a str>,
    pub session_start_secs: u64,
}

pub fn resolve_placeholders(text: &str, ctx: &PresenceContext) -> String {
    text.replace("{{title}}", ctx.title)
        .replace("{{state}}", ctx.state_fallback)
        .replace("{{cover}}", ctx.cover_url.unwrap_or(""))
}

fn custom_buttons(preset: &CustomPresence) -> Vec<(String, String)> {
    [
        (&preset.button1_text, &preset.button1_url),
        (&preset.button2_text, &preset.button2_url),
    ]
    .into_iter()
    .filter_map(|(label, url)| {
        let label = label.as_deref().unwrap_or("").trim();
        let url = url.as_deref().unwrap_or("").trim();
        if label.is_empty() || url.is_empty() {
            None
        } else {
            Some((label.to_string(), url.to_string()))
        }
    })
    .collect()
}

const DEFAULT_CLIENT_ID: &str = "1454731999637147732";

pub struct DiscordRpc {
    client: Mutex<Option<DiscordIpcClient>>,
    connected: AtomicBool,
    active_client_id: Mutex<Option<String>>,
}

impl DiscordRpc {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
            connected: AtomicBool::new(false),
            active_client_id: Mutex::new(None),
        }
    }

    pub fn connect(&self) -> Result<(), String> {
        self.connect_with(DEFAULT_CLIENT_ID)
    }

    fn connect_with(&self, client_id: &str) -> Result<(), String> {
        if self.connected.load(Ordering::Acquire) {
            return Ok(());
        }

        let mut client_guard = self.client.lock();

        if self.connected.load(Ordering::Acquire) {
            return Ok(());
        }

        let mut client = DiscordIpcClient::new(client_id);

        client
            .connect()
            .map_err(|e| format!("Failed to connect to Discord: {}", e))?;

        *client_guard = Some(client);
        *self.active_client_id.lock() = Some(client_id.to_string());
        self.connected.store(true, Ordering::Release);
        log::info!("Connected to Discord Rich Presence");
        Ok(())
    }

    fn ensure_connected(&self, client_id: &str) -> Result<(), String> {
        let effective = {
            let trimmed = client_id.trim();
            if trimmed.is_empty() {
                DEFAULT_CLIENT_ID.to_string()
            } else {
                trimmed.to_string()
            }
        };
        let same = self.connected.load(Ordering::Acquire)
            && self.active_client_id.lock().as_deref() == Some(effective.as_str());
        if same {
            return Ok(());
        }
        self.disconnect();
        self.connect_with(&effective)
    }

    pub fn disconnect(&self) {
        let mut client_guard = self.client.lock();
        if let Some(ref mut client) = *client_guard {
            let _ = client.close();
        }
        *client_guard = None;
        self.connected.store(false, Ordering::Release);
        log::info!("Disconnected from Discord Rich Presence");
    }

    pub fn set_activity(
        &self,
        game_title: &str,
        cover_url: Option<&str>,
        custom_state: &str,
        buttons: Vec<(&str, &str)>,
        start_timestamp: u64,
    ) -> Result<(), String> {
        if !self.connected.load(Ordering::Acquire) && self.connect().is_err() {
            return Ok(());
        }

        let mut client_guard = self.client.lock();
        let client = match client_guard.as_mut() {
            Some(c) => c,
            None => return Ok(()),
        };

        let mut activity_builder = activity::Activity::new()
            .details(game_title)
            .state(custom_state)
            .activity_type(ActivityType::Playing)
            .status_display_type(StatusDisplayType::Details);

        let timestamps = activity::Timestamps::new().start(start_timestamp as i64);
        activity_builder = activity_builder.timestamps(timestamps);

        let mut assets = activity::Assets::new().large_text(game_title);

        if let Some(url) = cover_url {
            assets = assets.large_image(url);
        }

        activity_builder = activity_builder.assets(assets);

        if !buttons.is_empty() {
            let button_list: Vec<activity::Button> = buttons
                .into_iter()
                .take(2)
                .map(|(label, url)| activity::Button::new(label, url))
                .collect();
            activity_builder = activity_builder.buttons(button_list);
        }

        match client.set_activity(activity_builder) {
            Ok(_) => {
                log::info!("Discord activity set: {}", game_title);
                Ok(())
            }
            Err(e) => {
                self.connected.store(false, Ordering::Release);
                log::warn!("Failed to set Discord activity: {}", e);
                Ok(())
            }
        }
    }

    pub fn set_custom_activity(
        &self,
        preset: &CustomPresence,
        ctx: &PresenceContext,
    ) -> Result<(), String> {
        if self
            .ensure_connected(preset.client_id.as_deref().unwrap_or(""))
            .is_err()
        {
            return Ok(());
        }
        if !self.connected.load(Ordering::Acquire) {
            return Ok(());
        }

        let mut client_guard = self.client.lock();
        let client = match client_guard.as_mut() {
            Some(c) => c,
            None => return Ok(()),
        };

        let activity_type = match preset.activity_type {
            Some(PresenceActivityType::Listening) => ActivityType::Listening,
            Some(PresenceActivityType::Watching) => ActivityType::Watching,
            Some(PresenceActivityType::Competing) => ActivityType::Competing,
            _ => ActivityType::Playing,
        };
        let name = preset
            .name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("Poketto");
        let details_raw = resolve_placeholders(preset.details.as_deref().unwrap_or(""), ctx);
        let details = if details_raw.trim().is_empty() {
            ctx.title.to_string()
        } else {
            details_raw
        };
        let state_raw = resolve_placeholders(preset.state.as_deref().unwrap_or(""), ctx);
        let state = if state_raw.trim().is_empty() {
            ctx.state_fallback.to_string()
        } else {
            state_raw
        };

        let mut activity_builder = activity::Activity::new()
            .name(name)
            .details(details)
            .state(state)
            .activity_type(activity_type)
            .status_display_type(StatusDisplayType::Details);

        let details_url = resolve_placeholders(preset.details_url.as_deref().unwrap_or(""), ctx);
        if !details_url.trim().is_empty() {
            activity_builder = activity_builder.details_url(details_url);
        }
        let state_url = resolve_placeholders(preset.state_url.as_deref().unwrap_or(""), ctx);
        if !state_url.trim().is_empty() {
            activity_builder = activity_builder.state_url(state_url);
        }

        let timestamps = match preset.timestamp_mode {
            Some(crate::models::PresenceTimestampMode::Custom) => {
                let mut timestamps = activity::Timestamps::new();
                let mut any = false;
                if let Some(start) = preset.custom_start {
                    timestamps = timestamps.start(start.saturating_mul(1000));
                    any = true;
                }
                if let Some(end) = preset.custom_end {
                    timestamps = timestamps.end(end.saturating_mul(1000));
                    any = true;
                }
                any.then_some(timestamps)
            }
            _ => Some(
                activity::Timestamps::new()
                    .start(ctx.session_start_secs.saturating_mul(1000) as i64),
            ),
        };
        if let Some(timestamps) = timestamps {
            activity_builder = activity_builder.timestamps(timestamps);
        }

        if let (Some(size), Some(max)) = (preset.party_size, preset.party_max) {
            activity_builder = activity_builder.party(activity::Party::new().size([
                size.min(i32::MAX as u32) as i32,
                max.min(i32::MAX as u32) as i32,
            ]));
        }

        let large_image = resolve_placeholders(preset.large_image.as_deref().unwrap_or(""), ctx);
        let large_text = resolve_placeholders(preset.large_text.as_deref().unwrap_or(""), ctx);
        let large_url = resolve_placeholders(preset.large_url.as_deref().unwrap_or(""), ctx);
        let small_image = resolve_placeholders(preset.small_image.as_deref().unwrap_or(""), ctx);
        let small_text = resolve_placeholders(preset.small_text.as_deref().unwrap_or(""), ctx);
        let small_url = resolve_placeholders(preset.small_url.as_deref().unwrap_or(""), ctx);
        if [&large_image, &large_text, &large_url, &small_image, &small_text, &small_url]
            .iter()
            .any(|s| !s.trim().is_empty())
        {
            let mut assets = activity::Assets::new();
            if !large_image.trim().is_empty() {
                assets = assets.large_image(large_image);
            }
            if !large_text.trim().is_empty() {
                assets = assets.large_text(large_text);
            }
            if !large_url.trim().is_empty() {
                assets = assets.large_url(large_url);
            }
            if !small_image.trim().is_empty() {
                assets = assets.small_image(small_image);
            }
            if !small_text.trim().is_empty() {
                assets = assets.small_text(small_text);
            }
            if !small_url.trim().is_empty() {
                assets = assets.small_url(small_url);
            }
            activity_builder = activity_builder.assets(assets);
        }

        let buttons = custom_buttons(preset);
        if !buttons.is_empty() {
            activity_builder = activity_builder
                .buttons(buttons.into_iter().map(|(label, url)| activity::Button::new(label, url)).collect());
        }

        match client.set_activity(activity_builder) {
            Ok(_) => {
                log::info!("Discord custom activity set: {}", name);
                Ok(())
            }
            Err(e) => {
                self.connected.store(false, Ordering::Release);
                log::warn!("Failed to set Discord custom activity: {}", e);
                Ok(())
            }
        }
    }


    pub fn clear_activity(&self) -> Result<(), String> {
        if !self.connected.load(Ordering::Acquire) {
            return Ok(());
        }

        let mut client_guard = self.client.lock();
        if let Some(ref mut client) = *client_guard {
            match client.clear_activity() {
                Ok(_) => {
                    log::info!("Discord activity cleared");
                }
                Err(e) => {
                    log::warn!("Failed to clear Discord activity: {}", e);
                    self.connected.store(false, Ordering::Release);
                }
            }
        }
        Ok(())
    }
}

impl Default for DiscordRpc {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DiscordRpc {
    fn drop(&mut self) {
        self.disconnect();
    }
}

pub fn get_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> PresenceContext<'static> {
        PresenceContext {
            title: "KoiChoco",
            state_fallback: "Idle",
            cover_url: Some("https://example.com/cover.jpg"),
            session_start_secs: 1700000000,
        }
    }

    #[test]
    fn test_placeholders_resolve_all_tokens() {
        let ctx = context();
        assert_eq!(
            resolve_placeholders("Hi {{title}} ({{state}}) {{cover}}!", &ctx),
            "Hi KoiChoco (Idle) https://example.com/cover.jpg!"
        );
    }

    #[test]
    fn test_placeholders_leave_unknown_tokens_intact() {
        let ctx = context();
        assert_eq!(
            resolve_placeholders("{{title}} {{mystery}}", &ctx),
            "KoiChoco {{mystery}}"
        );
    }

    #[test]
    fn test_custom_buttons_keep_ordered_complete_pairs() {
        let preset = CustomPresence {
            button1_text: Some("VNDB".to_string()),
            button1_url: Some("https://vndb.org".to_string()),
            button2_text: Some(" ".to_string()),
            button2_url: Some("https://example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(
            custom_buttons(&preset),
            vec![("VNDB".to_string(), "https://vndb.org".to_string())]
        );
    }
}
