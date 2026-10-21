# WeckPilot für Linux

WeckPilot ist eine moderne Wecker-Anwendung für Linux. Mehrere Weckzeiten,
Wochenpläne und eine sanft ansteigende Lautstärke lassen sich in einer
kompakten Oberfläche verwalten. Die Farbgebung folgt dem Nebel-Thema: neblige
Grautöne mit grün leuchtenden Ziffern im Stil des Linux-Mint-Logos.

Die macOS-Version wird getrennt im Repository
[nojan01/WeckPilot](https://github.com/nojan01/WeckPilot) gepflegt.

## Funktionen

- beliebig viele einmalige oder wiederkehrende Alarme
- individuelle Wochentage und Beschriftungen
- Snooze-Funktion und progressiv ansteigende Lautstärke
- lokale Speicherung der Einstellungen
- deutsche und englische Oberfläche mit automatischer Systemspracherkennung
  und manueller Sprachwahl
- Fenstergröße in Stufen von 50 %, 75 %, 100 % und 125 %
- Energieverwaltung, die den Ruhezustand verhindert, solange ein Wecker aktiv ist
- optionaler Wake-Helper, der den Rechner für einen Alarm aus dem Ruhezustand
  aufweckt

## Installation

WeckPilot wird als `.deb`-Paket, AppImage und `.rpm` gebaut. Unter Debian,
Ubuntu und Linux Mint:

```bash
sudo apt install ./WeckPilot_1.0.1_amd64.deb
```

Benötigt werden WebKitGTK 4.1, systemd, `rtcwake` (util-linux) und `pkexec`
(polkit). Das Debian-Paket zieht diese Abhängigkeiten automatisch nach.
Getestet wurde unter Linux Mint 22 mit Cinnamon.

## Energieverwaltung und Wake-Helper

- **Energieverwaltung:** Solange die Option aktiv ist, blockiert WeckPilot den
  Ruhezustand über logind und den Bildschirmschoner über
  `org.freedesktop.ScreenSaver` (D-Bus). In manchen Remote-Sitzungen
  (z. B. xrdp) erlaubt polkit keine logind-Sperre. Dann wirkt nur die
  Bildschirmschoner-Sperre.
- **Wake-Helper:** Die Installation erfolgt nach Passwortabfrage über `pkexec`.
  Dabei werden die systemd-Units `weckpilot-wake.path` und
  `weckpilot-wake.service` sowie das Skript
  `/usr/local/lib/weckpilot/weckpilot-wake-apply` angelegt. Bei jeder Änderung
  des Weckplans in `/var/lib/weckpilot/schedule.json` und vor jedem Ruhezustand
  stellt das Skript mit `rtcwake` die RTC-Weckzeit eine Minute vor dem Alarm.
  Das Mainboard muss das Aufwachen per RTC unterstützen.

## Entwicklung

Benötigt werden Node.js, Rust und die Tauri-Voraussetzungen für Linux. Unter
Debian, Ubuntu und Linux Mint:

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev \
  libayatana-appindicator3-dev build-essential curl wget file
```

```bash
npm ci
npm run tauri:dev
```

Pakete (`.deb`, AppImage, `.rpm`) erstellt:

```bash
npm run tauri:build
```

Sie liegen danach unter `src-tauri/target/release/bundle/`.

Tests:

```bash
cd src-tauri && cargo test                      # Unit-Tests
cd src-tauri && cargo test -- --include-ignored # zusätzlich D-Bus, braucht Desktop-Sitzung
bash src-tauri/helpers/test-wake-apply.sh       # Root-Skript des Wake-Helpers
```

Die Web-Oberfläche liegt in `dist/` (Farbgebung in `dist/theme-nebel.css`),
der native Tauri-Code in `src-tauri/`, der Wake-Helper in `src-tauri/helpers/`.

## Datenschutz

Alarme und Einstellungen werden lokal auf dem Gerät gespeichert. Die Oberfläche
lädt die Schrift Orbitron zur Laufzeit von Google Fonts; dabei kann eine
Verbindung zu Google-Servern entstehen.

## Lizenz

Copyright © 2026 Norbert Jander

WeckPilot ist unter der [MIT-Lizenz](LICENSE) veröffentlicht. Hinweise zu den
verwendeten Open-Source-Komponenten stehen in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
