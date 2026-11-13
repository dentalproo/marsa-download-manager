#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use winreg::RegKey;
use winreg::enums::*;

const APP_NAME: &str = "Marsa";
const EXE_NAME: &str = "marsa-desktop.exe";
// The real app, embedded into the installer binary as a single file. The
// build script places the release exe at `src-tauri/resources/marsa-desktop.exe`
// before building the installer, so `include_bytes!` always finds it.
const APP_EXE: &[u8] = include_bytes!("../resources/marsa-desktop.exe");
// The browser integration's host (ADR-0005): the browsers start it beside
// the app, and it finds the app through the private channel. The build
// script stages it next to the app exe before building this installer, so
// `include_bytes!` always finds it.
const HOST_EXE: &[u8] = include_bytes!("../resources/marsa-host.exe");

fn default_install_dir() -> PathBuf {
    // %LOCALAPPDATA%\Programs\Marsa (per-user program files), not Roaming
    // (Roaming is for roaming settings). `dirs::data_local_dir` is Local.
    if let Some(local) = dirs::data_local_dir() {
        local.join("Programs").join(APP_NAME)
    } else {
        PathBuf::from(r"C:\Users")
            .join("user")
            .join("AppData")
            .join("Local")
            .join("Programs")
            .join(APP_NAME)
    }
}

fn staged_exe() -> Option<PathBuf> {
    // In a Tauri bundle the main app ships as a resource next to the installer.
    // Look in the usual candidate places so `cargo run` in dev works too.
    let mut candidates = Vec::new();
    if let Some(rd) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    {
        candidates.push(rd.join("resources").join(EXE_NAME));
        candidates.push(rd.join(EXE_NAME));
        candidates.push(rd);
    }
    // dev fallback: the debug build of the main app
    candidates.push(PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../desktop/src-tauri/target/debug/marsa-desktop.exe"
    )));
    candidates.push(PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../desktop/src-tauri/target/release/marsa-desktop.exe"
    )));
    candidates.into_iter().find(|p| p.exists())
}

fn copy_file(src: &Path, dst: &Path) -> std::io::Result<()> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(src, dst)?;
    Ok(())
}

/// Writes the app over the installed one. On an update, Tauri's updater
/// starts this installer and exits Marsa at once, so Windows may still hold
/// `marsa-desktop.exe` for a moment (and a virus scanner or the indexer may
/// hold it too). The write is retried for ten seconds before it is called a
/// failure.
fn write_app(dest: &Path) -> std::io::Result<()> {
    write_bytes(dest, APP_EXE)
}

/// Writes the host the browsers start (ADR-0005), as the app bytes, with the
/// same retry a write that an updating app holds can need.
fn write_host(dest: &Path) -> std::io::Result<()> {
    write_bytes(dest, HOST_EXE)
}

fn write_bytes(dest: &Path, bytes: &[u8]) -> std::io::Result<()> {
    const ATTEMPTS: u32 = 40;
    let mut last = None;
    for attempt in 0..ATTEMPTS {
        match std::fs::write(dest, bytes) {
            Ok(()) => return Ok(()),
            Err(e) => last = Some(e),
        }
        if attempt + 1 < ATTEMPTS {
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
    Err(last.expect("at least one attempt"))
}

fn make_shortcut(target_exe: &Path, lnk: &Path) -> Result<(), String> {
    if let Some(dir) = lnk.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    if lnk.exists() {
        let _ = std::fs::remove_file(lnk);
    }
    // Create the .lnk via WScript.Shell COM through PowerShell (built into
    // Windows, no extra dependency). The installer is Windows-first.
    let lnk_s = lnk.to_string_lossy().to_string();
    let exe_s = target_exe.to_string_lossy().to_string();
    let ps = format!(
        "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('{lnk}'); $s.TargetPath = '{exe}'; $s.Save()",
        lnk = lnk_s.replace("'", "''"),
        exe = exe_s.replace("'", "''"),
    );
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .status()
        .map_err(|e| format!("powershell: {e}"))?;
    Ok(())
}

fn write_uninstall_entry(install_dir: &Path) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(format!(
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{APP_NAME}"
        ))
        .map_err(|e| format!("registry: {e}"))?;
    let exe = install_dir.join(EXE_NAME);
    let uninstaller = install_dir.join("uninstall.exe");
    let _ = key.set_value("DisplayName", &format!("Marsa {APP_NAME} Setup"));
    let _ = key.set_value(
        "UninstallString",
        &format!("\"{}\"", uninstaller.to_string_lossy()),
    );
    let _ = key.set_value(
        "InstallLocation",
        &install_dir.to_string_lossy().to_string(),
    );
    let _ = key.set_value("DisplayVersion", &env!("CARGO_PKG_VERSION"));
    let _ = exe.as_os_str();
    Ok(())
}

fn do_install(path: &Path, desktop: bool, start_menu: bool) -> Result<String, String> {
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let dest = path.join(EXE_NAME);
    // Write the embedded app bytes (single-file installer), waiting for the
    // old app to let go of its own file after an update.
    write_app(&dest).map_err(|e| format!("write app: {e}"))?;
    // The same for the browser integration's host (ADR-0005), so that the
    // browsers that ask for it have something to start.
    write_host(&path.join("marsa-host.exe")).map_err(|e| format!("write host: {e}"))?;
    // Copy a second marker so updates and sweeps see a consistent folder
    let readme = path.join("README.txt");
    let _ = std::fs::write(
        &readme,
        format!(
            "Marsa {}\nInstalled from {}\n",
            env!("CARGO_PKG_VERSION"),
            path.display()
        ),
    );
    let _ = copy_file(&path.join(EXE_NAME), &path.join("marsa-desktop.exe.bak")).ok();
    if desktop {
        if let Some(desk) = dirs::desktop_dir() {
            let _ = make_shortcut(&dest, &desk.join("Marsa.lnk"));
        } else if let Some(home) = dirs::home_dir() {
            let _ = make_shortcut(&dest, &home.join("Desktop").join("Marsa.lnk"));
        }
    }
    if start_menu
        && let Some(programs) = dirs::data_dir()
    {
        let start = programs
            .join(r"Microsoft\Windows\Start Menu\Programs")
            .join("Marsa");
        let _ = make_shortcut(&dest, &start.join("Marsa.lnk"));
    }
    write_uninstall_entry(path)?;
    Ok(dest.to_string_lossy().to_string())
}

#[derive(Serialize)]
struct InstallResult {
    ok: bool,
    message: String,
    installed_path: String,
}

#[tauri::command]
fn get_default_path() -> String {
    default_install_dir().to_string_lossy().to_string()
}

#[tauri::command]
fn start_install(
    path: String,
    desktop_shortcut: bool,
    start_menu: bool,
    run_after: bool,
) -> InstallResult {
    let dir = PathBuf::from(&path);
    match do_install(&dir, desktop_shortcut, start_menu) {
        Ok(installed) => {
            if run_after {
                let _ = std::process::Command::new(&installed).spawn();
            }
            InstallResult {
                ok: true,
                message: "installed".into(),
                installed_path: installed,
            }
        }
        Err(e) => InstallResult {
            ok: false,
            message: e,
            installed_path: path,
        },
    }
}

#[tauri::command]
fn launch_app() -> Result<(), String> {
    let exe = default_install_dir().join(EXE_NAME);
    if exe.exists() {
        std::process::Command::new(&exe)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    } else if let Some(src) = staged_exe() {
        std::process::Command::new(src)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    } else {
        Err("app not found".into())
    }
}

fn is_silent(args: &[String]) -> bool {
    args.iter().any(|a| {
        let l = a.to_ascii_lowercase();
        l == "--silent"
            || l == "/s"
            || l == "-s"
            || l == "/p"
            || l == "--update"
            || l == "/update"
            || l.starts_with("--silent=")
    })
}

/// The arguments after Tauri's `/ARGS`: the updater passes the running
/// Marsa's own arguments on, so the app that opens again is the same one
/// the user had.
fn app_args(args: &[String]) -> Vec<String> {
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let l = arg.to_ascii_lowercase();
        if l == "/args" || l == "--args" {
            return it.cloned().collect();
        }
    }
    Vec::new()
}

