import QtQuick
import QtQuick.Layouts
import org.pumpkin.gui

// A horizontal fill bar with a caption underneath.
ColumnLayout {
    id: meter

    property real fraction: 0
    property string caption: ""

    spacing: 4

    Rectangle {
        Layout.fillWidth: true
        Layout.preferredHeight: 8
        radius: 4
        color: Theme.surfaceAlt

        Rectangle {
            width: parent.width * Math.max(0, Math.min(1, meter.fraction))
            height: parent.height
            radius: parent.radius
            color: Theme.loadColor(meter.fraction)

            Behavior on width {
                enabled: Theme.animations
                NumberAnimation {
                    duration: 220
                    easing.type: Easing.OutQuad
                }
            }
            Behavior on color {
                enabled: Theme.animations
                ColorAnimation {
                    duration: 220
                }
            }
        }
    }

    Text {
        text: meter.caption
        visible: meter.caption !== ""
        color: Theme.fgMuted
        font.pixelSize: 11
        elide: Text.ElideRight
        Layout.fillWidth: true
    }
}
