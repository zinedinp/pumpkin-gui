import QtQuick
import QtQuick.Controls
import org.pumpkin.gui

TabButton {
    id: tab

    implicitHeight: 34

    contentItem: Text {
        text: tab.text
        color: tab.checked ? Theme.fg : Theme.fgMuted
        font.pixelSize: 13
        font.bold: tab.checked
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
    }

    background: Rectangle {
        color: tab.checked ? Theme.background : tab.hovered ? Theme.surfaceAlt : Theme.surface

        // The active tab is marked by an accent rule rather than a filled block, so the bar stays
        // quiet next to the content below it.
        Rectangle {
            anchors.bottom: parent.bottom
            width: parent.width
            height: tab.checked ? 2 : 0
            color: Theme.accent
        }
    }
}
