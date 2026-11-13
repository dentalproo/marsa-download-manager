//! The signed update (ADR-0011, section 1; owner's request of 2026-10-31):
//! the newest release's `latest.json`, for this system, signed by one of the
//! project's two keys, naming a file in the project's own release downloads
//! and for the version it announces. The bytes themselves are checked after
//! they are downloaded.
//!
//! Everything here runs against `marsa-testserver` over HTTPS, with the
//! temporary CA created in memory for this test run (`Engine::with_test_root`,
//! `test-support`). Every refusal has a success twin on the same kind of
//! server (AGENTS.md, checklist 4), so a broken setup cannot pass as
//! "refuses the wrong thing".

use std::fs;

use marsa_core::update::{self, CheckError, KEYS, Offer};
use marsa_core::{Engine, Options, Stop};
use marsa_testserver::{FileSpec, TestServer, Tls, sha256_hex, test_ca_der};
use serde_json::json;

/// The update's file, as signed for 9.9.9 by the project's signing key
/// (`apps/desktop/src-tauri/update-keys`).
const FILE: &[u8] = include_bytes!("fixtures/update/marsa-update.bin");
const SIGNED: &str = include_str!("fixtures/update/marsa-update.bin.minisig");
const EMERGENCY: &str = include_str!("fixtures/update/marsa-update.bin.emergency.minisig");

/// A signature by a key that is not one of the project's two.
const SIGNED_BY_STRANGER: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHJzaWduIHNlY3JldCBrZXkKUlVTU3dDSjJGRlpFanBMKzZnT0ovUk8xTFRsRjd6SlhKUC8wRnNpZEU4cEdHUUhmSm9XZnhncklOczViSS9RODhZUDczZXJUVFh4bUtKK0NJYUovOEdmYmM1MGhNdWRPaFFRPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzk0MDkyODIzCWZpbGU6QzpcVXNlcnNcRUwtSGFtZFxBcHBEYXRhXExvY2FsXFRlbXBcb3BlbmNvZGVcd29ya1xmaWxlLmJpbglwcmVoYXNoZWQKK2FCbnFWNjVITy9WZUw4NndYRDNudk5aVFNJd3FmbFFiSUg2Y0JWTE4vQUs2akxQUGNSYStkd1diK2lhUEZ1U21zZXhlVWt3TnlKbDdIQ1ZpZGgrQ2c9PQo=";

/// The name Tauri's updater uses for a system. `check_from` takes it as an
/// argument, so these tests run the same on all three systems.
const TARGET: &str = "windows-x86_64";

const CURRENT: &str = "0.1.0";

fn trusting_engine() -> Engine {
    Engine::with_test_root(Options::default(), &test_ca_der()).unwrap()
}

fn system_engine() -> Engine {
    Engine::new(Options::default()).unwrap()
}

/// A `latest.json` in Tauri's static shape, with the optional fields it
/// also knows, naming one file for this system.
fn static_manifest(version: &str, url: &str, signature: &str) -> String {
    json!({
        "version": version,
        "notes": "a new Marsa",
        "pub_date": "2026-11-12T00:00:00Z",
        "platforms": { (TARGET): { "url": url, "signature": signature.trim() } },
    })
    .to_string()
}

/// A `latest.json` in the dynamic shape (the server resolved the platform).
fn dynamic_manifest(version: &str, url: &str, signature: &str) -> String {
    json!({
        "version": version,
        "url": url,
        "signature": signature.trim(),
    })
    .to_string()
}

/// A test server holding one release: its `latest.json`, and the file the
/// manifest names.
struct Releases {
    server: TestServer,
    manifest: String,
    downloads: String,
    path: String,
    file: String,
    body: String,
}

fn releases_on(server: TestServer, version: &str, signature: &str) -> Releases {
    let downloads = server.url("/releases/download/");
    let path = format!("/releases/download/v{version}/marsa-update.bin");
    let file = server.url(&path);
    server.put(&path, FileSpec::new(FILE.to_vec()));
    let body = static_manifest(version, &file, signature);
    server.put("/latest.json", FileSpec::new(body.clone().into_bytes()));
    let manifest = server.url("/latest.json");
    Releases {
        server,
        manifest,
        downloads,
        path,
        file,
        body,
    }
}

