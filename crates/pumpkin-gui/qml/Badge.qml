import QtQuick
import QtQuick.Layouts
import org.pumpkin.gui

// A compact label/value pair for the header bar.
ColumnLayout {
    id: badge

    property string label: ""
    property var value: ""
    property color accent: Theme.fg

    spacing: 0

    Text {
        text: badge.value
        color: badge.accent
        font.pixelSize: 15
        font.bold: true
        font.family: "monospace"
    }

    Text {
        text: badge.label
        color: Theme.fgMuted
        font.pixelSize: 9
        font.letterSpacing: 0.6
    }
}
