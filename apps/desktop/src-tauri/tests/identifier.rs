//! ADR-0001: the Tauri `identifier` is `io.github.marsa-app.marsa`,
//! the same constant the rest of Marsa uses.

#[test]
fn the_tauri_identifier_is_the_apple_bundle_id_of_adr_0001() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json"))
        .expect("tauri.conf.json");
    let config: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    assert_eq!(config["identifier"], "io.github.marsa-app.marsa");
    assert_eq!(config["identifier"], marsa_platform::ids::APPLE_BUNDLE_ID);
}