/// Whether Marsa must open again after the install. Tauri's updater says so
/// with `/R`, and adds `/ARGS` only when it wants the app back; `--launch`
/// and `/launch` are the plain installer's own switch.
fn relaunch(args: &[String]) -> bool {
    args.iter().any(|a| {
        let l = a.to_ascii_lowercase();
        l == "/r" || l == "/launch" || l == "--launch" || l == "/args" || l == "--args"
    })
}

fn main() {
    // Silent / update install: no UI at all, same as NSIS /S.
    let args: Vec<String> = std::env::args().collect();
    if is_silent(&args) {
        let dir = default_install_dir();
        let run_after = relaunch(&args);
        match do_install(&dir, true, true) {
            Ok(installed) => {
                if run_after {
                    let _ = std::process::Command::new(&installed)
                        .args(app_args(&args))
                        .spawn();
                }
                std::process::exit(0);
            }
            Err(_) => std::process::exit(1),
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_default_path,
            start_install,
            launch_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running Marsa installer");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    /// What Tauri's updater passes (`passive` install mode, restart after
    /// install): `/P /UPDATE /R /ARGS <Marsa's own arguments>`.
    #[test]
    fn an_update_is_silent_and_opens_marsa_again() {
        let update = args(&[
            "marsa-installer.exe",
            "/P",
            "/UPDATE",
            "/R",
            "/ARGS",
            "--open",
            "a b",
        ]);
        assert!(is_silent(&update));
        assert!(relaunch(&update));
        assert_eq!(app_args(&update), args(&["--open", "a b"]));
    }

    /// A plain double-click shows the window and leaves Marsa closed.
    #[test]
    fn a_plain_install_shows_the_window() {
        let plain = args(&["marsa-installer.exe"]);
        assert!(!is_silent(&plain));
        assert!(!relaunch(&plain));
        assert!(app_args(&plain).is_empty());
        // The quiet switches: Tauri's and NSIS's.
        for silent in [
            "/S",
            "/s",
            "-s",
            "--silent",
            "/P",
            "/UPDATE",
            "--update",
            "--silent=1",
        ] {
            assert!(
                is_silent(&args(&["marsa-installer.exe", silent])),
                "{silent}"
            );
        }
        for launch in ["/R", "/r", "--launch", "/launch", "--args"] {
            assert!(
                relaunch(&args(&["marsa-installer.exe", launch])),
                "{launch}"
            );
        }
    }

    /// Only what follows `/ARGS` is passed on; anything else is the
    /// installer's own.
    #[test]
    fn the_arguments_after_args_are_marsa_s_own() {
        assert!(app_args(&args(&["x", "--args"])).is_empty());
        assert!(app_args(&args(&["x", "/update", "--open"])).is_empty());
        assert_eq!(
            app_args(&args(&["x", "/args", "/a", "/b"])),
            args(&["/a", "/b"])
        );
    }
}
