//! The measurement ROADMAP phase 2 asks for («قياس الأداء: 8 و32 اتصالًا،
//! وحد السرعة، والذاكرة»), on this machine, against the local test server
//! only. Every test here is `#[ignore]`d: it is slow on purpose (a few
//! seconds each, and a couple of hundred MiB of disk) and its numbers belong
//! to the machine that ran it, so it is not part of the CI run. What the CI
//! checks is `parallel.rs` and `speed_limit*.rs`; this one prints the table
//! that `docs/PERFORMANCE.md` records. Run it with
//!
//! ```text
//! cargo test -p marsa-core --release --test performance -- --ignored --nocapture
//! ```

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use marsa_core::{Engine, Event, Options};
use marsa_platform::memory;
use marsa_testserver::{FileSpec, TestServer, random_bytes, sha256_hex};

const MIB: u64 = 1024 * 1024;

fn engine(options: Options) -> Engine {
    Engine::new(options).unwrap()
}

fn mib(bytes: u64) -> f64 {
    bytes as f64 / MIB as f64
}

/// Bytes per second, as MiB per second.
fn rate(bytes: u64, took: Duration) -> f64 {
    mib(bytes) / took.as_secs_f64()
}

/// Downloads `path` and returns (time to the last byte, time to the end).
/// The file itself is read by the callers that check it, never here: the
/// memory test must not read a whole file into this process.
async fn timed(
    engine: Engine,
    server: &TestServer,
    path: &str,
    dir: &Path,
    total: u64,
) -> (Duration, Duration) {
    let started = Instant::now();
    let mut last_byte = None;
    engine
        .download(&server.url(path), dir, &mut |event| {
            if let Event::Progress { done, .. } = event
                && done == total
            {
                last_byte.get_or_insert_with(|| started.elapsed());
            }
        })
        .await
        .unwrap();
    let finished = started.elapsed();
    let every_byte = last_byte.expect("the progress never reached the whole file");
    (every_byte, finished)
}

/// 8 and 32 connections against a server that gives each connection the same
/// speed: what more connections buy, and what the file costs beyond that.
#[tokio::test]
#[ignore = "measurement, run by hand: see docs/PERFORMANCE.md"]
async fn eight_and_thirty_two_connections() {
    const SIZE: u64 = 64 * MIB;
    const PER_CONNECTION: u64 = 2 * MIB; // bytes per second
    let server = TestServer::http();
    let data = random_bytes(SIZE as usize, 900);
    let expected = sha256_hex(&data);
    let mut rows = Vec::new();
    for connections in [8usize, 32] {
        let path = format!("/perf-{connections}.bin");
        server.put(&path, FileSpec::new(data.clone()).throttle(PER_CONNECTION));
        let dir = tempfile::tempdir().unwrap();
        let (transfer, finished) = timed(
            engine(Options {
                connections,
                ..Options::default()
            }),
            &server,
            &path,
            dir.path(),
            SIZE,
        )
        .await;
        let read = fs::read(dir.path().join(format!("perf-{connections}.bin"))).unwrap();
        assert_eq!(sha256_hex(&read), expected, "{connections} connections");
        assert_eq!(server.most_concurrent(&path), connections);
        eprintln!(
            "connections {connections:>2}: {:>6.2} s to the last byte ({:>5.1} MiB/s), \
             {:>6.2} s to the end ({:>5.1} MiB/s)",
            transfer.as_secs_f64(),
            rate(SIZE, transfer),
            finished.as_secs_f64(),
            rate(SIZE, finished),
        );
        rows.push((connections, transfer, finished));
    }
    let (eight, thirty_two) = (rows[0], rows[1]);
    // The server gives each connection 2 MiB/s, so 32 connections can be
    // four times as fast as 8. The disk and the hashing take part of that,
    // so the claim is only that more connections are faster, and the table
    // says by how much.
    assert!(
        thirty_two.1 < eight.1,
        "32 connections took {:?}, 8 took {:?}",
        thirty_two.1,
        eight.1
    );
}

/// The limit on the real clock: a download limited to 2 MiB/s averages
/// about that, as the owner's decision asks (0.9 to 1.1 × the limit).
#[tokio::test]
#[ignore = "measurement, run by hand: see docs/PERFORMANCE.md"]
async fn the_speed_limit_holds_on_the_real_clock() {
    const SIZE: u64 = 16 * MIB;
    const LIMIT: u64 = 2 * MIB;
    let server = TestServer::http();
    let data = random_bytes(SIZE as usize, 901);
    server.put("/limited.bin", FileSpec::new(data.clone()));
    let dir = tempfile::tempdir().unwrap();
    let (transfer, finished) = timed(
        engine(Options {
            connections: 8,
            download_limit: Some(LIMIT),
            ..Options::default()
        }),
        &server,
        "/limited.bin",
        dir.path(),
        SIZE,
    )
    .await;
    assert_eq!(fs::read(dir.path().join("limited.bin")).unwrap(), data);
    let measured = rate(SIZE, transfer);
    let limit = mib(LIMIT);
    eprintln!(
        "limit {limit:.0} MiB/s: {:>6.2} s to the last byte ({measured:.2} MiB/s, \
         {:.2} × the limit), {:>6.2} s to the end",
        transfer.as_secs_f64(),
        measured / limit,
        finished.as_secs_f64(),
    );
    assert!(
        transfer >= Duration::from_secs(6),
        "measured over {transfer:?} only"
    );
    let ratio = measured / limit;
    assert!(
        (0.9..=1.1).contains(&ratio),
        "{ratio:.3} × the limit over {transfer:?}"
    );
}

/// The memory a download of 256 MiB adds: the engine streams, so the file is
/// never held. The test server keeps its own copy in this same process, so
/// the file is put before the baseline is read: what is measured is the
/// download, not the fixture.
#[tokio::test]
#[ignore = "measurement, run by hand: see docs/PERFORMANCE.md"]
async fn the_memory_does_not_follow_the_file() {
    const SIZE: u64 = 256 * MIB;
    let server = TestServer::http();
    let data = random_bytes(SIZE as usize, 902);
    server.put("/huge.bin", FileSpec::new(data));
    let dir = tempfile::tempdir().unwrap();

    let before = memory::peak_rss_bytes().expect("no peak resident set on this system");
    let now = memory::rss_bytes().expect("no resident set on this system");
    let (transfer, finished) = timed(
        engine(Options {
            connections: 8,
            ..Options::default()
        }),
        &server,
        "/huge.bin",
        dir.path(),
        SIZE,
    )
    .await;
    let after = memory::peak_rss_bytes().expect("no peak resident set on this system");
    let held = memory::rss_bytes().expect("no resident set on this system");

    eprintln!(
        "{:.0} MiB in {:>6.2} s ({:.1} MiB/s), {:>6.2} s to the end",
        mib(SIZE),
        transfer.as_secs_f64(),
        rate(SIZE, transfer),
        finished.as_secs_f64()
    );
    eprintln!(
        "resident: {:.1} MiB before, {:.1} MiB after; peak {:.1} MiB before, \
         {:.1} MiB after (+{:.1} MiB)",
        mib(now),
        mib(held),
        mib(before),
        mib(after),
        mib(after.saturating_sub(before))
    );
    // Eight connections with their own buffers, plus the file's last blocks
    // and the hashing: a few MiB. A quarter of the file would mean it was
    // held somewhere.
    assert!(
        after.saturating_sub(before) < SIZE / 4,
        "the peak grew by {:.1} MiB for a {:.0} MiB file",
        mib(after.saturating_sub(before)),
        mib(SIZE)
    );
}
