//! Linux-Gegenstuecke zu den macOS-Funktionen in `lib.rs`.
//!
//! - Ruhezustand verhindern: logind-Inhibitor (Systemschlaf) und
//!   `org.freedesktop.ScreenSaver.Inhibit` (Bildschirmschoner/Abdunkeln).
//! - Bildschirm wecken: `SimulateUserActivity` und DPMS.
//! - Wake-Helper: systemd-Path-Unit, die bei jeder Aenderung von
//!   `schedule.json` und vor jedem Suspend den RTC-Wecker per `rtcwake` setzt.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use zbus::blocking::Connection;
use zbus::zvariant::OwnedFd;

pub const SHARED_DIR: &str = "/var/lib/weckpilot";
pub const SCHEDULE_FILE: &str = "/var/lib/weckpilot/schedule.json";
const HELPER_SCRIPT: &str = "/usr/local/lib/weckpilot/weckpilot-wake-apply";
const PATH_UNIT: &str = "/etc/systemd/system/weckpilot-wake.path";
const PATH_UNIT_NAME: &str = "weckpilot-wake.path";

const APP_NAME: &str = "WeckPilot";
const INHIBIT_REASON: &str = "An alarm is active";

/// Die Helper-Dateien sind eingebettet: pkexec laeuft als root und kann in
/// ein FUSE-gemountetes AppImage nicht hineinsehen, deshalb werden sie vor
/// der Installation in ein temporaeres Verzeichnis geschrieben.
const HELPER_FILES: [(&str, &str); 5] = [
    ("install.sh", include_str!("../helpers/install.sh")),
    ("uninstall.sh", include_str!("../helpers/uninstall.sh")),
    (
        "weckpilot-wake-apply",
        include_str!("../helpers/weckpilot-wake-apply"),
    ),
    (
        "weckpilot-wake.service",
        include_str!("../helpers/weckpilot-wake.service"),
    ),
    (
        "weckpilot-wake.path",
        include_str!("../helpers/weckpilot-wake.path"),
    ),
];

// ============================================================
// Sleep Prevention & Screen Wake
// ============================================================

struct SleepInhibitor {
    /// Der ScreenSaver-Inhibitor gilt nur, solange diese Verbindung besteht.
    session: Option<Connection>,
    cookie: Option<u32>,
    /// logind gibt den Inhibitor frei, sobald dieser Deskriptor geschlossen wird.
    _logind_fd: Option<OwnedFd>,
}

impl Drop for SleepInhibitor {
    fn drop(&mut self) {
        if let (Some(session), Some(cookie)) = (&self.session, self.cookie) {
            let _ = session.call_method(
                Some("org.freedesktop.ScreenSaver"),
                "/org/freedesktop/ScreenSaver",
                Some("org.freedesktop.ScreenSaver"),
                "UnInhibit",
                &(cookie,),
            );
        }
    }
}

static INHIBITOR: Mutex<Option<SleepInhibitor>> = Mutex::new(None);
static INHIBITOR_GENERATION: AtomicU64 = AtomicU64::new(0);

fn screensaver_inhibit() -> Option<(Connection, u32)> {
    let session = Connection::session().ok()?;
    let reply = session
        .call_method(
            Some("org.freedesktop.ScreenSaver"),
            "/org/freedesktop/ScreenSaver",
            Some("org.freedesktop.ScreenSaver"),
            "Inhibit",
            &(APP_NAME, INHIBIT_REASON),
        )
        .ok()?;
    let cookie = reply.body().deserialize::<u32>().ok()?;
    Some((session, cookie))
}

fn logind_inhibit() -> Option<OwnedFd> {
    let system = Connection::system().ok()?;
    let reply = system
        .call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1",
            Some("org.freedesktop.login1.Manager"),
            "Inhibit",
            &("sleep:idle", APP_NAME, INHIBIT_REASON, "block"),
        )
        .ok()?;
    reply.body().deserialize::<OwnedFd>().ok()
}

/// Wie `caffeinate -d -t` unter macOS: der Inhibitor endet nach `minutes`
/// Minuten, bei `allow_sleep` oder mit dem Prozess.
pub fn prevent_sleep(minutes: u32) -> Result<String, String> {
    allow_sleep();

    let screensaver = screensaver_inhibit();
    let logind_fd = logind_inhibit();
    if screensaver.is_none() && logind_fd.is_none() {
        return Err("Neither logind nor the screensaver service accepted the inhibit request".into());
    }

    let (session, cookie) = match screensaver {
        Some((session, cookie)) => (Some(session), Some(cookie)),
        None => (None, None),
    };
    let generation = INHIBITOR_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    *INHIBITOR.lock().unwrap_or_else(|e| e.into_inner()) = Some(SleepInhibitor {
        session,
        cookie,
        _logind_fd: logind_fd,
    });

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(u64::from(minutes) * 60));
        if INHIBITOR_GENERATION.load(Ordering::SeqCst) == generation {
            allow_sleep();
        }
    });

    Ok(format!("Sleep prevented for {} minutes", minutes))
}

