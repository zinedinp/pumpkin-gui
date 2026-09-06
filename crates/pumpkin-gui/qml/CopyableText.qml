import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.pumpkin.gui

// A monospace value with a copy button, for things you actually need on the clipboard.
//
// Qt Quick exposes no clipboard type to QML and cxx-qt-lib has no QClipboard binding, so the copy
// goes through an off-screen TextEdit -- `copy()` works on the document, not on what is painted.
RowLayout {
    id: field

    property string value: ""
    property string label: ""
    // Table cells: value and button on one line, no caption, left-aligned.
    property bool compact: false
    property bool mono: true

    spacing: 6

    Text {
        visible: field.compact
        text: field.value
        color: Theme.fg
        font.pixelSize: Theme.tableCellSize
        font.family: field.mono ? "monospace" : "sans-serif"
        elide: Text.ElideMiddle
        verticalAlignment: Text.AlignVCenter
        Layout.fillWidth: field.compact
        Layout.preferredWidth: field.compact ? 1 : 0
        Layout.alignment: Qt.AlignVCenter
    }

    ColumnLayout {
        visible: !field.compact
        spacing: 0
        Layout.alignment: Qt.AlignRight
        Layout.preferredWidth: field.compact ? 0 : implicitWidth
        Layout.maximumWidth: field.compact ? 0 : implicitWidth

        Text {
            text: field.value
            color: Theme.fg
            font.pixelSize: 13
            font.family: field.mono ? "monospace" : "sans-serif"
            horizontalAlignment: Text.AlignRight
            Layout.alignment: Qt.AlignRight
        }

        Text {
            text: field.label
            visible: field.label !== ""
            color: Theme.fgMuted
            font.pixelSize: 11
            horizontalAlignment: Text.AlignRight
            Layout.alignment: Qt.AlignRight
        }
    }

    TextEdit {
        id: clipboardHelper
        visible: false
        text: field.value
    }

    IconButton {
        source: Icons.copy
        tint: copiedHint.visible ? Theme.good : Theme.fg
        tooltip: qsTr("Copy")
        Layout.alignment: Qt.AlignVCenter
        Layout.preferredWidth: implicitWidth
        Layout.preferredHeight: implicitHeight
        Layout.minimumWidth: implicitWidth
        Layout.minimumHeight: implicitHeight

        onClicked: {
            clipboardHelper.selectAll();
            clipboardHelper.copy();
            clipboardHelper.deselect();
            copiedHint.show();
        }
    }

    Timer {
        id: copiedHint

        interval: 1200

        function show() {
            visible = true;
            restart();
        }

        property bool visible: false
        onTriggered: visible = false
    }
}
