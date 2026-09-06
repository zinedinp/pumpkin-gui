import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.pumpkin.gui

Item {
    id: page

    // The `Players` QObject: supplies the rows and receives the actions.
    required property var controller

    readonly property var players: controller.rows

    // Binding a ListView straight to the Rust list would rebuild every delegate whenever any
    // field changes -- ping alone moves constantly -- throwing away scroll position and hover
    // state. Syncing into a ListModel row by row keeps the delegates alive.
    ListModel {
        id: rows
    }

    property string filter: ""
    readonly property var viewKeys: ["online", "offline", "operator", "banned", "whitelisted"]
    property string view: "online"

    ListModel {
        id: dimCounts
    }

    function inView(entry) {
        switch (page.view) {
        case "offline":
            return !entry.isOnline;
        case "operator":
            return entry.operator;
        case "banned":
            return entry.banned;
        case "whitelisted":
            return entry.whitelisted;
        default:
            return entry.isOnline;
        }
    }

    function matches(entry) {
        if (page.filter !== "" && !entry.name.toLowerCase().includes(page.filter.toLowerCase()) && !entry.uuid.toLowerCase().includes(page.filter.toLowerCase()))
            return false;
        return page.inView(entry);
    }

    function rebuildDims() {
        const counts = {};
        const source = page.players.filter(page.inView);
        for (let i = 0; i < source.length; ++i) {
            const dim = String(source[i].dimension || "").replace("minecraft:", "");
            if (dim === "")
                continue;
            counts[dim] = (counts[dim] || 0) + 1;
        }

        const keys = Object.keys(counts).sort();
        for (let i = 0; i < keys.length; ++i) {
            const entry = {
                "chipLabel": keys[i],
                "chipCount": counts[keys[i]]
            };
            if (i < dimCounts.count)
                dimCounts.set(i, entry);
            else
                dimCounts.append(entry);
        }
        if (dimCounts.count > keys.length)
            dimCounts.remove(keys.length, dimCounts.count - keys.length);
    }

    function sync() {
        const source = page.players.filter(page.matches);

        for (let i = 0; i < source.length; ++i) {
            const entry = source[i];
            if (i < rows.count) {
                // set() only emits changes for the keys that actually differ.
                rows.set(i, entry);
            } else {
                rows.append(entry);
            }
        }

        if (rows.count > source.length)
            rows.remove(source.length, rows.count - source.length);

        page.rebuildDims();
    }

    function addWhitelist() {
        const name = whitelistField.text.trim();
        if (name === "")
            return;
        page.controller.whitelist(name);
        whitelistField.clear();
    }

    Connections {
        target: page.players !== undefined ? page : null
        function onPlayersChanged() {
            page.sync();
        }
        function onFilterChanged() {
            page.sync();
        }
        function onViewChanged() {
            page.sync();
        }
    }

    Component.onCompleted: sync()

    ReasonDialog {
        id: reasonDialog
        onConfirmed: (action, playerName, reason) => {
            if (action === "ban")
                page.controller.ban(playerName, reason);
            else
                page.controller.kick(playerName, reason);
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: Theme.gap

        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            ThemedComboBox {
                id: viewBox
                Layout.preferredHeight: Theme.controlHeight
                Layout.preferredWidth: 150
                model: [qsTr("Online"), qsTr("Offline"), qsTr("Operator"), qsTr("Banned"), qsTr("Whitelisted")]
                onCurrentIndexChanged: page.view = page.viewKeys[currentIndex]
            }

            ThemedField {
                id: search
                Layout.preferredHeight: Theme.controlHeight
                Layout.preferredWidth: 220
                placeholderText: qsTr("Search players…")
                onTextChanged: page.filter = text
            }

            Repeater {
                model: dimCounts

                delegate: CountChip {
                    required property string chipLabel
                    required property int chipCount
                    label: chipLabel
                    count: chipCount
                }
            }

            Item {
                Layout.fillWidth: true
            }

            ThemedField {
                id: whitelistField
                Layout.preferredHeight: Theme.controlHeight
                Layout.preferredWidth: 220
                enabled: page.controller.hasCommands
                placeholderText: qsTr("Player name…")
                onAccepted: page.addWhitelist()
            }

            ThemedButton {
                text: qsTr("Whitelist")
                accent: Theme.accent
                Layout.preferredHeight: Theme.controlHeight
                enabled: page.controller.hasCommands && whitelistField.text.trim() !== ""
                onClicked: page.addWhitelist()
            }
        }

        Card {
            Layout.fillWidth: true
            Layout.fillHeight: true

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                // Header row
                RowLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: false
                    Layout.bottomMargin: 6
                    spacing: 0

                    Text {
                        text: qsTr("NAME")
                        color: Theme.accent
                        font.pixelSize: Theme.tableHeaderSize
                        font.bold: true
                        Layout.preferredWidth: 186
                        Layout.leftMargin: Theme.tableEdge
                        Layout.rightMargin: Theme.tableCellPad
                    }
                    TableVRule {}
                    Text {
                        text: qsTr("EDITION")
                        color: Theme.accent
                        font.pixelSize: Theme.tableHeaderSize
                        font.bold: true
                        Layout.preferredWidth: 78
                        Layout.leftMargin: Theme.tableCellPad
                        Layout.rightMargin: Theme.tableCellPad
                    }
                    TableVRule {}
                    Text {
                        text: qsTr("UUID")
                        color: Theme.accent
                        font.pixelSize: Theme.tableHeaderSize
                        font.bold: true
                        Layout.preferredWidth: 276
                        Layout.leftMargin: Theme.tableCellPad
                        Layout.rightMargin: Theme.tableCellPad
                    }
                    TableVRule {}
                    Text {
                        text: qsTr("PING")
                        color: Theme.accent
                        font.pixelSize: Theme.tableHeaderSize
                        font.bold: true
                        horizontalAlignment: Text.AlignRight
                        Layout.preferredWidth: 60
                        Layout.leftMargin: Theme.tableCellPad
                        Layout.rightMargin: Theme.tableCellPad
                    }
                    TableVRule {}
                    Text {
                        text: qsTr("DIMENSION")
                        color: Theme.accent
                        font.pixelSize: Theme.tableHeaderSize
                        font.bold: true
                        Layout.preferredWidth: 120
                        Layout.leftMargin: Theme.tableCellPad
                        Layout.rightMargin: Theme.tableCellPad
                    }
                    TableVRule {}
                    Text {
                        text: qsTr("MODE")
                        color: Theme.accent
                        font.pixelSize: Theme.tableHeaderSize
                        font.bold: true
                        Layout.preferredWidth: 80
                        Layout.leftMargin: Theme.tableCellPad
                        Layout.rightMargin: Theme.tableCellPad
                    }
                    TableVRule {}
                    Text {
                        text: qsTr("ONLINE")
                        color: Theme.accent
                        font.pixelSize: Theme.tableHeaderSize
                        font.bold: true
                        Layout.preferredWidth: 70
                        Layout.leftMargin: Theme.tableCellPad
                        Layout.rightMargin: Theme.tableCellPad
                    }
                    TableVRule {}
                    Item {
                        Layout.fillWidth: true
                    }
                    Text {
                        text: qsTr("ACTIONS")
                        color: Theme.accent
                        font.pixelSize: Theme.tableHeaderSize
                        font.bold: true
                        Layout.leftMargin: Theme.tableCellPad
                        Layout.rightMargin: Theme.tableEdge
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: Theme.rule
                }

                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: rows
                    boundsBehavior: Flickable.StopAtBounds

                    ScrollBar.vertical: ScrollBar {}

                    delegate: PlayerRowDelegate {
                        width: ListView.view.width
                        controller: page.controller
                        view: page.view
                        onReasonRequested: (action, playerName) => reasonDialog.open(action, playerName)
                    }
                }

                Text {
                    Layout.fillWidth: true
                    Layout.topMargin: 12
                    visible: rows.count === 0
                    text: {
                        if (page.filter !== "")
                            return qsTr("No player matches “%1”.").arg(page.filter);
                        switch (page.view) {
                        case "offline":
                            return qsTr("No offline players.");
                        case "operator":
                            return qsTr("No operators.");
                        case "banned":
                            return qsTr("No banned players.");
                        case "whitelisted":
                            return qsTr("Whitelist is empty.");
                        default:
                            return qsTr("No players online.");
                        }
                    }
                    color: Theme.fgMuted
                    font.pixelSize: 12
                    horizontalAlignment: Text.AlignHCenter
                }
            }
        }
    }
}
