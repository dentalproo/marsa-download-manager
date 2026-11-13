//! The Native Messaging host behind this package's build (ADR-0005): the
//! browser starts `marsa-host` beside `marsa-desktop.exe` — the program
//! this binary stands beside, running the same library `crates/marsa-host`
//! builds.

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Ok(data_dir) = marsa_host::data_dir() else {
        return ExitCode::FAILURE;
    };
    let (mut input, mut output) = (io::stdin().lock(), io::stdout().lock());
    marsa_host::answer_connection(&data_dir, &mut input, &mut output)
}
