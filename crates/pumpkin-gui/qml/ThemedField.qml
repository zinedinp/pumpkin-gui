import QtQuick
import QtQuick.Controls
import org.pumpkin.gui

// Qt Quick Controls' Basic style paints a light background regardless of the theme, so every
// control the UI uses gets a themed wrapper instead of being restyled at each call site.
TextField {
    id: field

    color: Theme.fg
    placeholderTextColor: Theme.fgMuted
    selectionColor: Theme.accent
    selectedTextColor: Theme.surface
    font.pixelSize: 12
    implicitHeight: Theme.controlHeight
    leftPadding: 10
    rightPadding: 10

    background: Rectangle {
        color: Theme.background
        border.color: field.activeFocus ? Theme.accent : Theme.border
        border.width: 1
        radius: 4
    }
}
