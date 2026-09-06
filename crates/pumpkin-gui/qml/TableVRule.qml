import QtQuick
import QtQuick.Layouts
import org.pumpkin.gui

// Vertical slice between table columns.
Item {
    id: slot

    implicitWidth: 1
    implicitHeight: 1
    Layout.preferredWidth: 1
    Layout.minimumWidth: 1
    Layout.maximumWidth: 1
    Layout.alignment: Qt.AlignTop

    Rectangle {
        width: 1
        color: Theme.rule
        height: slot.parent ? slot.parent.height : 1
        y: slot.parent ? -slot.y : 0
    }
}
