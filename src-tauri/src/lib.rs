use serde::{Deserialize, Serialize};
#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "linux")]
mod linux;

// ============================================================
// Sleep Prevention & Screen Wake
// ============================================================

/// Verhindert Schlafmodus mit macOS caffeinate
#[tauri::command]
fn prevent_sleep(minutes: u32) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let seconds = minutes * 60;
        let output = Command::new("caffeinate")
            .args(["-d", "-t", &seconds.to_string()])
            .spawn();

        match output {
            Ok(_) => Ok(format!("Sleep prevented for {} minutes", minutes)),
            Err(e) => Err(format!("Error: {}", e)),
        }
    }

    #[cfg(target_os = "linux")]
    {
        linux::prevent_sleep(minutes)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = minutes;
        Err("Only supported on macOS and Linux".to_string())
    }
}

/// Hebt die Schlafverhinderung auf. Unter macOS laeuft `caffeinate` bis zum
/// eigenen Timeout weiter.
#[tauri::command]
fn allow_sleep() {
    #[cfg(target_os = "linux")]
    linux::allow_sleep();
}

/// Prüft ob die App Berechtigung hat, Schlaf zu verhindern
#[tauri::command]
fn check_sleep_permission() -> bool {
    cfg!(any(target_os = "macos", target_os = "linux"))
}

/// Weckt den Bildschirm auf (beendet Bildschirmschoner)
#[tauri::command]
fn wake_screen() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("caffeinate").args(["-u", "-t", "5"]).spawn();

        match output {
            Ok(_) => Ok("Screen awakened".to_string()),
            Err(e) => Err(format!("Error: {}", e)),
        }
    }

    #[cfg(target_os = "linux")]
    {
        linux::wake_screen()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err("Only supported on macOS and Linux".to_string())
    }
}

// ============================================================
// Wake Helper Management
// ============================================================

#[cfg(target_os = "macos")]
const HELPER_BINARY: &str = "/usr/local/bin/WeckPilotWakeHelper";
#[cfg(target_os = "macos")]
const HELPER_PLIST: &str = "/Library/LaunchDaemons/de.little-tools.weckpilot.wake-helper.plist";
#[cfg(target_os = "macos")]
const HELPER_LABEL: &str = "de.little-tools.weckpilot.wake-helper";
#[cfg(target_os = "macos")]
const SHARED_DIR: &str = "/Users/Shared/WeckPilot";
#[cfg(target_os = "macos")]
const SCHEDULE_FILE: &str = "/Users/Shared/WeckPilot/schedule.json";

// Technische Altpfade bleiben lesbar, damit bestehende Installationen nach
// der Umbenennung ohne Neuinstallation oder Verlust des Wake-Schedules laufen.
#[cfg(target_os = "macos")]
const LEGACY_HELPER_BINARY: &str = "/usr/local/bin/AlarmMasterWakeHelper";
#[cfg(target_os = "macos")]
const LEGACY_HELPER_PLIST: &str = "/Library/LaunchDaemons/com.alarmmaster.wake-helper.plist";
#[cfg(target_os = "macos")]
const LEGACY_HELPER_LABEL: &str = "com.alarmmaster.wake-helper";
#[cfg(target_os = "macos")]
const LEGACY_SHARED_DIR: &str = "/Users/Shared/AlarmMaster";
#[cfg(target_os = "macos")]
const LEGACY_SCHEDULE_FILE: &str = "/Users/Shared/AlarmMaster/schedule.json";

#[cfg(target_os = "macos")]
fn current_helper_installed() -> bool {
    std::path::Path::new(HELPER_BINARY).exists() && std::path::Path::new(HELPER_PLIST).exists()
}

#[cfg(target_os = "macos")]
fn legacy_helper_installed() -> bool {
    std::path::Path::new(LEGACY_HELPER_BINARY).exists()
        && std::path::Path::new(LEGACY_HELPER_PLIST).exists()
}