fn releases(version: &str, signature: &str) -> Releases {
    releases_on(
        TestServer::https(Tls::trusted(&["localhost"]), "localhost"),
        version,
        signature,
    )
}

impl Releases {
    /// Puts `body` in place of the manifest, keeping the file.
    fn set_manifest(&self, body: String) {
        self.server
            .put("/latest.json", FileSpec::new(body.into_bytes()));
    }
}

async fn check_at(
    engine: &Engine,
    releases: &Releases,
    current: &str,
) -> Result<Option<Offer>, CheckError> {
    update::check_from(
        engine,
        &releases.manifest,
        &releases.downloads,
        current,
        TARGET,
        &Stop::new(),
    )
    .await
}

async fn check(engine: &Engine, releases: &Releases) -> Result<Option<Offer>, CheckError> {
    check_at(engine, releases, CURRENT).await
}

async fn offered(engine: &Engine, releases: &Releases) -> Offer {
    check(engine, releases)
        .await
        .unwrap()
        .expect("the update is offered")
}

async fn download(engine: &Engine, url: &str) -> Vec<u8> {
    let dir = tempfile::tempdir().unwrap();
    let done = engine.download(url, dir.path(), &mut |_| {}).await.unwrap();
    fs::read(done.file).unwrap()
}

#[tokio::test]
async fn a_signed_newer_update_is_offered_and_its_bytes_verify() {
    let releases = releases("9.9.9", SIGNED);
    let engine = trusting_engine();

    let offer = offered(&engine, &releases).await;

    assert_eq!(offer.version, "9.9.9");
    assert_eq!(offer.url, releases.file);
    assert_eq!(offer.key(), Some(KEYS[0]));
    // The manifest is kept exactly as it came: Tauri's updater installs
    // from it (`Updater::restore_update`).
    assert_eq!(offer.raw, releases.body);

    let bytes = download(&engine, &offer.url).await;
    assert_eq!(sha256_hex(&bytes), sha256_hex(FILE));
    assert!(offer.verify(&bytes));
    // The same offer does not bless other bytes.
    assert!(!offer.verify(b"marsa update 9.9.8\n"));
}

#[tokio::test]
async fn the_offline_emergency_key_signs_an_update_too() {
    let releases = releases("9.9.9", EMERGENCY);
    let engine = trusting_engine();

    let offer = offered(&engine, &releases).await;

    assert_eq!(offer.key(), Some(KEYS[1]));
    assert!(offer.verify(&download(&engine, &offer.url).await));
}

#[tokio::test]
async fn a_manifest_is_refused_when_its_certificate_is_not_trusted() {
    // The very server and manifest the first test accepts, asked by an
    // engine that trusts only the system store: the certificate is the only
    // difference.
    let releases = releases("9.9.9", SIGNED);

    let err = check(&system_engine(), &releases).await.unwrap_err();

    assert!(matches!(err, CheckError::Engine(_)), "{err}");
    assert!(
        err.to_string().to_lowercase().contains("certificate"),
        "{err}"
    );
    assert!(
        releases.server.gets("/latest.json").is_empty(),
        "the request went through"
    );
}

#[tokio::test]
async fn a_self_signed_certificate_is_refused_even_by_a_trusting_engine() {
    let releases = releases_on(
        TestServer::https(Tls::self_signed(&["localhost"]), "localhost"),
        "9.9.9",
        SIGNED,
    );

    let err = check(&trusting_engine(), &releases).await.unwrap_err();

    assert!(matches!(err, CheckError::Engine(_)), "{err}");
    assert!(releases.server.requests().is_empty());
}

#[tokio::test]
async fn an_update_signed_by_a_key_of_nobody_s_is_refused() {
    // A real signature, for a real file, by a key that is not one of the
    // project's two: no update is offered.
    let releases = releases("9.9.9", SIGNED_BY_STRANGER);

    let err = check(&trusting_engine(), &releases).await.unwrap_err();

    assert!(matches!(err, CheckError::NotOurs), "{err}");
}

