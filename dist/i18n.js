/**
 * WeckPilot interface translations.
 * English is the fallback; German is selected automatically on German systems.
 */

const I18n = (() => {
    const STORAGE_KEY = 'weckpilot-language';
    const translations = {
        en: {
            language: 'Language',
            settings: 'Settings',
            quit: 'Quit WeckPilot',
            toggleTimeFormat: 'Toggle 12/24-hour format',
            alarmSettings: 'Alarm Settings',
            addAlarm: '+ Add Alarm',
            noAlarmSet: 'No alarm set',
            noAlarms: 'No alarms set',
            sleepMode: '🌙 Sleep Mode',
            brightness: 'Brightness',
            autoStart: 'Auto-start at',
            autoEnd: 'Auto-end at',
            powerManagement: '⚡ Power Management',
            powerInactive: 'Prevents your computer from sleeping while an alarm is active',
            powerActive: '✓ Computer sleep is being prevented',
            wakeFromSleep: 'Wake from Sleep',
            checking: 'Checking…',
            wakeHelperDescription: 'Installs a system service that wakes your computer from suspend before an alarm rings.',
            installHelper: 'Install Helper',
            uninstallHelper: 'Uninstall Helper',
            installing: 'Installing…',
            removing: 'Removing…',
            active: 'Active',
            notInstalled: 'Not installed',
            nextWake: 'Next wake: {value}',
            newAlarm: 'New Alarm',
            editAlarm: 'Edit Alarm',
            close: 'Close',
            alarmTime: 'Alarm Time',
            label: 'Label',
            labelPlaceholder: 'e.g. Wake up',
            type: 'Type',
            recurring: 'Recurring',
            once: 'Once',
            days: 'Days',
            sound: 'Sound',
            volumeFade: 'Volume Fade',
            fadeIn: 'Fade in',
            snooze: 'Snooze',
            snoozeMinutes: 'Snooze: {minutes} min',
            delete: 'Delete',
            save: 'Save',
            alarmRinging: '⏰ ALARM!',
            stop: 'Stop',
            today: 'Today',
            tomorrow: 'Tomorrow',
            alarm: 'Alarm',
            selectDay: 'Please select at least one day.',
            tauriUnavailable: 'The Tauri API is unavailable. Please run WeckPilot as a desktop app.',
            error: 'Error: {value}',
            weekdays: {
                full: { so: 'Sunday', mo: 'Monday', di: 'Tuesday', mi: 'Wednesday', do: 'Thursday', fr: 'Friday', sa: 'Saturday' },
                short: { so: 'SUN', mo: 'MON', di: 'TUE', mi: 'WED', do: 'THU', fr: 'FRI', sa: 'SAT' },
                narrow: { so: 'S', mo: 'M', di: 'T', mi: 'W', do: 'T', fr: 'F', sa: 'S' }
            },
            sounds: {
                gentleRise: '🌅 Gentle Rise', morningBirds: '🐦 Morning Birds', oceanWaves: '🌊 Ocean Waves',
                windChimes: '🎐 Wind Chimes', softPiano: '🎹 Soft Piano', zenBells: '🔔 Zen Bells',
                digitalBeep: '📟 Digital Beep', classicAlarm: '⏰ Classic Alarm', rooster: '🐓 Rooster', harp: '🎵 Harp'
            }
        },
        de: {
            language: 'Sprache',
            settings: 'Einstellungen',
            quit: 'WeckPilot beenden',
            toggleTimeFormat: 'Zwischen 12- und 24-Stunden-Format wechseln',
            alarmSettings: 'Wecker-Einstellungen',
            addAlarm: '+ Wecker hinzufügen',
            noAlarmSet: 'Kein Wecker gestellt',
            noAlarms: 'Keine Wecker eingerichtet',
            sleepMode: '🌙 Nachtmodus',
            brightness: 'Helligkeit',
            autoStart: 'Automatisch ab',
            autoEnd: 'Automatisch bis',
            powerManagement: '⚡ Energieverwaltung',
            powerInactive: 'Verhindert den Ruhezustand des Rechners, solange ein Wecker aktiv ist',
            powerActive: '✓ Ruhezustand des Rechners wird verhindert',
            wakeFromSleep: 'Aus Ruhezustand aufwecken',
            checking: 'Wird geprüft…',
            wakeHelperDescription: 'Installiert einen Systemdienst, der den Rechner vor einem Wecktermin aus dem Ruhezustand aufweckt.',
            installHelper: 'Systemdienst installieren',
            uninstallHelper: 'Systemdienst entfernen',
            installing: 'Wird installiert…',
            removing: 'Wird entfernt…',
            active: 'Aktiv',
            notInstalled: 'Nicht installiert',
            nextWake: 'Nächstes Aufwachen: {value}',
            newAlarm: 'Neuer Wecker',
            editAlarm: 'Wecker bearbeiten',
            close: 'Schließen',
            alarmTime: 'Weckzeit',
            label: 'Bezeichnung',
            labelPlaceholder: 'z. B. Aufstehen',
            type: 'Typ',
            recurring: 'Wiederkehrend',
            once: 'Einmalig',
            days: 'Tage',
            sound: 'Weckton',
            volumeFade: 'Lautstärke',
            fadeIn: 'Langsam lauter werden',
            snooze: 'Schlummern',
            snoozeMinutes: 'Schlummern: {minutes} Min.',
            delete: 'Löschen',
            save: 'Speichern',
            alarmRinging: '⏰ WECKER!',
            stop: 'Beenden',
            today: 'Heute',
            tomorrow: 'Morgen',
            alarm: 'Wecker',
            selectDay: 'Bitte mindestens einen Tag auswählen.',
            tauriUnavailable: 'Die Tauri-API ist nicht verfügbar. Bitte WeckPilot als Desktop-App starten.',
            error: 'Fehler: {value}',
            weekdays: {
                full: { so: 'Sonntag', mo: 'Montag', di: 'Dienstag', mi: 'Mittwoch', do: 'Donnerstag', fr: 'Freitag', sa: 'Samstag' },
                short: { so: 'SO', mo: 'MO', di: 'DI', mi: 'MI', do: 'DO', fr: 'FR', sa: 'SA' },
                narrow: { so: 'S', mo: 'M', di: 'D', mi: 'M', do: 'D', fr: 'F', sa: 'S' }
            },
            sounds: {
                gentleRise: '🌅 Sanftes Erwachen', morningBirds: '🐦 Morgenvögel', oceanWaves: '🌊 Meereswellen',
                windChimes: '🎐 Windspiel', softPiano: '🎹 Sanftes Klavier', zenBells: '🔔 Zen-Glocken',
                digitalBeep: '📟 Digitaler Signalton', classicAlarm: '⏰ Klassischer Wecker', rooster: '🐓 Hahn', harp: '🎵 Harfe'
            }
        }
    };

    let currentLanguage = 'en';

    function getNestedValue(object, path) {
        return path.split('.').reduce((value, key) => value?.[key], object);
    }

    function detectLanguage() {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (saved === 'de' || saved === 'en') return saved;
        const languages = navigator.languages?.length ? navigator.languages : [navigator.language];
        return languages.some(language => language?.toLowerCase().startsWith('de')) ? 'de' : 'en';
    }

    function t(key, values = {}) {
        const value = getNestedValue(translations[currentLanguage], key)
            ?? getNestedValue(translations.en, key)
            ?? key;
        if (typeof value !== 'string') return key;
        return value.replace(/\{(\w+)\}/g, (_, name) => values[name] ?? `{${name}}`);
    }

    function apply(root = document) {
        document.documentElement.lang = currentLanguage;
        root.querySelectorAll('[data-i18n]').forEach(element => {
            element.textContent = t(element.dataset.i18n);
        });
        root.querySelectorAll('[data-i18n-title]').forEach(element => {
            element.title = t(element.dataset.i18nTitle);
        });
        root.querySelectorAll('[data-i18n-aria-label]').forEach(element => {
            element.setAttribute('aria-label', t(element.dataset.i18nAriaLabel));
        });
        root.querySelectorAll('[data-i18n-placeholder]').forEach(element => {
            element.placeholder = t(element.dataset.i18nPlaceholder);
        });
        const selector = document.getElementById('language-select');
        if (selector) selector.value = currentLanguage;
    }

    function setLanguage(language, persist = true) {
        currentLanguage = language === 'de' ? 'de' : 'en';
        if (persist) localStorage.setItem(STORAGE_KEY, currentLanguage);
        apply();
        window.dispatchEvent(new CustomEvent('weckpilot-language-changed', {
            detail: { language: currentLanguage }
        }));
    }

    function init() {
        currentLanguage = detectLanguage();
        apply();
        document.getElementById('language-select')?.addEventListener('change', event => {
            setLanguage(event.target.value);
        });
    }

    return { init, setLanguage, getLanguage: () => currentLanguage, t, apply };
})();

window.I18n = I18n;
I18n.init();
