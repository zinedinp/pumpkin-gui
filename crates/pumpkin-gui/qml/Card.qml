import QtQuick
import QtQuick.Layouts
import org.pumpkin.gui

// A titled panel. Everything on the dashboard sits in one of these so spacing and borders stay
// consistent without each page repeating the same Rectangle.
Rectangle {
    id: card

    property string title: ""
    default property alias content: contentArea.data

    color: Theme.surface
    border.color: Theme.border
    border.width: 1
    radius: Theme.radius
    implicitHeight: layout.implicitHeight + 2 * Theme.gap

    ColumnLayout {
        id: layout
        anchors.fill: parent
        anchors.margins: Theme.gap
        spacing: 8

        Text {
            text: card.title
            visible: card.title !== ""
            color: Theme.accent
            font.pixelSize: Theme.tableHeaderSize + 2
            font.bold: true
            font.letterSpacing: 0.8
            Layout.fillWidth: true
        }

        Item {
            id: contentArea
            Layout.fillWidth: true
            Layout.fillHeight: true
            implicitHeight: childrenRect.height
        }
    }
}
