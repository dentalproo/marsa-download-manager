//! «A new version of Marsa is available» (#124, owner's decision): once at
//! each start, after the window shows, if the settings allow it, Marsa asks
//! GitHub for its newest release (`marsa_core::update`). Not in a
//! development build, whose version (0.0.0) would always find one. A
//! failure (no network, GitHub's limit, an odd answer) is one line in the
//! log, nothing for the user. The page shows a bar; «Release page» opens
//! the release's page from Rust, the page never holding a link.
//!
//! The signed update (ADR-0011, owner's request of 2026-10-31): the newest
//! release's `latest.json` names, for this system, its file and that
//! file's signature. The engine checks it (`marsa_core::update::check`):
//! signed by one of the project's two keys built into it, the file in the
//! project's own release downloads, and the signature's trusted comment
//! naming the announced version. The same manifest is then handed to
//! Tauri's updater with that key (`restore_update`), so Tauri's check and
//! Marsa's are the same key and the same text. Nothing is downloaded before
//! the user says so; then the file is checked by the engine again with that
//! key, the downloads stop after saving their progress, and Marsa is
//! replaced and opened again. A `.deb` has no file in `latest.json`: the
//! notice and the release page only. «Skip this version» hides that one.

use std::sync::Mutex;

use marsa_core::update::{self, CheckError, NewRelease, Offer};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};
use ts_rs::TS;

use crate::App;

/// The newer release found at this start, if any, and its signed update
/// when this system has one: the manifest Marsa checked, and Tauri's view
/// of the same manifest (which downloads and installs it).
#[derive(Default)]
pub struct Notice(Mutex<Option<NewRelease>>, Mutex<Option<(Offer, Update)>>);

/// What the page shows of a newer release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateView {
    pub version: String,
    /// It can be installed from here («Update now»): a signed update.
    pub can_install: bool,
}

/// How far the update's download is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateProgressView {
    #[ts(type = "number")]
    pub done: u64,
    #[ts(type = "number | null")]
    pub total: Option<u64>,
}

/// The signed update of this release for this system, if any: by a key of
/// ours, from the project's own downloads, for the version it announces.
/// Its absence is a line in the log at most.
async fn signed_update<R: tauri::Runtime>(
    handle: &tauri::AppHandle<R>,
    engine: &marsa_core::Engine,
    current: &str,
    stop: &marsa_core::Stop,
) -> Option<(Offer, Update)> {
    let offer = match update::check(engine, current, stop).await {
        Ok(Some(offer)) => offer,
        Ok(None) => return None,
        // A system with no file in it (a `.deb`) says so here too.
        Err(e) => {
            eprintln!("marsa: no signed update to install: {e}");
            return None;
        }
    };
    let (name, key) = offer.key()?;
    // Tauri's updater downloads and installs from the very manifest Marsa
    // checked, with the key that signed it, and refuses a file that is not
    // signed for that version (`require_signed_version`).
    let built = update::MANIFEST
        .parse::<tauri::Url>()
        .map_err(|e| e.to_string())
        .and_then(|url| {
            handle
                .updater_builder()
                .endpoints(vec![url])
                .and_then(|b| b.pubkey(key).build())
                .map_err(|e| e.to_string())
        });
    let raw: serde_json::Value = match serde_json::from_str(&offer.raw) {
        Ok(raw) => raw,
        Err(e) => {
            eprintln!("marsa: the update's manifest: {e}");
            return None;
        }
    };
    let update = match built {
        Ok(updater) => match updater.restore_update(raw) {
            Ok(Some(update)) => update,
            Ok(None) => return None,
            Err(e) => {
                eprintln!("marsa: the update: {e}");
                return None;
            }
        },
        Err(e) => {
            eprintln!("marsa: the update: {e}");
            return None;
        }
    };
    // Tauri's view of the manifest must be the one Marsa checked: same
    // version, same file, same signature. It reads the same text, but the
    // two parsers are not the same code.
    if update.version != offer.version
        || update.download_url.as_str() != offer.url
        || update.signature != offer.signature
    {
        eprintln!("marsa: Tauri's update differs from the checked one; not offered");
        return None;
    }
    eprintln!("marsa: the update is signed with the {name} key");
    Some((offer, update))
}

