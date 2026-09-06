import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.pumpkin.gui

// Command line with history and tab completion, matching what rustyline offers in the terminal.
RowLayout {
    id: input

    required property var controller

    property var history: []
    // Index into `history`; equal to its length means "editing a new line".
    property int historyIndex: 0
    property string draft: ""

    spacing: 8

    function submit() {
        const text = field.text.trim();
        if (text === "")
            return;

        input.controller.submit(text);

        // Same rule as a shell: repeating the previous command does not add a duplicate entry.
        if (history.length === 0 || history[history.length - 1] !== text)
            history.push(text);
        historyIndex = history.length;
        draft = "";
        field.clear();
    }

    function recall(offset) {
        if (history.length === 0)
            return;

        // Stepping off the end restores whatever was being typed before the first recall.
        if (historyIndex === history.length)
            draft = field.text;

        const next = Math.max(0, Math.min(history.length, historyIndex + offset));
        historyIndex = next;
        field.text = next === history.length ? draft : history[next];
        field.cursorPosition = field.text.length;
    }

    function complete() {
        const candidates = input.controller.complete(field.text, field.cursorPosition);
        if (candidates.length === 0)
            return;

        if (candidates.length === 1) {
            // A single match completes the word in place.
            const head = field.text.slice(0, field.cursorPosition);
            const lastSpace = head.lastIndexOf(" ");
            field.text = head.slice(0, lastSpace + 1) + candidates[0] + field.text.slice(field.cursorPosition);
            field.cursorPosition = lastSpace + 1 + candidates[0].length;
        } else {
            hint.text = candidates.join("  ");
        }
    }

    Text {
        text: "$"
        color: Theme.accent
        font.family: "monospace"
        font.pixelSize: 13
        font.bold: true
    }

    ColumnLayout {
        Layout.fillWidth: true
        spacing: 2

        ThemedField {
            id: field

            Layout.fillWidth: true
            enabled: input.controller.hasCommands
            placeholderText: enabled ? qsTr("Type a command…") : qsTr("Waiting for the server…")
            font.family: "monospace"
            font.pixelSize: 13

            onAccepted: input.submit()
            onTextEdited: hint.text = ""

            Keys.onUpPressed: event => {
                event.accepted = true;
                input.recall(-1);
            }
            Keys.onDownPressed: event => {
                event.accepted = true;
                input.recall(1);
            }
            // Swallow Tab so focus does not also jump to the Run button.
            Keys.onTabPressed: event => {
                event.accepted = true;
                input.complete();
            }
        }

        Text {
            id: hint
            Layout.fillWidth: true
            visible: text !== ""
            color: Theme.fgMuted
            font.family: "monospace"
            font.pixelSize: 11
            elide: Text.ElideRight
        }
    }

    ThemedButton {
        text: qsTr("Run")
        enabled: input.controller.hasCommands && field.text.trim() !== ""
        onClicked: input.submit()
    }
}