#[cfg(target_os = "macos")]
fn active_helper_paths() -> (&'static str, &'static str, &'static str) {
    if current_helper_installed() || !legacy_helper_installed() {
        (SHARED_DIR, SCHEDULE_FILE, HELPER_LABEL)
    } else {
        (LEGACY_SHARED_DIR, LEGACY_SCHEDULE_FILE, LEGACY_HELPER_LABEL)
    }
}

#[derive(Serialize, Deserialize)]
struct WakeSchedule {
    #[serde(rename = "nextWake")]
    next_wake: Option<String>,
    enabled: bool,
    #[serde(rename = "alarmTime")]
    alarm_time: Option<String>,
    label: Option<String>,
}

#[derive(Serialize)]
struct WakeHelperStatus {
    installed: bool,
    daemon_loaded: bool,
    has_schedule: bool,
    next_wake: Option<String>,
    log_tail: Option<String>,
}

/// Liest Schedule und die letzten Log-Zeilen des Helpers aus dessen Verzeichnis.
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn read_helper_status(
    installed: bool,
    daemon_loaded: bool,
    shared_dir: &str,
    schedule_file: &str,
) -> WakeHelperStatus {
    let (has_schedule, next_wake) = if let Ok(data) = std::fs::read_to_string(schedule_file) {
        if let Ok(schedule) = serde_json::from_str::<WakeSchedule>(&data) {
            (
                schedule.enabled && schedule.next_wake.is_some(),
                schedule.next_wake,
            )
        } else {
            (false, None)
        }
    } else {
        (false, None)
    };

    let log_path = format!("{}/helper.log", shared_dir);
    let log_tail = std::fs::read_to_string(&log_path).ok().map(|content| {
        let lines: Vec<&str> = content.lines().collect();
        let start = lines.len().saturating_sub(10);
        lines[start..].join("\n")
    });

    WakeHelperStatus {
        installed,
        daemon_loaded,
        has_schedule,
        next_wake,
        log_tail,
    }
}

/// Serialisiert den Schedule fuer den Linux-Helper.
#[cfg(target_os = "linux")]
fn wake_schedule_json(
    next_wake: Option<String>,
    alarm_time: Option<String>,
    label: Option<String>,
) -> Result<String, String> {
    let schedule = WakeSchedule {
        enabled: next_wake.is_some(),
        next_wake,
        alarm_time,
        label,
    };
    serde_json::to_string_pretty(&schedule).map_err(|e| format!("JSON error: {}", e))
}

