import QtQuick
import QtQuick.Controls
import org.pumpkin.gui

ComboBox {
    id: combo

    implicitHeight: Theme.controlHeight

    contentItem: Text {
        text: combo.displayText
        color: Theme.fg
        font.pixelSize: 12
        verticalAlignment: Text.AlignVCenter
        leftPadding: 8
        rightPadding: combo.indicator.width
        elide: Text.ElideRight
    }

    indicator: Text {
        x: combo.width - width - 8
        y: combo.height / 2 - height / 2
        text: "▾"
        color: Theme.fgMuted
        font.pixelSize: 12
    }

    background: Rectangle {
        color: Theme.background
        border.color: combo.activeFocus ? Theme.accent : Theme.border
        border.width: 1
        radius: 4
    }

    delegate: ItemDelegate {
        id: option

        required property int index
        required property var modelData

        width: combo.width
        highlighted: combo.highlightedIndex === index

        contentItem: Text {
            text: option.modelData
            color: Theme.fg
            font.pixelSize: 12
            verticalAlignment: Text.AlignVCenter
        }

        background: Rectangle {
            color: option.highlighted ? Theme.surfaceAlt : Theme.surface
        }
    }

    popup: Popup {
        y: combo.height
        width: combo.width
        implicitHeight: contentItem.implicitHeight
        padding: 1

        contentItem: ListView {
            clip: true
            implicitHeight: contentHeight
            model: combo.popup.visible ? combo.delegateModel : null
            currentIndex: combo.highlightedIndex
        }

        background: Rectangle {
            color: Theme.surface
            border.color: Theme.border
            border.width: 1
            radius: 4
        }
    }
}