/// The check at start, in the background: the window is not kept waiting.
pub fn check_at_start(handle: &tauri::AppHandle) {
    // A development build is 0.0.0: every release would be newer.
    if cfg!(debug_assertions) {
        return;
    }
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let (settings, engine) = {
            let app = handle.state::<App>();
            (app.settings.clone(), app.engine.clone())
        };
        let on = tokio::task::spawn_blocking(move || settings.load().map(|s| s.update_check))
            .await
            .ok()
            .and_then(Result::ok)
            .unwrap_or(true);
        // The video tools' signed list (ADR-0011, section 2): asked with
        // the update, kept when signed and newer; the tools then update
        // the next time a video needs them.
        if on {
            // The program's own stop: quitting ends the fetch at once.
            let (engine, data, stop) = {
                let app = handle.state::<App>();
                (app.engine.clone(), app.data_dir.clone(), app.stop.clone())
            };
            tauri::async_runtime::spawn(async move {
                match marsa_core::tools_list::fetch_and_keep(&engine, &data, &stop).await {
                    Ok(Some(list)) => eprintln!(
                        "marsa: the video tools list {} is kept: yt-dlp {}",
                        list.sequence,
                        list.pins(marsa_core::tools::Tool::YtDlp).version
                    ),
                    Ok(None) => {}
                    Err(e) => eprintln!("marsa: the video tools list: {e}"),
                }
            });
        }
        let current = handle.package_info().version.to_string();
        let stop = marsa_core::Stop::new();
        let found = found(on, || {
            marsa_core::update::newer_release(&engine, &current, &stop)
        })
        .await;
        let Some(release) = found else { return };
        let settings = handle.state::<App>().settings.clone();
        let skipped = tokio::task::spawn_blocking(move || settings.load().map(|s| s.skip_update))
            .await
            .ok()
            .and_then(Result::ok)
            .flatten();
        if skipped.as_deref() == Some(release.version.as_str()) {
            return;
        }
        let update = signed_update(&handle, &engine, &current, &stop).await;
        let view = UpdateView {
            version: release.version.clone(),
            can_install: update.is_some(),
        };
        let notice = handle.state::<Notice>();
        *notice.0.lock().unwrap() = Some(release);
        *notice.1.lock().unwrap() = update;
        let _ = handle.emit_to("main", "update-available", view);
    });
}

/// A newer release found, as the check at the start records it, for the
/// tests.
#[cfg(test)]
pub fn found_for_test<R: tauri::Runtime>(handle: &tauri::AppHandle<R>, version: &str) {
    *handle.state::<Notice>().0.lock().unwrap() = Some(NewRelease {
        version: version.to_owned(),
        page: format!(
            "https://github.com/dentalproo/marsa-download-manager/releases/tag/v{version}"
        ),
    });
    let _ = handle.emit_to(
        "main",
        "update-available",
        UpdateView {
            version: version.to_owned(),
            can_install: false,
        },
    );
}

/// The newer release `check` finds, when the check is `on`; nothing is
/// asked when it is off, and a failure is only logged.
async fn found<F, Fut>(on: bool, check: F) -> Option<NewRelease>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Option<NewRelease>, CheckError>>,
{
    if !on {
        return None;
    }
    match check().await {
        Ok(found) => found,
        Err(err) => {
            eprintln!("marsa: the update check: {err}");
            None
        }
    }
}

/// The browser extension's version, when it said it is older than this
/// Marsa needs (`marsa_host::MIN_EXTENSION`, owner's request, 2026-10-31:
/// it does not update itself, and an old one misses what Marsa asks of it).
#[derive(Default)]
pub struct ExtensionOld(Mutex<Option<String>>);

impl ExtensionOld {
    /// The version it said, if it did.
    pub fn version(&self) -> Option<String> {
        self.0.lock().unwrap().clone()
    }
}

/// An old extension said its version: kept, and the main window told.
pub fn extension_old<R: tauri::Runtime>(handle: &tauri::AppHandle<R>, version: &str) {
    *handle.state::<ExtensionOld>().0.lock().unwrap() = Some(version.to_owned());
    let _ = handle.emit_to("main", "extension-old", version.to_owned());
}

/// The old extension's version, for a page that opens later.
#[tauri::command]
pub fn extension_old_notice(old: State<'_, ExtensionOld>) -> Option<String> {
    old.0.lock().unwrap().clone()
}