/// Prüft ob der Wake Helper installiert ist
#[tauri::command]
fn is_wake_helper_installed() -> bool {
    #[cfg(target_os = "macos")]
    {
        current_helper_installed() || legacy_helper_installed()
    }

    #[cfg(target_os = "linux")]
    {
        linux::helper_installed()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    false
}

/// Holt den detaillierten Status des Wake Helpers
#[tauri::command]
fn get_wake_helper_status() -> WakeHelperStatus {
    #[cfg(target_os = "macos")]
    {
        let installed = current_helper_installed() || legacy_helper_installed();
        let (shared_dir, schedule_file, helper_label) = active_helper_paths();

        // Check if daemon is loaded
        let daemon_loaded = Command::new("launchctl")
            .args(["list"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(helper_label))
            .unwrap_or(false);

        read_helper_status(installed, daemon_loaded, shared_dir, schedule_file)
    }

    #[cfg(target_os = "linux")]
    {
        read_helper_status(
            linux::helper_installed(),
            linux::helper_active(),
            linux::SHARED_DIR,
            linux::SCHEDULE_FILE,
        )
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        WakeHelperStatus {
            installed: false,
            daemon_loaded: false,
            has_schedule: false,
            next_wake: None,
            log_tail: None,
        }
    }
}

/// Installiert den Wake Helper (erfordert Admin-Rechte)
#[tauri::command]
async fn install_wake_helper(app: tauri::AppHandle) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        use tauri::Manager;

        let resource_dir = app
            .path()
            .resource_dir()
            .map_err(|e| format!("Resource dir error: {}", e))?;
        let helpers_dir = resource_dir.join("helpers");

        // Verify helper files exist
        let install_script = helpers_dir.join("install.sh");
        if !install_script.exists() {
            return Err(format!(
                "Install script not found at: {}",
                install_script.display()
            ));
        }

        // Run install script with admin privileges via osascript
        let script = format!(
            "do shell script \"bash '{}' '{}'\" with administrator privileges",
            install_script.display(),
            helpers_dir.display()
        );

        let output = Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("Failed to run installer: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(format!("Wake Helper installed successfully.\n{}", stdout))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("User canceled") || stderr.contains("-128") {
                Err("Installation canceled by the user.".to_string())
            } else {
                Err(format!("Installation failed: {}", stderr))
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let _ = app;
        linux::install_helper()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = app;
        Err("Only supported on macOS and Linux".to_string())
    }
}

/// Deinstalliert den Wake Helper
#[tauri::command]
async fn uninstall_wake_helper(app: tauri::AppHandle) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        use tauri::Manager;

        let resource_dir = app
            .path()
            .resource_dir()
            .map_err(|e| format!("Resource dir error: {}", e))?;
        let helpers_dir = resource_dir.join("helpers");
        let uninstall_script = helpers_dir.join("uninstall.sh");

        if !uninstall_script.exists() {
            return Err(format!(
                "Uninstall script not found at: {}",
                uninstall_script.display()
            ));
        }

        let script = format!(
            "do shell script \"bash '{}'\" with administrator privileges",
            uninstall_script.display()
        );

        let output = Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("Failed to run uninstaller: {}", e))?;

        if output.status.success() {
            Ok("Wake Helper uninstalled successfully.".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("User canceled") || stderr.contains("-128") {
                Err("Uninstallation canceled by the user.".to_string())
            } else {
                Err(format!("Uninstallation failed: {}", stderr))
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let _ = app;
        linux::uninstall_helper()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = app;
        Err("Only supported on macOS and Linux".to_string())
    }
}

/// Aktualisiert den Wake-Schedule (schreibt schedule.json für den Helper)
#[tauri::command]
fn update_wake_schedule(
    next_wake: Option<String>,
    alarm_time: Option<String>,
    label: Option<String>,
) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let (shared_dir, schedule_file, _) = active_helper_paths();

        // Ensure shared directory exists
        std::fs::create_dir_all(shared_dir)
            .map_err(|e| format!("Cannot create directory: {}", e))?;

        // Set directory permissions so both user and daemon can access
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o777);
            std::fs::set_permissions(shared_dir, perms).ok();
        }

        let schedule = WakeSchedule {
            next_wake: next_wake.clone(),
            enabled: next_wake.is_some(),
            alarm_time,
            label,
        };

        let json =
            serde_json::to_string_pretty(&schedule).map_err(|e| format!("JSON error: {}", e))?;

        std::fs::write(schedule_file, &json)
            .map_err(|e| format!("Cannot write schedule: {}", e))?;

        // Set file permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o666);
            std::fs::set_permissions(schedule_file, perms).ok();
        }

        match &schedule.next_wake {
            Some(wake) => Ok(format!("Wake schedule updated: {}", wake)),
            None => Ok("Wake schedule disabled".to_string()),
        }
    }

    #[cfg(target_os = "linux")]
    {
        let message = match &next_wake {
            Some(wake) => format!("Wake schedule updated: {}", wake),
            None => "Wake schedule disabled".to_string(),
        };
        linux::write_schedule(&wake_schedule_json(next_wake, alarm_time, label)?)?;
        Ok(message)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (next_wake, alarm_time, label);
        Err("Only supported on macOS and Linux".to_string())
    }
}

