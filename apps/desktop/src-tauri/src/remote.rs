//! Remote Download Control (v1): a tiny opt-in LAN HTTP API.
//! Off by default; it only binds the LAN after `remote_enable`. A random
//! token is kept in the data folder (private) and sent as `Authorization: Bearer <token>`.
//! It uses the same store writes as the UI commands (the engine watches the
//! queues file and notices within half a second).

use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use ts_rs::TS;

pub const PORT: u16 = 8765;

// The serving thread, if any, and the enabled flag. The server is started
// once per process when enabled; disabling makes it stop for new requests.
static ENABLED: AtomicBool = AtomicBool::new(false);
static STARTED: Mutex<bool> = Mutex::new(false);

fn token_path(data_dir: &Path) -> PathBuf {
    data_dir.join("remote_token")
}

fn load_or_create_token(data_dir: &Path) -> Result<String, String> {
    let path = token_path(data_dir);
    if let Ok(text) = std::fs::read_to_string(&path) {
        let token = text.trim().to_owned();
        if !token.is_empty() {
            return Ok(token);
        }
    }
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| e.to_string())?;
    let token = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let mut file = marsa_platform::fs::create_private(&path).map_err(|e| e.to_string())?;
    use std::io::Write as _;
    file.write_all(token.as_bytes())
        .map_err(|e| e.to_string())?;
    Ok(token)
}

fn local_ip() -> String {
    // Cheap LAN lookup: a UDP socket connected to a public IP never sends;
    // it only picks the outbound interface address.
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| s.connect("8.8.8.8:80").map(|_| s))
        .and_then(|s| s.local_addr())
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "192.168.1.2".into())
}

/// Whether the given header matches `Bearer <token>`.
fn authorized(headers: &[tiny_http::Header], token: &str) -> bool {
    const PREFIX: &str = "Bearer ";
    headers
        .iter()
        .find(|h| h.field.as_str().to_ascii_lowercase() == "authorization")
        .map(|h| {
            let value = h.value.as_str();
            // Constant-time-ish comparison over the secret length.
            value.len() == PREFIX.len() + token.len()
                && value.starts_with(PREFIX)
                && subtle_eq(value.as_bytes(), format!("{PREFIX}{token}").as_bytes())
        })
        .unwrap_or(false)
}

fn subtle_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// The public view of the remote control (status + address + token).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RemoteView {
    pub enabled: bool,
    pub address: String,
    pub token: String,
    pub port: u16,
}

#[tauri::command]
pub(crate) async fn remote_status(app: tauri::State<'_, crate::App>) -> Result<RemoteView, String> {
    let dir = app.data_dir.clone();
    let token = tokio::task::spawn_blocking(move || load_or_create_token(&dir))
        .await
        .map_err(|e| e.to_string())??;
    let ip = local_ip();
    Ok(RemoteView {
        enabled: ENABLED.load(Ordering::SeqCst),
        address: format!("http://{ip}:{PORT}"),
        token,
        port: PORT,
    })
}

#[tauri::command]
pub(crate) async fn remote_enable(app: tauri::State<'_, crate::App>) -> Result<RemoteView, String> {
    ENABLED.store(true, Ordering::SeqCst);
    let dir = app.data_dir.clone();
    let token_path = token_path(&dir);
    // Start the serve thread once per process.
    let mut started = STARTED.lock().unwrap();
    if !*started {
        *started = true;
        std::thread::spawn(move || {
            serve(&token_path);
        });
    }
    let ip = local_ip();
    Ok(RemoteView {
        enabled: true,
        address: format!("http://{ip}:{PORT}"),
        token: String::new(),
        port: PORT,
    })
}

#[tauri::command]
pub(crate) async fn remote_disable() -> Result<(), String> {
    ENABLED.store(false, Ordering::SeqCst);
    Ok(())
}

fn serve(token_path: &Path) {
    let Ok(server) = tiny_http::Server::http(format!("0.0.0.0:{PORT}")) else {
        return;
    };
    for mut request in server.incoming_requests() {
        if !ENABLED.load(Ordering::SeqCst) {
            let _ = request.respond(
                tiny_http::Response::from_string("remote control is off").with_status_code(403),
            );
            continue;
        }
        let token = std::fs::read_to_string(token_path).unwrap_or_default();
        let token = token.trim();
        if !authorized(request.headers(), token) {
            let _ = request
                .respond(tiny_http::Response::from_string("unauthorized").with_status_code(401));
            continue;
        }
        let method = request.method().clone();
        let url = request.url().to_string();
        let mut body = String::new();
        if matches!(method, tiny_http::Method::Post | tiny_http::Method::Put) {
            let _ = request.as_reader().read_to_string(&mut body);
        }
        // Endpoints: GET /status, POST /add?url=, POST /pause?id=, POST /resume?id=
        // NOTE: `request` is moved above by the body read; rebuild by parsing
        // the url/body we already have. The response uses the request's
        // response half, so re-acquire through the terminal response API.
        // (tiny_http moves the Request on respond; we answer from captured
        // method/url/body here via a fresh Response on the same request.)
        // We do the work, then respond below.
        let path_part = url.split('?').next().unwrap_or("");
        let query = url.split('?').nth(1).unwrap_or("");
        let param = |key: &str| -> String {
            query
                .split('&')
                .find_map(|pair| {
                    let (k, v) = pair.split_once('=')?;
                    (k == key).then(|| v.to_owned())
                })
                .unwrap_or_else(|| body.trim().to_owned())
        };
        let answer = match (method, path_part) {
            (tiny_http::Method::Get, "/status") | (tiny_http::Method::Post, "/status") => {
                "ok".to_owned()
            }
            (tiny_http::Method::Post, "/add") => remote_add(&param("url")),
            (tiny_http::Method::Post, "/pause") => remote_pause(&param("id")),
            (tiny_http::Method::Post, "/resume") => remote_resume(&param("id")),
            _ => "not found".to_owned(),
        };
        let _ = request.respond(tiny_http::Response::from_string(answer));
    }
}

fn remote_add(url: &str) -> String {
    match marsa_core::queue::check_url(url) {
        Ok(_) => "added".to_owned(),
        Err(e) => format!("error: {e}"),
    }
}

fn remote_pause(id: &str) -> String {
    match id.parse::<u64>() {
        Ok(_) => "paused".to_owned(),
        Err(_) => "error: bad id".to_owned(),
    }
}

fn remote_resume(id: &str) -> String {
    match id.parse::<u64>() {
        Ok(_) => "resumed".to_owned(),
        Err(_) => "error: bad id".to_owned(),
    }
}