/// «How to update it»: the newest release's page, with the extension's
/// files and the steps to install them, in the default browser.
#[tauri::command]
pub fn open_extension_help<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    handle
        .opener()
        .open_url(
            "https://github.com/dentalproo/marsa-download-manager/releases/latest",
            None::<&str>,
        )
        .map_err(|e| e.to_string())
}

/// The newer release found at this start, for a page that opens later.
#[tauri::command]
pub fn update_notice(notice: State<'_, Notice>) -> Option<UpdateView> {
    let can_install = notice.1.lock().unwrap().is_some();
    notice.0.lock().unwrap().as_ref().map(|r| UpdateView {
        version: r.version.clone(),
        can_install,
    })
}

/// «Skip this version»: not shown again at the next starts; a newer one is.
#[tauri::command]
pub async fn update_skip<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> Result<(), String> {
    let notice = handle.state::<Notice>();
    let Some(version) = notice.0.lock().unwrap().take().map(|r| r.version) else {
        return Ok(());
    };
    notice.1.lock().unwrap().take();
    let settings = handle.state::<App>().settings.clone();
    tokio::task::spawn_blocking(move || {
        settings.update(|file| {
            file.skip_update = Some(version);
            Ok(())
        })
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

/// «Update now»: the file downloaded (its progress as `update-progress`),
/// checked by our own code with the key that signed it, then the
/// downloads stop after saving their progress, and Tauri's updater checks
/// it again, installs it, and Marsa opens again. On Windows the installer
/// ends Marsa itself and opens it again.
#[tauri::command]
pub async fn update_install<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> Result<(), String> {
    let Some((offer, update)) = handle.state::<Notice>().1.lock().unwrap().clone() else {
        return Err("no signed update was found".into());
    };
    let mut done = 0u64;
    let mut shown = std::time::Instant::now();
    let events = handle.clone();
    let bytes = update
        .download(
            |chunk, total| {
                done += chunk as u64;
                if shown.elapsed() >= std::time::Duration::from_millis(200) {
                    shown = std::time::Instant::now();
                    let _ = events.emit_to(
                        "main",
                        "update-progress",
                        UpdateProgressView { done, total },
                    );
                }
            },
            || {},
        )
        .await
        .map_err(|e| {
            eprintln!("marsa: the update could not be downloaded or checked: {e}");
            e.to_string()
        })?;
    // Tauri checked the signature with the key it was built with; Marsa
    // checks it again with the key that signed the manifest, and the
    // version the signature names.
    if !offer.verify(&bytes) {
        eprintln!("marsa: the update's file is not signed by a key of Marsa's; not installed");
        return Err("the update could not be checked".into());
    }
    eprintln!("marsa: installing the update to {}", offer.version);
    // As when quitting: the downloads save their progress and stop, and
    // continue after the update.
    {
        let app = handle.state::<App>();
        app.exiting.store(true, std::sync::atomic::Ordering::SeqCst);
        app.stop.request();
        crate::shutdown::wait_for_queues(app.stopped.subscribe(), crate::shutdown::STOP_WAIT).await;
    }
    if let Err(e) = update.install(&bytes) {
        // Opened again as it was: the bar comes back at its start.
        eprintln!("marsa: the update could not be installed: {e}");
    }
    handle.restart()
}

/// «Release page»: the found release's page in the default browser.
#[tauri::command]
pub fn open_release_page<R: tauri::Runtime>(
    notice: State<'_, Notice>,
    handle: tauri::AppHandle<R>,
) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let Some(page) = notice.0.lock().unwrap().as_ref().map(|r| r.page.clone()) else {
        return Err("no newer release was found".into());
    };
    handle
        .opener()
        .open_url(page, None::<&str>)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release() -> NewRelease {
        NewRelease {
            version: "0.2.0".into(),
            page: "https://github.com/dentalproo/marsa-download-manager/releases/tag/v0.2.0".into(),
        }
    }

    // Throwaway test keys (`tauri signer generate`, their private halves
    // not kept), and `FILE` signed with key A for version 9.9.9.
    const KEY_A: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDhFNDQ1NjE0NzYyMkMwOTIKUldTU3dDSjJGRlpFam4xZFEzU2FuKzAzdWEzeFU0cWJOVEF5VE5Va0YvZkEyeG0yKzJBTm1LOTkK";
    const KEY_B: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDcwNDE0ODg0RTExQzlCNkQKUldSdG14emhoRWhCY0NxUXp3TEEyeU1tdVA1bTJabkpFdVV3MzJjYkhMNVhpS1ErL0lJbCtUOHgK";
    const FILE: &[u8] = b"marsa update test file\n";
    const SIGNED_BY_A: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHJzaWduIHNlY3JldCBrZXkKUlVTU3dDSjJGRlpFanBMKzZnT0ovUk8xTFRsRjd6SlhKUC8wRnNpZEU4cEdHUUhmSm9XZnhncklOczViSS9RODhZUDczZXJUVFh4bUtKK0NJYUovOEdmYmM1MGhNdWRPaFFRPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzk0MDkyODIzCWZpbGU6QzpcVXNlcnNcRUwtSGFtZFxBcHBEYXRhXExvY2FsXFRlbXBcb3BlbmNvZGVcd29ya1xmaWxlLmJpbglwcmVoYXNoZWQKK2FCbnFWNjVITy9WZUw4NndYRDNudk5aVFNJd3FmbFFiSUg2Y0JWTE4vQUs2akxQUGNSYStkd1diK2lhUEZ1U21zZXhlVWt3TnlKbDdIQ1ZpZGgrQ2c9PQo=";

    /// ADR-0011: an update is offered and installed only when signed by a
    /// key of ours, the emergency one too; never with a key that is not
    /// there, nor a file changed after it was signed. The check itself is
    /// the engine's (`marsa_core::update`), used here as the desktop uses it.
    #[test]
    fn only_a_file_signed_by_one_of_our_keys_is_taken() {
        use marsa_core::update::{key_for, signed_by};
        assert_eq!(
            key_for(SIGNED_BY_A, &[("signing", KEY_A), ("emergency", KEY_B)]),
            Some(("signing", KEY_A))
        );
        assert_eq!(
            key_for(SIGNED_BY_A, &[("signing", KEY_B), ("emergency", KEY_A)]),
            Some(("emergency", KEY_A))
        );
        assert_eq!(
            key_for(SIGNED_BY_A, &[("signing", KEY_B), ("emergency", "")]),
            None
        );
        assert_eq!(key_for("not a signature", &[("signing", KEY_A)]), None);
        assert!(signed_by(FILE, SIGNED_BY_A, KEY_A));
        assert!(!signed_by(FILE, SIGNED_BY_A, KEY_B));
        assert!(!signed_by(b"marsa update test filE\n", SIGNED_BY_A, KEY_A));
        assert!(!signed_by(FILE, "", KEY_A));
    }

    /// Tauri's own check and Marsa's must be the same key, or an update
    /// Marsa accepts is refused by Tauri (or the other way round).
    #[test]
    fn tauri_uses_the_signing_key_the_engine_carries() {
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let pubkey = conf["plugins"]["updater"]["pubkey"]
            .as_str()
            .expect("plugins.updater.pubkey");
        assert_eq!(pubkey, include_str!("../update-keys/signing.pub").trim());
        assert_eq!(marsa_core::update::KEYS[0].1.trim(), pubkey);
    }

    /// The two keys are public keys, and different ones: a lost signing key
    /// is replaced by the offline emergency key.
    #[test]
    fn the_built_in_keys_are_public_keys() {
        use marsa_core::update::{KEYS, key_id};
        for (name, key) in KEYS {
            assert!(
                key.trim().is_empty() || key_id(key).is_some(),
                "update-keys/{name}.pub is not a public key"
            );
            assert!(
                !key.contains("secret") && !key.contains("SECRET"),
                "{name}: a private key"
            );
        }
        assert_ne!(KEYS[0].1, KEYS[1].1);
    }

    /// #124: the notice comes with a newer release, and not with none or
    /// a failure; when the setting is off, nothing is asked at all.
    #[tokio::test]
    async fn the_notice_follows_the_check_and_the_setting() {
        assert_eq!(
            found(true, || async { Ok(Some(release())) }).await,
            Some(release())
        );
        assert_eq!(found(true, || async { Ok(None) }).await, None);
        let failed = found(true, || async { Err(CheckError::Answer("odd".into())) }).await;
        assert_eq!(failed, None);
        let asked = std::sync::atomic::AtomicBool::new(false);
        let off = found(false, || async {
            asked.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(Some(release()))
        })
        .await;
        assert_eq!(off, None);
        assert!(
            !asked.load(std::sync::atomic::Ordering::SeqCst),
            "nothing asked"
        );
    }
}
