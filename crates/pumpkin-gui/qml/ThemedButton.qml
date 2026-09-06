import QtQuick
import QtQuick.Controls
import org.pumpkin.gui

Button {
    id: button

    property color accent: Theme.fg

    implicitHeight: Theme.controlHeight
    leftPadding: 14
    rightPadding: 14

    contentItem: Text {
        text: button.text
        color: button.enabled ? button.accent : Theme.fgMuted
        font.pixelSize: 12
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
    }

    background: Rectangle {
        color: !button.enabled ? Theme.surface : button.down ? Theme.border : button.hovered ? Theme.surfaceAlt : Theme.surface
        border.color: Theme.border
        border.width: 1
        radius: 4
    }
}
