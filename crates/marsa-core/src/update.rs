//! Whether a newer Marsa is out (#124, owner's decision): one question to
//! GitHub's list of releases, never a download or an install. The answer
//! says the newer version and its release page, built here on Marsa's own
//! releases address (never a link taken from the answer).
//!
//! The list, not `/releases/latest`: that one leaves out pre-releases, and
//! every 0.x release is one. Drafts and tags that are not `vX.Y.Z` are
//! passed over. One request with the engine itself (its TLS, its
//! `Marsa/…` User-Agent), and at most [`MAX_ANSWER`] bytes of answer.

use std::collections::BTreeMap;

use base64::Engine as _;
use marsa_platform::target::{Arch, Os, Target};
use serde::Deserialize;

use crate::{Engine, Error, Stop};

/// GitHub's list of Marsa's releases.
pub const RELEASES: &str =
    "https://api.github.com/repos/dentalproo/marsa-download-manager/releases?per_page=30";

/// Every release page is under this address.
pub const RELEASE_PAGES: &str = "https://github.com/dentalproo/marsa-download-manager/releases/";

/// The most of GitHub's answer that is read.
pub const MAX_ANSWER: usize = 1024 * 1024;

/// A release newer than the running Marsa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRelease {
    /// `X.Y.Z`, without the `v`.
    pub version: String,
    /// Its page, always under [`RELEASE_PAGES`].
    pub page: String,
}

/// Why the question could not be answered.
#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error("could not be fetched: {0}")]
    Engine(#[from] Error),
    #[error("GitHub's answer could not be read: {0}")]
    Answer(String),
    #[error("{0:?} is not a version of the form X.Y.Z")]
    Version(String),
    #[error("the update manifest has no file for this system: {0}")]
    NoFile(String),
    #[error("the update's file is not in Marsa's own releases: {0}")]
    Foreign(String),
    #[error("the update is not signed by a key of Marsa's")]
    NotOurs,
    #[error("the update is not signed for version {0}")]
    NotSignedFor(String),
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    draft: bool,
}

/// The newest release on GitHub if it is newer than `current` (`X.Y.Z`,
/// the running app's version), else `None`.
pub async fn newer_release(
    engine: &Engine,
    current: &str,
    stop: &Stop,
) -> Result<Option<NewRelease>, CheckError> {
    ask(engine, RELEASES, current, stop).await
}

/// **Test builds only.** [`newer_release`], asking `list` (a local test
/// server's) instead of GitHub.
#[cfg(feature = "test-support")]
pub async fn newer_release_from(
    engine: &Engine,
    list: &str,
    current: &str,
    stop: &Stop,
) -> Result<Option<NewRelease>, CheckError> {
    ask(engine, list, current, stop).await
}

async fn ask(
    engine: &Engine,
    list: &str,
    current: &str,
    stop: &Stop,
) -> Result<Option<NewRelease>, CheckError> {
    let current = version(current).ok_or_else(|| CheckError::Version(current.to_owned()))?;
    let answer = engine.fetch_text(list, MAX_ANSWER, stop).await?;
    newest(&answer, current)
}

/// The newest release in GitHub's answer `json` that is newer than
/// `current`.
fn newest(json: &str, current: (u64, u64, u64)) -> Result<Option<NewRelease>, CheckError> {
    let releases: Vec<Release> =
        serde_json::from_str(json).map_err(|e| CheckError::Answer(e.to_string()))?;
    let best = releases
        .iter()
        .filter(|r| !r.draft)
        .filter_map(|r| {
            let number = r.tag_name.strip_prefix('v')?;
            Some((version(number)?, number))
        })
        .filter(|(v, _)| *v > current)
        .max_by_key(|(v, _)| *v);
    Ok(best.map(|(_, number)| NewRelease {
        version: number.to_owned(),
        page: format!("{RELEASE_PAGES}tag/v{number}"),
    }))
}