#[tokio::test]
async fn a_signature_for_another_version_is_refused() {
    // The signature says 9.9.9; the manifest says 9.9.8. An old signed file
    // cannot be passed off as a new release.
    let releases = releases("9.9.8", SIGNED);

    let err = check(&trusting_engine(), &releases).await.unwrap_err();

    assert!(
        matches!(err, CheckError::NotSignedFor(ref v) if v == "9.9.8"),
        "{err}"
    );
}

#[tokio::test]
async fn a_manifest_that_names_a_file_outside_the_release_is_refused() {
    let releases = releases("9.9.9", SIGNED);
    let engine = trusting_engine();
    let elsewhere = [
        "https://evil.example/v9.9.9/marsa-update.bin".to_owned(),
        format!("{}v9.9.9/../../marsa-update.bin", releases.downloads),
        format!("{}v9.9.9/..%2f..%2fmarsa-update.bin", releases.downloads),
        format!("{}v9.9.9/marsa update.bin", releases.downloads),
        format!("{}v9.9.9/marsa\u{202e}update.bin", releases.downloads),
        format!("{}v9.9.9/", releases.downloads),
        format!("{}v9.9.8/marsa-update.bin", releases.downloads),
    ];
    for url in elsewhere {
        releases.set_manifest(static_manifest("9.9.9", &url, SIGNED));
        let err = check(&engine, &releases)
            .await
            .expect_err(&format!("{url} must be refused"));
        assert!(matches!(err, CheckError::Foreign(_)), "{url}: {err}");
    }
}

#[tokio::test]
async fn nothing_is_offered_when_the_announced_version_is_not_newer() {
    let releases = releases("9.9.9", SIGNED);
    let engine = trusting_engine();

    for current in ["9.9.9", "10.0.0", "9.9.10"] {
        assert!(
            check_at(&engine, &releases, current)
                .await
                .unwrap()
                .is_none(),
            "{current}"
        );
    }
    // The same manifest is offered to an older Marsa.
    assert!(
        check_at(&engine, &releases, "9.9.8")
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn a_manifest_without_this_system_is_not_offered() {
    let releases = releases("9.9.9", SIGNED);
    releases.set_manifest(
        json!({
            "version": "9.9.9",
            "platforms": { "darwin-aarch64": {
                "url": releases.file,
                "signature": SIGNED.trim(),
            } },
        })
        .to_string(),
    );

    let err = check(&trusting_engine(), &releases).await.unwrap_err();

    assert!(
        matches!(err, CheckError::NoFile(ref t) if t == TARGET),
        "{err}"
    );
}

#[tokio::test]
async fn a_manifest_with_a_field_nobody_knows_is_refused() {
    // A strict manifest: a field Marsa does not know may mean the shape
    // changed, and guessing is how a check is skipped.
    let releases = releases("9.9.9", SIGNED);
    releases.set_manifest(
        json!({
            "version": "9.9.9",
            "evil": true,
            "platforms": { (TARGET): {
                "url": releases.file,
                "signature": SIGNED.trim(),
            } },
        })
        .to_string(),
    );

    let err = check(&trusting_engine(), &releases).await.unwrap_err();

    assert!(matches!(err, CheckError::Answer(_)), "{err}");
}

#[tokio::test]
async fn the_dynamic_shape_tauri_also_knows_is_accepted() {
    // A server that resolved the platform itself answers with `url` and
    // `signature` at the top; Tauri accepts that shape, so Marsa must too.
    let releases = releases("9.9.9", SIGNED);
    releases.set_manifest(dynamic_manifest("9.9.9", &releases.file, SIGNED));
    let engine = trusting_engine();

    let offer = offered(&engine, &releases).await;

    assert_eq!(offer.url, releases.file);
    assert!(offer.verify(&download(&engine, &offer.url).await));
}

#[tokio::test]
async fn a_release_file_changed_after_it_was_signed_is_refused() {
    let releases = releases("9.9.9", SIGNED);
    let engine = trusting_engine();
    let offer = offered(&engine, &releases).await;

    // The server now serves other bytes at the same address, after the
    // manifest (and its signature) was made.
    releases.server.put(
        &releases.path,
        FileSpec::new(b"marsa update 9.9.9\n(evil)\n".to_vec()),
    );
    let bytes = download(&engine, &offer.url).await;

    assert_ne!(bytes, FILE);
    assert!(!offer.verify(&bytes));
}
