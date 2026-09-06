import QtQuick
import org.pumpkin.gui

// Stands in for a tab that is not built yet, so the shell can be navigated end to end.
Item {
    property string text: ""

    Text {
        anchors.centerIn: parent
        text: qsTr("%1 — not implemented yet").arg(parent.text)
        color: Theme.fgMuted
        font.pixelSize: 13
    }
}