pub fn allow_sleep() {
    INHIBITOR_GENERATION.fetch_add(1, Ordering::SeqCst);
    let previous = INHIBITOR.lock().unwrap_or_else(|e| e.into_inner()).take();
    drop(previous);
}

pub fn wake_screen() -> Result<String, String> {
    let mut woke = false;

    if let Ok(session) = Connection::session() {
        woke |= session
            .call_method(
                Some("org.freedesktop.ScreenSaver"),
                "/org/freedesktop/ScreenSaver",
                Some("org.freedesktop.ScreenSaver"),
                "SimulateUserActivity",
                &(),
            )
            .is_ok();
    }

    if std::env::var_os("DISPLAY").is_some() {
        woke |= Command::new("xset")
            .args(["dpms", "force", "on"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
    }

    if woke {
        Ok("Screen awakened".to_string())
    } else {
        Err("No screen wake mechanism available".to_string())
    }
}

// ============================================================
// Wake Helper Management
// ============================================================

pub fn helper_installed() -> bool {
    Path::new(HELPER_SCRIPT).exists() && Path::new(PATH_UNIT).exists()
}

pub fn helper_active() -> bool {
    Command::new("systemctl")
        .args(["is-active", "--quiet", PATH_UNIT_NAME])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn write_helper_files() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join(format!("weckpilot-helper-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot create {}: {}", dir.display(), e))?;
    for (name, content) in HELPER_FILES {
        std::fs::write(dir.join(name), content)
            .map_err(|e| format!("Cannot write helper file {}: {}", name, e))?;
    }
    Ok(dir)
}

/// Fuehrt ein Helper-Skript ueber pkexec mit Administratorrechten aus.
fn run_privileged(script: &str, action: &str) -> Result<String, String> {
    let dir = write_helper_files()?;
    let result = Command::new("pkexec")
        .arg("/bin/bash")
        .arg(dir.join(script))
        .arg(&dir)
        .output();
    let _ = std::fs::remove_dir_all(&dir);

    let output = result.map_err(|e| format!("Failed to run pkexec: {}", e))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    // pkexec: 126 = Dialog abgebrochen, 127 = nicht autorisiert
    match output.status.code() {
        Some(126) | Some(127) => Err(format!("{} canceled by the user.", action)),
        _ => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("{} failed: {}{}", action, stderr, stdout))
        }
    }
}

pub fn install_helper() -> Result<String, String> {
    let stdout = run_privileged("install.sh", "Installation")?;
    Ok(format!("Wake Helper installed successfully.\n{}", stdout))
}

pub fn uninstall_helper() -> Result<String, String> {
    run_privileged("uninstall.sh", "Uninstallation")?;
    Ok("Wake Helper uninstalled successfully.".to_string())
}

/// Das Verzeichnis gehoert root; die App darf nur die vom Installer
/// angelegte, ihr gehoerende `schedule.json` ueberschreiben.
pub fn write_schedule(json: &str) -> Result<(), String> {
    if !Path::new(SCHEDULE_FILE).exists() {
        return Err("Wake Helper is not installed".to_string());
    }
    std::fs::write(SCHEDULE_FILE, json).map_err(|e| format!("Cannot write schedule: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_files_sind_vollstaendig_eingebettet() {
        for (name, content) in HELPER_FILES {
            assert!(!content.trim().is_empty(), "{name} ist leer");
        }
        let apply = HELPER_FILES
            .iter()
            .find(|(name, _)| *name == "weckpilot-wake-apply")
            .unwrap()
            .1;
        assert!(apply.contains("rtcwake") || apply.contains("RTCWAKE"));
    }

    fn logind_inhibitors() -> String {
        let output = Command::new("systemd-inhibit")
            .args(["--list", "--no-pager"])
            .output()
            .expect("systemd-inhibit");
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    /// Braucht eine laufende Desktop-Sitzung: `cargo test -- --ignored`
    #[test]
    #[ignore]
    fn prevent_sleep_setzt_und_loest_den_inhibitor() {
        // polkit verweigert logind-Inhibitoren z. B. in xrdp-Sitzungen;
        // dann muss die ScreenSaver-Sperre allein greifen.
        let logind_allowed = logind_inhibit().is_some();

        prevent_sleep(1).expect("inhibit");
        {
            let guard = INHIBITOR.lock().unwrap();
            let inhibitor = guard.as_ref().expect("inhibitor gespeichert");
            assert!(inhibitor.cookie.is_some() || inhibitor._logind_fd.is_some());
        }
        if logind_allowed {
            assert!(logind_inhibitors().contains(APP_NAME));
        }

        allow_sleep();
        std::thread::sleep(Duration::from_millis(300));
        assert!(INHIBITOR.lock().unwrap().is_none());
        assert!(!logind_inhibitors().contains(APP_NAME));
    }

    #[test]
    fn units_zeigen_auf_die_installierten_pfade() {
        let service = HELPER_FILES[3].1;
        let path = HELPER_FILES[4].1;
        assert!(service.contains(&format!("ExecStart={}", HELPER_SCRIPT)));
        assert!(path.contains(&format!("PathChanged={}", SCHEDULE_FILE)));
    }
}
