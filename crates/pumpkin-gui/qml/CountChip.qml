import QtQuick
import QtQuick.Layouts
import org.pumpkin.gui

// Compact "N label" pill used above the player and world tables.
Rectangle {
    id: chip

    property int count: 0
    property string label: ""

    implicitHeight: Theme.controlHeight
    implicitWidth: row.implicitWidth + 16
    radius: 6
    color: Theme.surfaceAlt
    border.color: Theme.border
    border.width: 1

    Row {
        id: row
        anchors.centerIn: parent
        spacing: 6

        Text {
            text: chip.count
            color: Theme.fg
            font.pixelSize: 13
            font.bold: true
            font.family: "monospace"
        }
        Text {
            text: chip.label
            color: Theme.fg
            font.pixelSize: 12
        }
    }
}
