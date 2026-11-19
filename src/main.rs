#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

/// 一般公開用のデフォルト Discord Application ID (Client ID)
/// 公開前にここをご自身の公開用 Application ID（数字のみ）に書き換えてください。
const DEFAULT_CLIENT_ID: &str = "YOUR_PUBLIC_DISCORD_APPLICATION_ID";

/// Developer Portal の Art Assets に登録したアイコン名（ジャケット取得時は右下アイコンに使用）
const DEFAULT_LARGE_IMAGE: &str = "ytm_logo";

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const TICKS_PER_SEC: i64 = 10_000_000;
const WINDOWS_TO_UNIX_EPOCH_SECS: i64 = 11_644_473_600;

struct Config {
    client_id: String,
    large_image: String,
    app_filters: Vec<String>,
}

impl Config {
    fn load() -> Self {
        let mut client_id = env::var("DISCORD_CLIENT_ID")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string());
        let mut large_image = DEFAULT_LARGE_IMAGE.to_string();
        let mut app_filters: Vec<String> = Vec::new();

        if let Some(cfg_path) = config_file_path() {
            if let Ok(content) = fs::read_to_string(&cfg_path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    if let Some((k, v)) = trimmed.split_once('=') {
                        let key = k.trim();
                        let val = v.trim();
                        match key {
                            "CLIENT_ID" if !val.is_empty() && val != "YOUR_DISCORD_APPLICATION_ID" => {
                                client_id = val.to_string();
                            }
                            "LARGE_IMAGE" if !val.is_empty() => {
                                large_image = val.to_string();
                            }
                            "APP_FILTERS" => {
                                app_filters = val
                                    .split(',')
                                    .map(|s| s.trim().to_lowercase())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                            }
                            _ => {}
                        }
                    }
                }
            } else {
                // 上級者向けの任意設定ファイル（空欄のままでデフォルト設定が動作します）
                let template = "\
# 通常は編集不要です（空欄のままアプリ内蔵のデフォルトIDで動作します）
# 自分で作成したカスタムIDで上書きしたい場合のみ入力してください
CLIENT_ID=

# アイコン名を変更したい場合のみ入力（空欄でデフォルトを使用）
LARGE_IMAGE=

# 特定のアプリのみ監視したい場合はカンマ区切りで指定（空欄なら再生中のメディア全般）
# 例: APP_FILTERS=chrome,msedge,youtube
APP_FILTERS=
";
                let _ = fs::write(&cfg_path, template);
            }
        }

        Self {
            client_id,
            large_image,
            app_filters,
        }
    }

    fn matches_filter(&self, app_id: &str) -> bool {
        if self.app_filters.is_empty() {
            return true;
        }
        let lower = app_id.to_lowercase();
        self.app_filters.iter().any(|f| lower.contains(f))
    }
}

fn exe_dir() -> Option<PathBuf> {
    env::current_exe().ok()?.parent().map(|p| p.to_path_buf())
}

fn config_file_path() -> Option<PathBuf> {
    Some(exe_dir()?.join("config.txt"))
}

fn append_log(message: &str) {
    if let Some(dir) = exe_dir() {
        let log_path = dir.join("ytm-discord-rpc.log");
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = writeln!(file, "[{}] {}", now_unix_secs(), message);
        }
    }
}

fn clean_artist_name(artist: &str) -> String {
    artist.trim_end_matches(" - Topic").trim().to_string()
}

fn query_itunes_artwork(agent: &ureq::Agent, term: &str, country: &str) -> Option<String> {
    let resp = agent
        .get("https://itunes.apple.com/search")
        .query("term", term)
        .query("media", "music")
        .query("entity", "song")
        .query("limit", "1")
        .query("country", country)
        .call()
        .ok()?;
    let body = resp.into_string().ok()?;
    let parsed: Value = serde_json::from_str(&body).ok()?;
    let raw_url = parsed
        .get("results")?
        .as_array()?
        .first()?
        .get("artworkUrl100")?
        .as_str()?;
    Some(raw_url.replace("100x100bb", "512x512bb"))
}

fn fetch_artwork_url(
    agent: &ureq::Agent,
    cache: &mut HashMap<String, Option<String>>,
    title: &str,
    artist: &str,
) -> Option<String> {
    if title.is_empty() {
        return None;
    }

    let key = format!("{title}\0{artist}");
    if let Some(cached) = cache.get(&key) {
        return cached.clone();
    }

    let term = if artist.is_empty() {
        title.to_string()
    } else {
        format!("{title} {artist}")
    };

    let found = query_itunes_artwork(agent, &term, "JP")
        .or_else(|| query_itunes_artwork(agent, &term, "US"))
        .or_else(|| {
            if !artist.is_empty() {
                query_itunes_artwork(agent, title, "JP")
            } else {
                None
            }
        });

    if cache.len() >= 200 {
        cache.clear();
    }
    cache.insert(key, found.clone());
    found
}

