//! `marsa-host`: the browser starts it for Marsa's extension (ADR-0005).
//! It reads the extension's messages from stdin, checks each one
//! (`marsa_host::parse`), hands a download to the running app over the
//! user's private channel, and writes every answer to stdout. It never
//! downloads anything itself.
//!
//! The data folder is the app's (`MARSA_DATA_DIR`, else the user's
//! standard one), so the host finds the app that the user runs. When no
//! app is running, the host starts the one next to it (on macOS through
//! LaunchServices, so that the browser's quarantine mark does not pass to
//! it), unless `MARSA_NO_LAUNCH` is set (tests), and waits for it to open
//! its channel.
//!
//! A `link` message (M5, `runtime.connectNative`) turns the host into the
//! extension's link to the app for the rest of its life ([`link`]): Marsa
//! asks that browser for cookies on it. The link never starts the app.
//!
//! A `ping` is answered by the host itself, with Marsa's language read from
//! its settings (read only; the extension's video button speaks it).
//!
//! Nothing is written to stderr that the browser would need: Chrome keeps
//! it in its log, Firefox in the browser console.

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Ok(data_dir) = marsa_host::data_dir() else {
        return ExitCode::FAILURE;
    };
    let (mut input, mut output) = (io::stdin().lock(), io::stdout().lock());
    marsa_host::answer_connection(&data_dir, &mut input, &mut output)
}
