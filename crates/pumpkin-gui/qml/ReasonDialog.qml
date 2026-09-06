import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.pumpkin.gui

// Confirms a kick or ban and collects an optional reason.
//
// Deliberately a confirmation step rather than a one-click action: both are disruptive and easy
// to trigger by accident on a hovered row.
Dialog {
    id: dialog

    property string action: ""
    property string playerName: ""

    signal confirmed(string action, string playerName, string reason)

    function open(nextAction, nextPlayer) {
        action = nextAction;
        playerName = nextPlayer;
        reasonField.clear();
        visible = true;
        reasonField.forceActiveFocus();
    }

    title: action === "ban" ? qsTr("Ban %1").arg(playerName) : qsTr("Kick %1").arg(playerName)

    modal: true
    anchors.centerIn: Overlay.overlay
    width: 380
    standardButtons: Dialog.Ok | Dialog.Cancel

    background: Rectangle {
        color: Theme.surface
        border.color: Theme.border
        border.width: 1
        radius: Theme.radius
    }

    onAccepted: dialog.confirmed(action, playerName, reasonField.text)

    ColumnLayout {
        width: parent.width
        spacing: 8

        Text {
            text: dialog.action === "ban" ? qsTr("The player is disconnected and added to the ban list.") : qsTr("The player is disconnected and may rejoin.")
            color: Theme.fgMuted
            font.pixelSize: 12
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }

        ThemedField {
            id: reasonField
            Layout.fillWidth: true
            placeholderText: qsTr("Reason (optional)")
            onAccepted: dialog.accept()
        }
    }
}
