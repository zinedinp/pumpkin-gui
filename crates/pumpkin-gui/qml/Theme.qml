pragma Singleton

import QtQuick

// Every colour in the UI comes from here, so switching themes is one property change rather than
// a sweep through the components.
QtObject {
    id: theme

    // -1 means "follow the desktop"; Rust overrides it at startup from the `gui.theme` config.
    property int preference: -1

    // Turned off for headless capture: offscreen rendering does not advance the animation driver,
    // so animated bars would be caught mid-transition and screenshot comparisons would be flaky.
    property bool animations: true

    readonly property bool dark: preference === 1 || (preference === -1 && Application.styleHints.colorScheme === Qt.Dark)

    function toggle() {
        preference = dark ? 0 : 1;
    }

    // Surfaces
    readonly property color background: dark ? "#16181d" : "#f6f7f9"
    readonly property color surface: dark ? "#1e2128" : "#ffffff"
    readonly property color surfaceAlt: dark ? "#262a33" : "#eef0f4"
    readonly property color border: dark ? "#333844" : "#d8dce4"
    // Hairline between chrome segments (header / tabs / content) and table rows.
    readonly property color rule: dark ? "#4a5160" : "#b8c0cc"
    readonly property color rowEven: dark ? "#1e2128" : "#ffffff"
    readonly property color rowOdd: dark ? "#252a33" : "#eef1f5"
    readonly property color rowHover: dark ? "#30363f" : "#e2e7ee"

    // Text
    readonly property color fg: dark ? "#ffffff" : "#000000"
    readonly property color fgMuted: dark ? "#9aa1ae" : "#5c6472"

    // Accents. `accent` is Pumpkin's orange in both themes, lightened for dark backgrounds.
    readonly property color accent: dark ? "#ff9d4d" : "#e0701a"
    readonly property color good: dark ? "#5ac37f" : "#2f8f52"
    readonly property color warn: dark ? "#e8c05a" : "#a97a10"
    readonly property color danger: dark ? "#e8705f" : "#c2412e"

    readonly property int radius: 8
    readonly property int gap: 12
    // Inset for the first and last table cells so labels are not aligned with the card edge.
    readonly property int tableEdge: 12
    // Horizontal padding on either side of a column
    readonly property int tableCellPad: 8
    readonly property int tableHeaderSize: 12
    readonly property int tableCellSize: 13
    // Shared control height so the player search field and the whitelist row line up.
    readonly property int controlHeight: 32

    function withAlpha(c, alpha) {
        return Qt.rgba(c.r, c.g, c.b, alpha);
    }

    // Shared thresholds so the TPS badge, the tick graph and the core bars agree on what
    // "healthy" looks like.
    function loadColor(fraction) {
        if (fraction >= 0.9)
            return danger;
        if (fraction >= 0.6)
            return warn;
        return good;
    }

    // TPS is healthy at 20 and bad as it drops, so the scale runs the other way.
    function tpsColor(tps) {
        if (tps >= 19.0)
            return good;
        if (tps >= 15.0)
            return warn;
        return danger;
    }
}
