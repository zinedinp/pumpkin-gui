import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.pumpkin.gui

// A yes/no dialog for actions that cannot be undone.
Dialog {
    id: dialog

    property string body: ""
    property string confirmText: qsTr("Confirm")

    signal confirmed

    modal: true
    anchors.centerIn: Overlay.overlay
    width: 400
    closePolicy: Popup.CloseOnEscape

    background: Rectangle {
        color: Theme.surface
        border.color: Theme.border
        border.width: 1
        radius: Theme.radius
    }

    header: Text {
        text: dialog.title
        color: Theme.fg
        font.pixelSize: 15
        font.bold: true
        padding: 16
    }

    contentItem: Text {
        text: dialog.body
        color: Theme.fgMuted
        font.pixelSize: 12
        wrapMode: Text.WordWrap
        leftPadding: 16
        rightPadding: 16
    }

    // Wrapped in an Item because a bare RowLayout has no padding of its own and would sit
    // flush against the dialog edges.
    footer: Item {
        implicitHeight: footerRow.implicitHeight + 32

        RowLayout {
            id: footerRow

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: 16
            spacing: 8

            Item {
                Layout.fillWidth: true
            }

            ThemedButton {
                text: qsTr("Cancel")
                onClicked: dialog.close()
            }

            ThemedButton {
                text: dialog.confirmText
                accent: Theme.danger
                onClicked: {
                    dialog.confirmed();
                    dialog.close();
                }
            }
        }
    }
}