#[derive(Debug, Clone)]
struct TrackInfo {
    app_id: String,
    title: String,
    artist: String,
    album: String,
    start_unix_sec: Option<i64>,
    end_unix_sec: Option<i64>,
    artwork_url: Option<String>,
}

impl TrackInfo {
    fn is_same_as(&self, other: &TrackInfo) -> bool {
        if self.title != other.title || self.artist != other.artist || self.album != other.album {
            return false;
        }
        match (self.start_unix_sec, other.start_unix_sec) {
            (Some(a), Some(b)) => (a - b).abs() <= 3,
            (None, None) => true,
            _ => false,
        }
    }
}

struct DiscordIpc {
    pipe: File,
    client_id: String,
}

impl DiscordIpc {
    fn connect(client_id: &str) -> io::Result<Self> {
        let mut last_err = None;
        for i in 0..10 {
            let path = format!(r"\\.\pipe\discord-ipc-{i}");
            match OpenOptions::new().read(true).write(true).open(&path) {
                Ok(pipe) => {
                    let mut ipc = Self {
                        pipe,
                        client_id: client_id.to_string(),
                    };
                    ipc.handshake()?;
                    return Ok(ipc);
                }
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Discord IPC pipe not found")
        }))
    }

    fn handshake(&mut self) -> io::Result<()> {
        let payload = json!({
            "v": 1,
            "client_id": self.client_id,
        });
        self.send_frame(0, &payload)
    }

    fn set_listening_activity(
        &mut self,
        track: &TrackInfo,
        fallback_logo: &str,
    ) -> io::Result<()> {
        let mut activity = serde_json::Map::new();
        // ActivityType::Listening = 2
        activity.insert("type".to_string(), json!(2));
        // StatusDisplayType: 0=Name(アプリ名), 1=State(アーティスト), 2=Details(曲名)
        activity.insert("status_display_type".to_string(), json!(2));

        let details = if track.title.is_empty() {
            "Unknown Track"
        } else {
            &track.title
        };
        activity.insert("details".to_string(), json!(details));

        if !track.artist.is_empty() {
            activity.insert("state".to_string(), json!(track.artist));
        }

        if let (Some(start), Some(end)) = (track.start_unix_sec, track.end_unix_sec) {
            activity.insert(
                "timestamps".to_string(),
                json!({
                    "start": start,
                    "end": end,
                }),
            );
        }

        let large_text = if !track.album.is_empty() {
            track.album.as_str()
        } else if !track.title.is_empty() {
            track.title.as_str()
        } else {
            "YouTube Music"
        };

        if let Some(art_url) = &track.artwork_url {
            let mut assets = serde_json::Map::new();
            assets.insert("large_image".to_string(), json!(art_url));
            assets.insert("large_text".to_string(), json!(large_text));
            if !fallback_logo.is_empty() {
                assets.insert("small_image".to_string(), json!(fallback_logo));
                assets.insert("small_text".to_string(), json!("YouTube Music"));
            }
            activity.insert("assets".to_string(), Value::Object(assets));
        } else if !fallback_logo.is_empty() {
            activity.insert(
                "assets".to_string(),
                json!({
                    "large_image": fallback_logo,
                    "large_text": large_text,
                }),
            );
        }

        let payload = json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": Value::Object(activity),
            },
            "nonce": next_nonce(),
        });
        self.send_frame(1, &payload)
    }

    fn clear_activity(&mut self) -> io::Result<()> {
        let payload = json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": Value::Null,
            },
            "nonce": next_nonce(),
        });
        self.send_frame(1, &payload)
    }

    fn send_frame(&mut self, opcode: u32, payload: &Value) -> io::Result<()> {
        let body = serde_json::to_vec(payload)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut header = [0u8; 8];
        header[0..4].copy_from_slice(&opcode.to_le_bytes());
        header[4..8].copy_from_slice(&(body.len() as u32).to_le_bytes());

        self.pipe.write_all(&header)?;
        self.pipe.write_all(&body)?;
        self.pipe.flush()?;

        let mut resp_header = [0u8; 8];
        self.pipe.read_exact(&mut resp_header)?;
        let resp_len = u32::from_le_bytes([
            resp_header[4],
            resp_header[5],
            resp_header[6],
            resp_header[7],
        ]) as usize;
        let mut resp_body = vec![0u8; resp_len];
        self.pipe.read_exact(&mut resp_body)?;

        Ok(())
    }
}

fn now_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn next_nonce() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("ytm-rpc-{ms}")
}