/// Plant ein Systemaufwachen (Legacy-Funktion, nutzt jetzt den Helper-Mechanismus)
#[tauri::command]
fn schedule_wake(hour: u8, minute: u8) -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        let (iso_formatted, alarm_time) = next_wake_time(hour, minute)?;
        linux::write_schedule(&wake_schedule_json(
            Some(iso_formatted.clone()),
            Some(alarm_time),
            None,
        )?)?;
        Ok(format!("Wake scheduled for {}", iso_formatted))
    }

    #[cfg(target_os = "macos")]
    {
        use chrono::{Duration, Local};

        let now = Local::now();
        let mut wake_time = now
            .date_naive()
            .and_hms_opt(hour as u32, minute as u32, 0)
            .unwrap();

        if wake_time.and_local_timezone(Local).unwrap() <= now {
            wake_time = wake_time + Duration::days(1);
        }

        let iso_formatted = wake_time.format("%Y-%m-%dT%H:%M:%S").to_string();
        let alarm_time = format!("{:02}:{:02}", hour, minute);

        // Write schedule file for the helper
        let schedule = WakeSchedule {
            next_wake: Some(iso_formatted.clone()),
            enabled: true,
            alarm_time: Some(alarm_time),
            label: None,
        };

        let (shared_dir, schedule_file, _) = active_helper_paths();
        std::fs::create_dir_all(shared_dir).ok();
        let json =
            serde_json::to_string_pretty(&schedule).map_err(|e| format!("JSON error: {}", e))?;
        std::fs::write(schedule_file, &json).map_err(|e| format!("Write error: {}", e))?;

        Ok(format!("Wake scheduled for {}", iso_formatted))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (hour, minute);
        Err("Only supported on macOS and Linux".to_string())
    }
}

/// Naechster Zeitpunkt `hour:minute` in lokaler Zeit (heute oder morgen).
#[cfg(target_os = "linux")]
fn next_wake_time(hour: u8, minute: u8) -> Result<(String, String), String> {
    use chrono::{Duration, Local};

    let now = Local::now();
    let mut wake_time = now
        .date_naive()
        .and_hms_opt(hour as u32, minute as u32, 0)
        .ok_or_else(|| format!("Invalid time {:02}:{:02}", hour, minute))?;

    if wake_time
        .and_local_timezone(Local)
        .earliest()
        .map_or(true, |t| t <= now)
    {
        wake_time += Duration::days(1);
    }

    Ok((
        wake_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
        format!("{:02}:{:02}", hour, minute),
    ))
}

/// Beschriftungen der macOS-Menueleiste.
///
/// muda setzt fuer `PredefinedMenuItem` ohne Text englische Standardtexte ein
/// und leitet den Namen aus `NSRunningApplication::localizedName` ab. Das
/// Executable dieses Bundles heisst "app", deshalb erscheint dort sonst
/// "About app" statt "Über WeckPilot". Jeder Eintrag wird daher ausdruecklich
/// gesetzt -- das korrigiert zugleich den Namen und die Sprache.
#[cfg(target_os = "macos")]
struct MenuLabels {
    about: String,
    hide: String,
    quit: String,
    edit: &'static str,
    undo: &'static str,
    redo: &'static str,
    cut: &'static str,
    copy: &'static str,
    paste: &'static str,
    select_all: &'static str,
    window: &'static str,
    minimize: &'static str,
    maximize: &'static str,
    close: &'static str,
}

/// Die Texte folgen der Terminologie, die macOS selbst verwendet, damit das
/// Menue sich nicht von anderen Programmen unterscheidet.
#[cfg(target_os = "macos")]
fn menu_labels(language: &str) -> MenuLabels {
    const APP_NAME: &str = "WeckPilot";

    if language == "de" {
        MenuLabels {
            about: format!("Über {APP_NAME}"),
            hide: format!("{APP_NAME} ausblenden"),
            quit: format!("{APP_NAME} beenden"),
            edit: "Bearbeiten",
            undo: "Widerrufen",
            redo: "Wiederholen",
            cut: "Ausschneiden",
            copy: "Kopieren",
            paste: "Einsetzen",
            select_all: "Alles auswählen",
            window: "Fenster",
            minimize: "Im Dock ablegen",
            maximize: "Zoomen",
            close: "Schließen",
        }
    } else {
        MenuLabels {
            about: format!("About {APP_NAME}"),
            hide: format!("Hide {APP_NAME}"),
            quit: format!("Quit {APP_NAME}"),
            edit: "Edit",
            undo: "Undo",
            redo: "Redo",
            cut: "Cut",
            copy: "Copy",
            paste: "Paste",
            select_all: "Select All",
            window: "Window",
            minimize: "Minimize",
            maximize: "Zoom",
            close: "Close",
        }
    }
}