/// `X.Y.Z` (digits only, each at most 9 of them), or `None`.
fn version(text: &str) -> Option<(u64, u64, u64)> {
    let mut parts = text.split('.');
    let mut part = || {
        let p = parts.next()?;
        (!p.is_empty() && p.len() <= 9 && p.bytes().all(|b| b.is_ascii_digit()))
            .then(|| p.parse().ok())
            .flatten()
    };
    let found = (part()?, part()?, part()?);
    parts.next().is_none().then_some(found)
}

// The signed update (ADR-0011, section 1). The newest release's
// `latest.json` names, for each system, its file and that file's
// minisign signature. Marsa offers it only when the key is one of the
// project's two, the file is in the project's own release downloads, and
// the signature's trusted comment names the announced version; the bytes
// themselves are checked after the download, before anything is installed.

/// The newest release's update manifest (ADR-0011). It lives under the
/// `updater` tag, not `releases/latest`: GitHub's `latest` leaves out
/// pre-releases, and every 0.x release is one. The release workflow replaces
/// this one file at each release; an older manifest it might serve names an
/// older version, which is never offered.
pub const MANIFEST: &str =
    "https://github.com/dentalproo/marsa-download-manager/releases/download/updater/latest.json";

/// The only place an update's file may come from.
pub const DOWNLOADS: &str =
    "https://github.com/dentalproo/marsa-download-manager/releases/download/";

/// The most of the manifest that is read.
pub const MAX_MANIFEST: usize = 64 * 1024;

/// The project's two public keys (ADR-0011, section 3), as `tauri signer
/// generate` writes them: the signing key, then the offline emergency key.
/// Empty: no update is offered with it.
pub const KEYS: [(&str, &str); 2] = [
    (
        "signing",
        include_str!("../../../apps/desktop/src-tauri/update-keys/signing.pub"),
    ),
    (
        "emergency",
        include_str!("../../../apps/desktop/src-tauri/update-keys/emergency.pub"),
    ),
];

/// The minisign text Tauri's signer base64-encodes in a key or a
/// signature (`untrusted comment: …`).
fn minisign_text(tauri_text: &str) -> Option<String> {
    let b64 = base64::engine::general_purpose::STANDARD;
    let bytes = b64.decode(tauri_text.trim()).ok()?;
    String::from_utf8(bytes).ok()
}

/// The 8-byte key id in a minisign public key or signature, as Tauri
/// writes them (base64 of the minisign text): its second line, after the
/// 2-byte algorithm. None when it is not one.
pub fn key_id(tauri_text: &str) -> Option<[u8; 8]> {
    let text = minisign_text(tauri_text)?;
    let line = text.lines().nth(1)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(line.trim())
        .ok()?;
    bytes.get(2..10)?.try_into().ok()
}

/// The key of `keys` that made `signature`: its name and public key.
pub fn key_for(
    signature: &str,
    keys: &[(&'static str, &'static str)],
) -> Option<(&'static str, &'static str)> {
    let id = key_id(signature)?;
    keys.iter()
        .copied()
        .find(|(_, key)| !key.trim().is_empty() && key_id(key) == Some(id))
}

/// Whether `bytes` are what `signature` signed, with `key`; both as Tauri
/// writes them.
pub fn signed_by(bytes: &[u8], signature: &str, key: &str) -> bool {
    let (Some(signature), Some(key)) = (minisign_text(signature), minisign_text(key)) else {
        return false;
    };
    let (Ok(signature), Ok(key)) = (
        minisign_verify::Signature::decode(&signature),
        minisign_verify::PublicKey::decode(&key),
    ) else {
        return false;
    };
    key.verify(bytes, &signature, false).is_ok()
}

/// Whether the signature's trusted comment names `version` (Tauri's
/// `--app-version`), so a manifest cannot pair a new number with an older
/// file.
pub fn signed_for_version(signature: &str, version: &str) -> bool {
    let Some(text) = minisign_text(signature) else {
        return false;
    };
    let Ok(signature) = minisign_verify::Signature::decode(&text) else {
        return false;
    };
    signature
        .trusted_comment()
        .split('\t')
        .any(|part| part.strip_prefix("version:") == Some(version))
}

