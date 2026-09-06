import QtQuick
import org.pumpkin.gui

// Compact Java / Bedrock pill for the player table.
Rectangle {
    id: tag

    property string edition: ""

    readonly property bool known: edition === "java" || edition === "bedrock"
    readonly property color accent: edition === "bedrock" ? Theme.good : Theme.accent

    visible: true
    implicitHeight: 20
    implicitWidth: label.implicitWidth + 12
    radius: 4
    color: known ? Theme.withAlpha(accent, Theme.dark ? 0.22 : 0.16) : Theme.surfaceAlt
    border.color: known ? Theme.withAlpha(accent, Theme.dark ? 0.7 : 0.55) : Theme.border
    border.width: 1

    Text {
        id: label
        anchors.centerIn: parent
        text: {
            if (tag.edition === "bedrock")
                return qsTr("Bedrock");
            if (tag.edition === "java")
                return qsTr("Java");
            return "–";
        }
        color: tag.known ? tag.accent : Theme.fgMuted
        font.pixelSize: 10
        font.bold: true
    }
}