/// Gleiche Regel wie `detectLanguage()` in `dist/i18n.js`: alles, was mit "de"
/// beginnt, ist Deutsch, alles andere faellt auf Englisch zurueck.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn normalize_language(raw: &str) -> &'static str {
    if raw.trim().to_ascii_lowercase().starts_with("de") {
        "de"
    } else {
        "en"
    }
}

/// Liest die erste Sprache aus der Plist-Liste, die `defaults` ausgibt:
/// `(\n    "de-DE",\n    "en-DE"\n)`. Die erste Zeile gewinnt, denn macOS
/// sortiert die Liste nach Vorrang.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn parse_apple_languages(output: &str) -> Option<&'static str> {
    output.lines().find_map(|line| {
        let start = line.find('"')? + 1;
        let rest = &line[start..];
        let end = rest.find('"')?;
        Some(normalize_language(&rest[..end]))
    })
}

/// Sprache fuer den ersten Menueaufbau. Die tatsaechliche Auswahl des Nutzers
/// liegt im localStorage des Webviews und ist zu diesem Zeitpunkt noch nicht
/// lesbar; das Frontend meldet sie direkt nach dem Laden nach.
#[cfg(target_os = "macos")]
fn system_language() -> &'static str {
    let output = match std::process::Command::new("defaults")
        .args(["read", "-g", "AppleLanguages"])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return "en",
    };

    parse_apple_languages(&String::from_utf8_lossy(&output.stdout)).unwrap_or("en")
}

#[cfg(target_os = "macos")]
fn build_macos_menu<R: tauri::Runtime, M: tauri::Manager<R>>(
    manager: &M,
    language: &str,
) -> tauri::Result<tauri::menu::Menu<R>> {
    use tauri::menu::{AboutMetadataBuilder, Menu, PredefinedMenuItem, Submenu};

    let labels = menu_labels(language);

    let about = AboutMetadataBuilder::new()
        .name(Some("WeckPilot"))
        .version(Some(manager.package_info().version.to_string()))
        .copyright(Some("Copyright © 2026 Norbert Jander"))
        .credits(Some(include_str!("../../LICENSE")))
        .build();

    let app_menu = Submenu::with_items(
        manager,
        "WeckPilot",
        true,
        &[
            &PredefinedMenuItem::about(manager, Some(&labels.about), Some(about))?,
            &PredefinedMenuItem::separator(manager)?,
            &PredefinedMenuItem::hide(manager, Some(&labels.hide))?,
            &PredefinedMenuItem::separator(manager)?,
            &PredefinedMenuItem::quit(manager, Some(&labels.quit))?,
        ],
    )?;

    // Ohne diese Eintraege reicht macOS Cmd+C/V/A nicht an den
    // Webview weiter; das Namensfeld eines Weckers waere dann
    // nicht mehr per Tastatur zu befuellen.
    let edit_menu = Submenu::with_items(
        manager,
        labels.edit,
        true,
        &[
            &PredefinedMenuItem::undo(manager, Some(labels.undo))?,
            &PredefinedMenuItem::redo(manager, Some(labels.redo))?,
            &PredefinedMenuItem::separator(manager)?,
            &PredefinedMenuItem::cut(manager, Some(labels.cut))?,
            &PredefinedMenuItem::copy(manager, Some(labels.copy))?,
            &PredefinedMenuItem::paste(manager, Some(labels.paste))?,
            &PredefinedMenuItem::select_all(manager, Some(labels.select_all))?,
        ],
    )?;

    let window_menu = Submenu::with_items(
        manager,
        labels.window,
        true,
        &[
            &PredefinedMenuItem::minimize(manager, Some(labels.minimize))?,
            &PredefinedMenuItem::maximize(manager, Some(labels.maximize))?,
            &PredefinedMenuItem::separator(manager)?,
            &PredefinedMenuItem::close_window(manager, Some(labels.close))?,
        ],
    )?;

    Menu::with_items(manager, &[&app_menu, &edit_menu, &window_menu])
}

