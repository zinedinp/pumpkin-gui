import QtQuick
import QtQuick.Layouts
import org.pumpkin.gui

// One headline number with a caption and an optional fill bar.
Rectangle {
    id: tile

    property string label: ""
    property var value: ""
    property string caption: ""
    property color accent: Theme.fg
    // Negative hides the bar; tiles like TPS have no meaningful 0..1 scale.
    property real fraction: -1

    color: Theme.surface
    border.color: Theme.border
    border.width: 1
    radius: Theme.radius
    implicitHeight: column.implicitHeight + 2 * Theme.gap

    ColumnLayout {
        id: column

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: Theme.gap
        spacing: 2

        Text {
            text: tile.label
            color: Theme.accent
            font.pixelSize: Theme.tableHeaderSize + 2
            font.bold: true
            font.letterSpacing: 0.8
        }

        Text {
            text: tile.value
            color: tile.accent
            font.pixelSize: 22
            font.bold: true
            elide: Text.ElideRight
            Layout.fillWidth: true
        }

        Rectangle {
            visible: tile.fraction >= 0
            Layout.fillWidth: true
            Layout.preferredHeight: 4
            Layout.topMargin: 2
            radius: 2
            color: Theme.surfaceAlt

            Rectangle {
                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                width: parent.width * Math.max(0, Math.min(1, tile.fraction))
                radius: parent.radius
                color: Theme.loadColor(tile.fraction)
            }
        }

        Text {
            text: tile.caption
            color: Theme.fgMuted
            font.pixelSize: 11
            elide: Text.ElideRight
            Layout.fillWidth: true
        }
    }
}
