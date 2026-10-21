# WeckPilot

WeckPilot ist eine moderne, native Wecker-Anwendung für macOS und Linux. Mehrere
Weckzeiten, Wochenpläne und eine sanft ansteigende Lautstärke lassen sich in
einer kompakten Oberfläche verwalten.

## Funktionen

- beliebig viele einmalige oder wiederkehrende Alarme
- individuelle Wochentage und Beschriftungen
- Snooze-Funktion und progressiv ansteigende Lautstärke
- lokale Speicherung der Einstellungen
- deutsche und englische Oberfläche mit automatischer Systemspracherkennung
  und manueller Sprachwahl
- optionaler Wake-Helper, der den Rechner für einen Alarm aufwecken kann
- native, signierte und von Apple notarisierte macOS-App
- Linux-Version mit eigener Farbgebung im Stil des Nebel-Themas

## Download

Die aktuelle signierte Version steht unter
[Releases](https://github.com/nojan01/WeckPilot/releases/latest) als DMG bereit.

Voraussetzung ist ein Mac mit Apple Silicon. Für den optionalen Wake-Helper
werden Administratorrechte und die Xcode Command Line Tools benötigt, weil der
mitgelieferte Swift-Quelltext lokal kompiliert und als LaunchDaemon installiert
wird.

### Linux

Unter Linux (getestet mit Linux Mint 22 / Cinnamon) erstellt WeckPilot eigene
Pakete im Format `.deb` und AppImage. Für die Laufzeit werden WebKitGTK 4.1,
systemd, `rtcwake` (util-linux) und `pkexec` (polkit) benötigt.

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
- **Farbgebung:** Unter Linux nutzt die Oberfläche das Nebel-Thema
  (`dist/theme-nebel.css`): neblige Grautöne mit grünem Mint-Leuchten. Unter
  macOS bleibt das bisherige Aussehen unverändert.

## Entwicklung

Benötigt werden Node.js, Rust und die Tauri-Voraussetzungen für macOS bzw.
Linux. Unter Debian, Ubuntu und Linux Mint sind das:

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev \
  libayatana-appindicator3-dev build-essential curl wget file
```

```bash
npm ci
npm run tauri:dev
```

Einen Release-Build erstellt:

```bash
npm run tauri:build
```

Unter Linux erzeugt `npm run tauri:build:linux` die Pakete `.deb` und AppImage.
Die Tests für das Root-Skript des Linux-Wake-Helpers laufen mit:

```bash
bash src-tauri/helpers/linux/test-wake-apply.sh
```

Der vollständige signierte und notarisierte Release-Prozess verwendet die im
macOS-Schlüsselbund hinterlegte Developer-ID und das Notarisierungsprofil
`DesktopProfileManager`:

```bash
./release-macos.sh
```

Die Web-Oberfläche liegt in `dist/`, der native Tauri-Code in `src-tauri/`.

## Datenschutz

Alarme und Einstellungen werden lokal auf dem Gerät gespeichert. Die Oberfläche
lädt die Schrift Orbitron zur Laufzeit von Google Fonts; dabei kann eine
Verbindung zu Google-Servern entstehen.

## Lizenz

Copyright © 2026 Norbert Jander

WeckPilot ist unter der [MIT-Lizenz](LICENSE) veröffentlicht. Hinweise zu den
verwendeten Open-Source-Komponenten stehen in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

Die interne Bundle-ID bleibt für die Update-Kompatibilität mit bereits
installierten Versionen unverändert. Bestehende Alarme und ein vorhandener
Wake-Helper werden von WeckPilot weiterverwendet.
