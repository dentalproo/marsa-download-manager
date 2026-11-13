//! Torrent downloads with librqbit (download-only by default).
//!
//! Torrent state (torrents, peers, seeders) belongs to librqbit's
//! session; this module keeps the [`Api`] in a lazy `OnceLock` and adds
//! torrents paused until the user presses start.
//! Seeding is **never** started automatically: quantifiably the user's
//! bandwidth stays. Manual start from the Torrents UI moves it to live.

use std::sync::OnceLock;

use librqbit::{AddTorrent, Api, SessionOptions};

static API: OnceLock<Api> = OnceLock::new();

fn api() -> &'static Api {
    API.get_or_init(|| {
        // Download-only ergonomics: upload capped to 1 byte/s (user asked for
        // zero sharing in Egypt where mobile data is precious). librqbit
        // still needs a trickle to stay in the swarm; 1 B/s is effectively
        // download-only while keeping the peer connection alive.
        let session = tokio::runtime::Handle::try_current()
            .map(|rt| {
                rt.block_on(librqbit::Session::new_with_opts(
                    std::env::temp_dir().join("Marsa-Torrents"),
                    SessionOptions {
                        ratelimits: librqbit::limits::LimitsConfig {
                            upload_bps: std::num::NonZeroU32::new(1),
                            download_bps: None,
                        },
                        ..Default::default()
                    },
                ))
            })
            .unwrap_or_else(|_| panic!("torrent session needs a tokio runtime"));
        let session = session.expect("torrent session");
        Api::new(session, None)
    })
}

pub(crate) async fn add_magnet(magnet: &str) -> Result<String, String> {
    let _parsed = marsa_torrent::parse_magnet(magnet.trim())?;
    let api = api();
    // Start paused: the user sees the analysis first and only then starts
    // downloading (download-only until seeding is chosen on purpose).
    let opts = librqbit::AddTorrentOptions {
        paused: true,
        overwrite: true,
        ..Default::default()
    };
    let res = api
        .api_add_torrent(AddTorrent::Url(magnet.trim().into()), Some(opts))
        .await
        .map_err(|e| e.to_string())?;
    let handle = res.details;
    let name = handle.name.unwrap_or_else(|| "torrent".to_owned());
    Ok(format!("added: {name}"))
}

#[tauri::command]
pub(crate) async fn torrent_add(magnet: String) -> Result<String, String> {
    add_magnet(&magnet).await
}

#[tauri::command]
pub(crate) async fn torrent_list() -> Result<String, String> {
    let api = api();
    let list = api.api_torrent_list();
    // Structured JSON for the Torrents cards (names, pieces, speeds, ratio).
    serde_json::to_string(&list).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn torrent_pause(id: usize) -> Result<(), String> {
    let api = api();
    api.api_torrent_action_pause(id.into())
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn torrent_start(id: usize) -> Result<(), String> {
    let api = api();
    api.api_torrent_action_start(id.into())
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn torrent_select_files(id: usize, files: Vec<usize>) -> Result<(), String> {
    let api = api();
    let set = files.into_iter().collect();
    api.api_torrent_action_update_only_files(id.into(), &set)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn torrent_forget(id: usize) -> Result<(), String> {
    let api = api();
    api.api_torrent_action_forget(id.into())
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}
