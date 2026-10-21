/**
 * Erkennt die Plattform, bevor Styles und Texte angewendet werden.
 * Unter Linux bekommt <html> die Klasse "platform-linux" (Nebel-Theme).
 */
(() => {
    const userAgent = navigator.userAgent || '';
    const platform = /Linux/i.test(userAgent) && !/Android/i.test(userAgent)
        ? 'linux'
        : (/Mac/i.test(userAgent) ? 'macos' : 'other');
    window.WeckPilotPlatform = platform;
    document.documentElement.classList.add(`platform-${platform}`);
})();