fn find_playing_session(
    manager: &GlobalSystemMediaTransportControlsSessionManager,
    config: &Config,
) -> windows::core::Result<Option<GlobalSystemMediaTransportControlsSession>> {
    if let Ok(sessions) = manager.GetSessions() {
        let count = sessions.Size()?;
        for i in 0..count {
            let session = sessions.GetAt(i)?;
            let app_id = session.SourceAppUserModelId()?.to_string();
            if !config.matches_filter(&app_id) {
                continue;
            }
            if let Ok(info) = session.GetPlaybackInfo() {
                if info.PlaybackStatus()?
                    == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
                {
                    return Ok(Some(session));
                }
            }
        }
    }

    if let Ok(session) = manager.GetCurrentSession() {
        let app_id = session.SourceAppUserModelId()?.to_string();
        if config.matches_filter(&app_id) {
            if let Ok(info) = session.GetPlaybackInfo() {
                if info.PlaybackStatus()?
                    == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
                {
                    return Ok(Some(session));
                }
            }
        }
    }

    Ok(None)
}

fn read_playing_track(
    manager: &GlobalSystemMediaTransportControlsSessionManager,
    config: &Config,
) -> windows::core::Result<Option<TrackInfo>> {
    let Some(session) = find_playing_session(manager, config)? else {
        return Ok(None);
    };

    let app_id = session.SourceAppUserModelId()?.to_string();
    let props = session.TryGetMediaPropertiesAsync()?.get()?;
    let title = props.Title()?.to_string();
    let artist = clean_artist_name(&props.Artist()?.to_string());
    let album = props.AlbumTitle()?.to_string();

    if title.is_empty() && artist.is_empty() {
        return Ok(None);
    }

    let (start_unix_sec, end_unix_sec) = match session.GetTimelineProperties() {
        Ok(timeline) => {
            let start_ticks = timeline.StartTime().map(|t| t.Duration).unwrap_or(0);
            let end_ticks = timeline.EndTime().map(|t| t.Duration).unwrap_or(0);
            let pos_ticks = timeline.Position().map(|t| t.Duration).unwrap_or(0);
            let duration_secs = (end_ticks - start_ticks) / TICKS_PER_SEC;

            if duration_secs > 0 {
                let now = now_unix_secs();
                let base_pos_secs = ((pos_ticks - start_ticks) / TICKS_PER_SEC).max(0);
                let extra_secs = timeline
                    .LastUpdatedTime()
                    .ok()
                    .map(|dt| dt.UniversalTime)
                    .filter(|&ut| ut > 0)
                    .map(|ut| {
                        let updated_unix = (ut / TICKS_PER_SEC) - WINDOWS_TO_UNIX_EPOCH_SECS;
                        (now - updated_unix).max(0)
                    })
                    .unwrap_or(0);

                let current_pos = (base_pos_secs + extra_secs).min(duration_secs);
                let start = now - current_pos;
                (Some(start), Some(start + duration_secs))
            } else {
                (None, None)
            }
        }
        Err(_) => (None, None),
    };

    Ok(Some(TrackInfo {
        app_id,
        title,
        artist,
        album,
        start_unix_sec,
        end_unix_sec,
        artwork_url: None,
    }))
}

fn main() -> windows::core::Result<()> {
    let config = Config::load();
    if config.client_id.is_empty() || config.client_id == "YOUR_PUBLIC_DISCORD_APPLICATION_ID" {
        append_log("DEFAULT_CLIENT_ID が未設定です。src/main.rs の DEFAULT_CLIENT_ID を設定してください。");
        return Ok(());
    }

    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.get()?;
    let http_agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .build();
    let mut artwork_cache: HashMap<String, Option<String>> = HashMap::new();
    let mut ipc: Option<DiscordIpc> = None;
    let mut last_track: Option<TrackInfo> = None;

    append_log("監視を開始しました。");

    loop {
        if ipc.is_none() {
            match DiscordIpc::connect(&config.client_id) {
                Ok(client) => {
                    append_log("Discord IPC に接続しました。");
                    ipc = Some(client);
                    last_track = None;
                }
                Err(_) => {
                    thread::sleep(POLL_INTERVAL);
                    continue;
                }
            }
        }

        let mut current_track = read_playing_track(&manager, &config).unwrap_or(None);

        if let Some(client) = ipc.as_mut() {
            let result = match (&mut current_track, &last_track) {
                (Some(track), Some(prev)) if track.is_same_as(prev) => Ok(()),
                (Some(track), _) => {
                    track.artwork_url = fetch_artwork_url(
                        &http_agent,
                        &mut artwork_cache,
                        &track.title,
                        &track.artist,
                    );
                    append_log(&format!(
                        "Playing: {} - {} (art: {}, app: {})",
                        track.title,
                        track.artist,
                        track.artwork_url.is_some(),
                        track.app_id
                    ));
                    client.set_listening_activity(track, &config.large_image)
                }
                (None, Some(_)) => {
                    append_log("再生停止または一時停止のためステータスをクリアしました。");
                    client.clear_activity()
                }
                (None, None) => Ok(()),
            };

            if let Err(err) = result {
                append_log(&format!("Discord IPC エラー（再接続待機）: {err}"));
                ipc = None;
                last_track = None;
            } else {
                last_track = current_track;
            }
        }

        thread::sleep(POLL_INTERVAL);
    }
}
