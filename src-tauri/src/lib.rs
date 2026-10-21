use serde::{Deserialize, Serialize};

mod linux;

// ============================================================
// Sleep Prevention & Screen Wake
// ============================================================

/// Verhindert den Ruhezustand ueber logind und den Bildschirmschoner-Dienst.
#[tauri::command]
fn prevent_sleep(minutes: u32) -> Result<String, String> {
    linux::prevent_sleep(minutes)
}

/// Hebt die Schlafverhinderung auf.
#[tauri::command]
fn allow_sleep() {
    linux::allow_sleep();
}

/// Prüft ob die App Berechtigung hat, Schlaf zu verhindern
#[tauri::command]
fn check_sleep_permission() -> bool {
    true
}

/// Weckt den Bildschirm auf (beendet Bildschirmschoner)
#[tauri::command]
fn wake_screen() -> Result<String, String> {
    linux::wake_screen()
}

// ============================================================
// Wake Helper Management
// ============================================================

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
fn read_helper_status(
    installed: bool,
    daemon_loaded: bool,
    shared_dir: &str,
    schedule_file: &str,
) -> WakeHelperStatus {
    let (has_schedule, next_wake) = std::fs::read_to_string(schedule_file)
        .ok()
        .and_then(|data| serde_json::from_str::<WakeSchedule>(&data).ok())
        .map_or((false, None), |schedule| {
            (
                schedule.enabled && schedule.next_wake.is_some(),
                schedule.next_wake,
            )
        });

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

/// Serialisiert den Schedule fuer den Helper.
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
    linux::helper_installed()
}

/// Holt den detaillierten Status des Wake Helpers
#[tauri::command]
fn get_wake_helper_status() -> WakeHelperStatus {
    read_helper_status(
        linux::helper_installed(),
        linux::helper_active(),
        linux::SHARED_DIR,
        linux::SCHEDULE_FILE,
    )
}

/// Installiert den Wake Helper (erfordert Root-Rechte via pkexec).
/// Async, damit der Passwortdialog das Fenster nicht blockiert.
#[tauri::command]
async fn install_wake_helper() -> Result<String, String> {
    linux::install_helper()
}

/// Deinstalliert den Wake Helper
#[tauri::command]
async fn uninstall_wake_helper() -> Result<String, String> {
    linux::uninstall_helper()
}

/// Aktualisiert den Wake-Schedule (schreibt schedule.json für den Helper)
#[tauri::command]
fn update_wake_schedule(
    next_wake: Option<String>,
    alarm_time: Option<String>,
    label: Option<String>,
) -> Result<String, String> {
    let message = match &next_wake {
        Some(wake) => format!("Wake schedule updated: {}", wake),
        None => "Wake schedule disabled".to_string(),
    };
    linux::write_schedule(&wake_schedule_json(next_wake, alarm_time, label)?)?;
    Ok(message)
}

/// Plant ein Systemaufwachen (Legacy-Funktion, nutzt den Helper-Mechanismus)
#[tauri::command]
fn schedule_wake(hour: u8, minute: u8) -> Result<String, String> {
    let (iso_formatted, alarm_time) = next_wake_time(hour, minute)?;
    linux::write_schedule(&wake_schedule_json(
        Some(iso_formatted.clone()),
        Some(alarm_time),
        None,
    )?)?;
    Ok(format!("Wake scheduled for {}", iso_formatted))
}

/// Naechster Zeitpunkt `hour:minute` in lokaler Zeit (heute oder morgen).
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

// ============================================================
// Fenstergroesse
// ============================================================

/// Muss zur Fenstergroesse in `tauri.conf.json` passen.
const BASE_WINDOW_SIZE: (f64, f64) = (700.0, 650.0);
const UI_SCALES: [f64; 4] = [0.5, 0.75, 1.0, 1.25];

fn validate_ui_scale(scale: f64) -> Result<f64, String> {
    UI_SCALES
        .iter()
        .copied()
        .find(|allowed| (allowed - scale).abs() < 1e-6)
        .ok_or_else(|| format!("Unsupported scale {}", scale))
}

/// Zoomt die Oberflaeche und passt das Fenster an, damit nichts abgeschnitten wird.
#[tauri::command]
fn set_ui_scale(window: tauri::WebviewWindow, scale: f64) -> Result<(), String> {
    let scale = validate_ui_scale(scale)?;
    window.set_zoom(scale).map_err(|e| e.to_string())?;
    window
        .set_size(tauri::LogicalSize::new(
            BASE_WINDOW_SIZE.0 * scale,
            BASE_WINDOW_SIZE.1 * scale,
        ))
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Kein Menue: GTK wuerde es als Leiste in das rahmenlose Fenster haengen.
    tauri::Builder::default()
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
            set_ui_scale
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
    fn next_wake_time_liegt_in_der_zukunft() {
        let (iso, alarm) = next_wake_time(6, 5).unwrap();
        assert_eq!(alarm, "06:05");
        assert!(iso.ends_with("T06:05:00"));
        let wake = chrono::NaiveDateTime::parse_from_str(&iso, "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(wake > chrono::Local::now().naive_local());

        assert!(next_wake_time(24, 0).is_err());
    }

    #[test]
    fn validate_ui_scale_erlaubt_nur_die_stufen() {
        for scale in [0.5, 0.75, 1.0, 1.25] {
            assert_eq!(validate_ui_scale(scale), Ok(scale));
        }
        for scale in [0.0, 0.25, 1.5, 2.0, -1.0, f64::NAN] {
            assert!(validate_ui_scale(scale).is_err(), "{scale} darf nicht gehen");
        }
    }

    #[test]
    fn schedule_json_ist_ohne_weckzeit_deaktiviert() {
        let json = wake_schedule_json(None, None, None).unwrap();
        let schedule: WakeSchedule = serde_json::from_str(&json).unwrap();
        assert!(!schedule.enabled);

        let json = wake_schedule_json(Some("2026-10-22T06:30:00".into()), None, None).unwrap();
        assert!(json.contains("\"nextWake\": \"2026-10-22T06:30:00\""));
        assert!(json.contains("\"enabled\": true"));
    }
}
