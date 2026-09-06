import QtQuick
import QtQuick.Layouts
import org.pumpkin.gui

Rectangle {
    id: root

    required property SetupController controller

    color: Theme.background

    Card {
        anchors.centerIn: parent
        width: Math.min(460, parent.width - 2 * Theme.gap)
        title: root.controller.headline

        ColumnLayout {
            width: parent.width
            spacing: Theme.gap

            Text {
                text: root.controller.failed
                    ? root.controller.statusMessage
                    : qsTr("No server was found next to pumpkin-gui. Choose the server executable, or connect to one that's already running.")
                color: root.controller.failed ? Theme.danger : Theme.fgMuted
                font.pixelSize: root.controller.failed ? 13 : 12
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }

            ThemedButton {
                text: qsTr("Choose server executable…")
                accent: Theme.accent
                Layout.fillWidth: true
                onClicked: root.controller.browseForBinary()
            }

            Rectangle {
                Layout.fillWidth: true
                height: 1
                color: Theme.rule
            }

            Text {
                text: qsTr("Or attach to a running server")
                color: Theme.fg
                font.pixelSize: 12
                font.bold: true
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 8

                ThemedField {
                    id: endpointField
                    Layout.fillWidth: true
                    placeholderText: qsTr("Socket path or pipe name")
                }

                ThemedButton {
                    text: qsTr("Attach")
                    onClicked: {
                        root.controller.useAttachEndpoint(endpointField.text);
                    }
                }
            }

            Text {
                text: root.controller.statusMessage
                visible: !root.controller.failed && text !== ""
                color: Theme.fgMuted
                font.pixelSize: 12
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }
    }
}