/// Tauri's name for `target` in `latest.json`.
pub fn target_name(target: Target) -> &'static str {
    match (target.os, target.arch) {
        (Os::Windows, Arch::X86_64) => "windows-x86_64",
        (Os::Windows, Arch::Aarch64) => "windows-aarch64",
        (Os::Macos, Arch::X86_64) => "darwin-x86_64",
        (Os::Macos, Arch::Aarch64) => "darwin-aarch64",
        (Os::Linux, Arch::X86_64) => "linux-x86_64",
        (Os::Linux, Arch::Aarch64) => "linux-aarch64",
    }
}

/// This build's name in `latest.json`, if Marsa ships for its system.
pub fn this_target() -> Option<&'static str> {
    marsa_platform::target::current().map(target_name)
}

/// A signed update the user may install: its version, its file, the
/// signature of that file, and the manifest as it came (for Tauri's
/// updater to install from).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    pub version: String,
    pub url: String,
    pub signature: String,
    pub raw: String,
}

impl Offer {
    /// The key of the project's that signed it, if any.
    pub fn key(&self) -> Option<(&'static str, &'static str)> {
        key_for(&self.signature, &KEYS)
    }

    /// Whether `bytes` are the file this offer names, signed for the
    /// version it announces.
    pub fn verify(&self, bytes: &[u8]) -> bool {
        self.verify_with(bytes, &KEYS)
    }

