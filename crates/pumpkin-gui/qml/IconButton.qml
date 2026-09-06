import QtQuick
import QtQuick.Controls
import org.pumpkin.gui

// A flat, icon-only button.
//
// `icon.color` is what makes the monochrome SVGs follow the theme: the files pin `currentColor`
// to a fixed grey, which Qt's SVG Tiny renderer does not resolve, so Qt Quick Controls recolours
// the rendered image instead.
ToolButton {
    id: button

    property url source
    property color tint: Theme.fg
    property string tooltip: ""

    icon.source: source
    icon.color: enabled ? tint : Theme.fgMuted
    icon.width: 22
    icon.height: 22
    display: AbstractButton.IconOnly
    implicitWidth: 36
    implicitHeight: 36
    opacity: enabled ? 1 : 0.45

    background: Rectangle {
        radius: 6
        color: {
            if (!button.enabled)
                return "transparent";
            const rest = Theme.dark ? 0.22 : 0.18;
            const hover = Theme.dark ? 0.34 : 0.28;
            const down = Theme.dark ? 0.46 : 0.38;
            return Theme.withAlpha(button.tint, button.down ? down : button.hovered ? hover : rest);
        }
        border.color: button.enabled ? Theme.withAlpha(button.tint, Theme.dark ? 0.7 : 0.65) : Theme.border
        border.width: 1
    }

    ToolTip.visible: hovered && tooltip !== ""
    ToolTip.text: tooltip
    ToolTip.delay: 400
}