/// Wird vom Frontend beim Start und bei jedem Sprachwechsel aufgerufen.
#[tauri::command]
fn set_menu_language(app: tauri::AppHandle, language: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let menu = build_macos_menu(&app, normalize_language(&language))
            .map_err(|error| error.to_string())?;
        app.set_menu(menu).map_err(|error| error.to_string())?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (&app, &language);
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().enable_macos_default_menu(false);

    // Unter Linux haengt GTK ein Menue als Leiste in das rahmenlose Fenster;
    // dort gibt es deshalb keines.
    #[cfg(target_os = "macos")]
    let builder = builder.menu(|app| build_macos_menu(app, system_language()));

    builder
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            prevent_sleep,
            allow_sleep,
            schedule_wake,
            check_sleep_permission,
            wake_screen,
            is_wake_helper_installed,
            get_wake_helper_status,
            install_wake_helper,
            uninstall_wake_helper,
            update_wake_schedule,
            set_menu_language
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_language_folgt_der_regel_des_frontends() {
        for deutsch in ["de", "de-DE", "DE-de", " de-AT ", "de_CH"] {
            assert_eq!(normalize_language(deutsch), "de", "{deutsch} ist Deutsch");
        }

        // Gegenprobe: alles andere faellt auf Englisch zurueck.
        for englisch in ["en", "en-US", "fr-FR", "", "nl"] {
            assert_eq!(normalize_language(englisch), "en", "{englisch} ist nicht Deutsch");
        }
    }

    #[test]
    fn parse_apple_languages_nimmt_die_erste_sprache() {
        let deutsch = "(\n    \"de-DE\"\n)\n";
        assert_eq!(parse_apple_languages(deutsch), Some("de"));

        // macOS sortiert nach Vorrang, die erste Zeile entscheidet.
        let englisch_zuerst = "(\n    \"en-US\",\n    \"de-DE\"\n)\n";
        assert_eq!(parse_apple_languages(englisch_zuerst), Some("en"));

        // Gegenprobe: ohne Anfuehrungszeichen gibt es nichts zu lesen.
        assert_eq!(parse_apple_languages("(\n)\n"), None);
        assert_eq!(parse_apple_languages(""), None);
    }

    /// Der eigentliche Fehler: ohne ausdruecklichen Text bildet muda den
    /// Eintrag aus `NSRunningApplication::localizedName`, und das Executable
    /// dieses Bundles heisst "app" -- im Menue stand deshalb "About app".
    #[cfg(target_os = "macos")]
    #[test]
    fn menu_labels_nennt_das_produkt_und_nicht_das_executable() {
        for sprache in ["de", "en"] {
            let labels = menu_labels(sprache);

            for eintrag in [&labels.about, &labels.hide, &labels.quit] {
                assert!(
                    eintrag.contains("WeckPilot"),
                    "{sprache}: {eintrag:?} nennt das Produkt nicht"
                );
                assert!(
                    !eintrag.ends_with(" app"),
                    "{sprache}: {eintrag:?} traegt noch den Executable-Namen"
                );
            }
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn menu_labels_uebersetzt_jeden_eintrag() {
        let de = menu_labels("de");
        let en = menu_labels("en");

        assert_eq!(de.about, "Über WeckPilot");
        assert_eq!(en.about, "About WeckPilot");

        // Kein Eintrag darf in beiden Sprachen gleich bleiben, sonst waere
        // die Uebersetzung unvollstaendig.
        let paare: [(&str, &str, &str); 11] = [
            ("edit", de.edit, en.edit),
            ("undo", de.undo, en.undo),
            ("redo", de.redo, en.redo),
            ("cut", de.cut, en.cut),
            ("copy", de.copy, en.copy),
            ("paste", de.paste, en.paste),
            ("select_all", de.select_all, en.select_all),
            ("window", de.window, en.window),
            ("minimize", de.minimize, en.minimize),
            ("maximize", de.maximize, en.maximize),
            ("close", de.close, en.close),
        ];

        for (feld, deutsch, englisch) in paare {
            assert_ne!(deutsch, englisch, "{feld} ist nicht uebersetzt");
            assert!(!deutsch.is_empty() && !englisch.is_empty(), "{feld} ist leer");
        }
    }
}
