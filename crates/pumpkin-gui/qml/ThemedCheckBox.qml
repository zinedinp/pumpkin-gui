import QtQuick
import QtQuick.Controls
import org.pumpkin.gui

CheckBox {
    id: box

    indicator: Rectangle {
        implicitWidth: 16
        implicitHeight: 16
        x: box.leftPadding
        y: box.height / 2 - height / 2
        radius: 3
        color: box.checked ? Theme.accent : Theme.background
        border.color: box.checked ? Theme.accent : Theme.border
        border.width: 1

        Text {
            anchors.centerIn: parent
            visible: box.checked
            text: "✓"
            color: Theme.surface
            font.pixelSize: 12
            font.bold: true
        }
    }

    contentItem: Text {
        text: box.text
        color: Theme.fg
        font.pixelSize: 12
        verticalAlignment: Text.AlignVCenter
        leftPadding: box.indicator.width + box.spacing
    }
}