    /// [`verify`](Self::verify) with `keys`, for tests that sign with a
    /// key of their own.
    pub fn verify_with(&self, bytes: &[u8], keys: &[(&'static str, &'static str)]) -> bool {
        signed_for_version(&self.signature, &self.version)
            && key_for(&self.signature, keys)
                .is_some_and(|(_, key)| signed_by(bytes, &self.signature, key))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    version: String,
    // Tauri's manifest carries these; Marsa shows neither, but a manifest
    // that has them must still be accepted.
    #[allow(dead_code)]
    #[serde(default)]
    notes: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    pub_date: Option<String>,
    #[serde(default)]
    platforms: Option<BTreeMap<String, RawPlatform>>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    signature: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPlatform {
    url: String,
    signature: String,
}

/// The signed update of the newest release for this system, if it is
/// newer than `current` (`X.Y.Z`) and passes every check. `None` when
/// there is nothing newer or this system has no file.
pub async fn check(
    engine: &Engine,
    current: &str,
    stop: &Stop,
) -> Result<Option<Offer>, CheckError> {
    let Some(target) = this_target() else {
        return Ok(None);
    };
    check_in(engine, MANIFEST, DOWNLOADS, current, target, &KEYS, stop).await
}

/// **Test builds only.** [`check`], asking `manifest_url` (a local test
/// server's) and accepting files under `downloads_prefix`.
#[cfg(feature = "test-support")]
pub async fn check_from(
    engine: &Engine,
    manifest_url: &str,
    downloads_prefix: &str,
    current: &str,
    target: &str,
    stop: &Stop,
) -> Result<Option<Offer>, CheckError> {
    check_in(
        engine,
        manifest_url,
        downloads_prefix,
        current,
        target,
        &KEYS,
        stop,
    )
    .await
}

async fn check_in(
    engine: &Engine,
    manifest_url: &str,
    downloads_prefix: &str,
    current: &str,
    target: &str,
    keys: &[(&'static str, &'static str)],
    stop: &Stop,
) -> Result<Option<Offer>, CheckError> {
    let current = version(current).ok_or_else(|| CheckError::Version(current.to_owned()))?;
    let raw = engine.fetch_text(manifest_url, MAX_MANIFEST, stop).await?;
    let manifest: RawManifest =
        serde_json::from_str(&raw).map_err(|e| CheckError::Answer(e.to_string()))?;
    let announced =
        version(&manifest.version).ok_or_else(|| CheckError::Version(manifest.version.clone()))?;
    if announced <= current {
        return Ok(None);
    }
    let (url, signature) = match manifest.platforms {
        Some(mut platforms) => {
            let platform = platforms
                .remove(target)
                .ok_or_else(|| CheckError::NoFile(target.to_owned()))?;
            (platform.url, platform.signature)
        }
        None => {
            let url = manifest
                .url
                .ok_or_else(|| CheckError::Answer("the manifest names no file".into()))?;
            let signature = manifest
                .signature
                .ok_or_else(|| CheckError::Answer("the manifest names no signature".into()))?;
            (url, signature)
        }
    };
    // The file is the release's own, under its version's folder: no other
    // host, no path that walks back out, nothing a URL could hide an
    // escape in (`%2e%2e`, a right-to-left mark, a query or a fragment).
    let expected = format!("{downloads_prefix}v{}/", manifest.version);
    if !url.starts_with(&expected)
        || url.len() <= expected.len()
        || !url.is_ascii()
        || url
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || matches!(c, '\\' | '?' | '#' | '%'))
        || url.split('/').any(|part| part == "..")
    {
        return Err(CheckError::Foreign(url));
    }
    if key_for(&signature, keys).is_none() {
        return Err(CheckError::NotOurs);
    }
    if !signed_for_version(&signature, &manifest.version) {
        return Err(CheckError::NotSignedFor(manifest.version));
    }
    Ok(Some(Offer {
        version: manifest.version,
        url,
        signature,
        raw,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, draft: bool) -> String {
        format!(r#"{{"tag_name":"{tag}","draft":{draft},"html_url":"https://evil.example/x"}}"#)
    }

    #[test]
    fn the_newest_release_newer_than_this_one_is_found() {
        let json = format!(
            "[{},{},{},{},{}]",
            release("v0.1.0", false),
            release("v0.2.0", false),
            release("v0.10.1", false),
            release("v0.11.0", true),
            release("nightly", false)
        );
        let found = newest(&json, (0, 1, 0)).unwrap().unwrap();
        // 0.10.1 is newer than 0.2.0; a draft and a tag of another form
        // do not count; the page is built here, never the answer's.
        assert_eq!(found.version, "0.10.1");
        assert_eq!(
            found.page,
            "https://github.com/dentalproo/marsa-download-manager/releases/tag/v0.10.1"
        );
        assert_eq!(newest(&json, (0, 10, 1)).unwrap(), None);
        assert_eq!(newest("[]", (0, 1, 0)).unwrap(), None);
    }

    #[test]
    fn only_plain_versions_count() {
        assert_eq!(version("1.2.3"), Some((1, 2, 3)));
        for bad in [
            "1.2",
            "1.2.3.4",
            "1.2.3-beta",
            "v1.2.3",
            "1..3",
            "1.2.x",
            "1234567890.0.0",
            "",
        ] {
            assert_eq!(version(bad), None, "{bad}");
        }
        let json = format!("[{}]", release("v1.2.3-rc.1", false));
        assert_eq!(newest(&json, (0, 1, 0)).unwrap(), None);
    }

    #[test]
    fn an_answer_that_is_not_a_list_is_an_error() {
        assert!(matches!(
            newest("{}", (0, 1, 0)),
            Err(CheckError::Answer(_))
        ));
        assert!(matches!(
            newest("not json", (0, 1, 0)),
            Err(CheckError::Answer(_))
        ));
    }

    // The signed update's helpers, against the fixtures signed by the two
    // keys the app carries (`tests/fixtures/update`). The `.minisig` files
    // hold exactly what Tauri's signer prints (base64 of the minisign text),
    // which is what a manifest's `signature` holds.

    const UPDATE: &[u8] = include_bytes!("../tests/fixtures/update/marsa-update.bin");
    const SIGNED: &str = include_str!("../tests/fixtures/update/marsa-update.bin.minisig");
    const EMERGENCY: &str =
        include_str!("../tests/fixtures/update/marsa-update.bin.emergency.minisig");

    #[test]
    fn each_system_has_tauri_s_name() {
        let windows = |arch| {
            target_name(Target {
                os: Os::Windows,
                arch,
            })
        };
        assert_eq!(windows(Arch::X86_64), "windows-x86_64");
        assert_eq!(windows(Arch::Aarch64), "windows-aarch64");
        let mac = |arch| {
            target_name(Target {
                os: Os::Macos,
                arch,
            })
        };
        assert_eq!(mac(Arch::X86_64), "darwin-x86_64");
        assert_eq!(mac(Arch::Aarch64), "darwin-aarch64");
        let linux = |arch| {
            target_name(Target {
                os: Os::Linux,
                arch,
            })
        };
        assert_eq!(linux(Arch::X86_64), "linux-x86_64");
        assert_eq!(linux(Arch::Aarch64), "linux-aarch64");
        assert_eq!(
            this_target(),
            marsa_platform::target::current().map(target_name)
        );
    }

    #[test]
    fn the_two_keys_are_known_and_differ() {
        let signing = key_id(KEYS[0].1).expect("the signing key's id");
        let emergency = key_id(KEYS[1].1).expect("the emergency key's id");
        assert_ne!(signing, emergency);
        assert_eq!(key_id("not base64"), None);
        assert_eq!(key_id(""), None);
    }

    #[test]
    fn a_signature_names_the_key_that_made_it() {
        assert_eq!(
            key_for(SIGNED.trim(), &KEYS).map(|(name, _)| name),
            Some("signing")
        );
        assert_eq!(
            key_for(EMERGENCY.trim(), &KEYS).map(|(name, _)| name),
            Some("emergency")
        );
        assert_eq!(key_for("not base64", &KEYS), None);
        assert_eq!(key_for("", &KEYS), None);
        // An empty key (never shipped) matches nothing.
        assert_eq!(key_for(SIGNED.trim(), &[("none", "")]), None);
    }

    #[test]
    fn only_the_file_that_was_signed_verifies() {
        assert!(signed_by(UPDATE, SIGNED.trim(), KEYS[0].1));
        assert!(!signed_by(
            b"marsa update 9.9.8\n",
            SIGNED.trim(),
            KEYS[0].1
        ));
        // The other key cannot check it, and a mangled signature is refused.
        assert!(!signed_by(UPDATE, SIGNED.trim(), KEYS[1].1));
        assert!(!signed_by(UPDATE, "not base64", KEYS[0].1));
        assert!(!signed_by(UPDATE, "", KEYS[0].1));
    }

    #[test]
    fn a_signature_says_which_version_it_is_for() {
        assert!(signed_for_version(SIGNED.trim(), "9.9.9"));
        assert!(signed_for_version(EMERGENCY.trim(), "9.9.9"));
        assert!(!signed_for_version(SIGNED.trim(), "9.9.8"));
        assert!(!signed_for_version(SIGNED.trim(), ""));
        assert!(!signed_for_version("not base64", "9.9.9"));
    }

    #[test]
    fn an_offer_verifies_the_bytes_and_the_version_together() {
        let offer = |version: &str, signature: &str| Offer {
            version: version.to_owned(),
            url: format!("{DOWNLOADS}v{version}/marsa.bin"),
            signature: signature.trim().to_owned(),
            raw: String::new(),
        };
        let good = offer("9.9.9", SIGNED);
        assert_eq!(good.key().map(|(name, _)| name), Some("signing"));
        assert!(good.verify(UPDATE));
        assert!(!good.verify(b"something else"));
        // The number and the signature must agree.
        assert!(!offer("9.9.8", SIGNED).verify(UPDATE));
        // The emergency key signs as well as the signing key.
        let emergency = offer("9.9.9", EMERGENCY);
        assert_eq!(emergency.key().map(|(name, _)| name), Some("emergency"));
        assert!(emergency.verify(UPDATE));
        // A key of nobody's is no offer at all.
        let stranger = offer("9.9.9", "not base64");
        assert_eq!(stranger.key(), None);
        assert!(!stranger.verify(UPDATE));
    }
}
