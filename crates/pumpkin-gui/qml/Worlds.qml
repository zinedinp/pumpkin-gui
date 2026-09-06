import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.pumpkin.gui

Item {
    id: page

    required property var stats

    readonly property var worlds: stats.worlds

    // Same ListModel sync as the player table: world time ticks every sample, and rebuilding
    // the delegates would throw away scroll position.
    ListModel {
        id: rows
    }

    function sync() {
        const source = page.worlds;

        for (let i = 0; i < source.length; ++i) {
            const entry = source[i];
            if (i < rows.count)
                rows.set(i, entry);
            else
                rows.append(entry);
        }

        if (rows.count > source.length)
            rows.remove(source.length, rows.count - source.length);
    }

    function weatherLabel(weather) {
        switch (weather) {
        case "rain":
            return qsTr("Rain");
        case "thunder":
            return qsTr("Thunder");
        case "clear":
            return qsTr("Clear");
        case "none":
            return "–";
        default:
            return weather;
        }
    }

    Connections {
        target: page.stats
        function onWorldsChanged() {
            page.sync();
        }
    }

    Component.onCompleted: sync()

    Card {
        anchors.fill: parent

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

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
                    Layout.preferredWidth: 140
                    Layout.leftMargin: Theme.tableEdge
                    Layout.rightMargin: Theme.tableCellPad
                }
                TableVRule {}
                Text {
                    text: qsTr("DIMENSION")
                    color: Theme.accent
                    font.pixelSize: Theme.tableHeaderSize
                    font.bold: true
                    Layout.preferredWidth: 160
                    Layout.leftMargin: Theme.tableCellPad
                    Layout.rightMargin: Theme.tableCellPad
                }
                TableVRule {}
                Text {
                    text: qsTr("PLAYERS")
                    color: Theme.accent
                    font.pixelSize: Theme.tableHeaderSize
                    font.bold: true
                    horizontalAlignment: Text.AlignRight
                    Layout.preferredWidth: 70
                    Layout.leftMargin: Theme.tableCellPad
                    Layout.rightMargin: Theme.tableCellPad
                }
                TableVRule {}
                Text {
                    text: qsTr("LOADED CHUNKS")
                    color: Theme.accent
                    font.pixelSize: Theme.tableHeaderSize
                    font.bold: true
                    horizontalAlignment: Text.AlignRight
                    Layout.preferredWidth: 110
                    Layout.leftMargin: Theme.tableCellPad
                    Layout.rightMargin: Theme.tableCellPad
                }
                TableVRule {}
                Text {
                    text: qsTr("ENTITIES")
                    color: Theme.accent
                    font.pixelSize: Theme.tableHeaderSize
                    font.bold: true
                    horizontalAlignment: Text.AlignRight
                    Layout.preferredWidth: 70
                    Layout.leftMargin: Theme.tableCellPad
                    Layout.rightMargin: Theme.tableCellPad
                }
                TableVRule {}
                Text {
                    text: qsTr("TIME")
                    color: Theme.accent
                    font.pixelSize: Theme.tableHeaderSize
                    font.bold: true
                    Layout.preferredWidth: 110
                    Layout.leftMargin: Theme.tableCellPad
                    Layout.rightMargin: Theme.tableCellPad
                }
                TableVRule {}
                Text {
                    text: qsTr("WEATHER")
                    color: Theme.accent
                    font.pixelSize: Theme.tableHeaderSize
                    font.bold: true
                    Layout.preferredWidth: 80
                    Layout.leftMargin: Theme.tableCellPad
                    Layout.rightMargin: Theme.tableCellPad
                }
                TableVRule {}
                Item {
                    Layout.fillWidth: true
                }
                Text {
                    text: qsTr("SIZE")
                    color: Theme.accent
                    font.pixelSize: Theme.tableHeaderSize
                    font.bold: true
                    horizontalAlignment: Text.AlignRight
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

                delegate: Rectangle {
                    required property string name
                    required property string dimension
                    required property int players
                    required property int chunks
                    required property int entities
                    required property real timeOfDay
                    required property string weather
                    required property real size
                    required property int index

                    width: ListView.view.width
                    height: 34
                    color: hover.hovered ? Theme.rowHover : (index % 2 === 0 ? Theme.rowEven : Theme.rowOdd)

                    HoverHandler {
                        id: hover
                    }

                    RowLayout {
                        anchors.fill: parent
                        spacing: 0

                        Text {
                            text: name
                            color: Theme.fg
                            font.pixelSize: Theme.tableCellSize
                            elide: Text.ElideRight
                            verticalAlignment: Text.AlignVCenter
                            Layout.fillHeight: true
                            Layout.preferredWidth: 140
                            Layout.leftMargin: Theme.tableEdge
                            Layout.rightMargin: Theme.tableCellPad
                        }
                        TableVRule {}
                        Text {
                            text: dimension.replace("minecraft:", "")
                            color: Theme.fgMuted
                            font.pixelSize: Theme.tableCellSize
                            elide: Text.ElideRight
                            verticalAlignment: Text.AlignVCenter
                            Layout.fillHeight: true
                            Layout.preferredWidth: 160
                            Layout.leftMargin: Theme.tableCellPad
                            Layout.rightMargin: Theme.tableCellPad
                        }
                        TableVRule {}
                        Text {
                            text: players
                            color: Theme.fg
                            font.pixelSize: Theme.tableCellSize
                            font.family: "monospace"
                            horizontalAlignment: Text.AlignRight
                            verticalAlignment: Text.AlignVCenter
                            Layout.fillHeight: true
                            Layout.preferredWidth: 70
                            Layout.leftMargin: Theme.tableCellPad
                            Layout.rightMargin: Theme.tableCellPad
                        }
                        TableVRule {}
                        Text {
                            text: chunks
                            color: Theme.fg
                            font.pixelSize: Theme.tableCellSize
                            font.family: "monospace"
                            horizontalAlignment: Text.AlignRight
                            verticalAlignment: Text.AlignVCenter
                            Layout.fillHeight: true
                            Layout.preferredWidth: 110
                            Layout.leftMargin: Theme.tableCellPad
                            Layout.rightMargin: Theme.tableCellPad
                        }
                        TableVRule {}
                        Text {
                            text: entities
                            color: Theme.fg
                            font.pixelSize: Theme.tableCellSize
                            font.family: "monospace"
                            horizontalAlignment: Text.AlignRight
                            verticalAlignment: Text.AlignVCenter
                            Layout.fillHeight: true
                            Layout.preferredWidth: 70
                            Layout.leftMargin: Theme.tableCellPad
                            Layout.rightMargin: Theme.tableCellPad
                        }
                        TableVRule {}
                        Text {
                            text: Format.gameTime(timeOfDay)
                            color: Theme.fgMuted
                            font.pixelSize: Theme.tableCellSize
                            font.family: "monospace"
                            elide: Text.ElideRight
                            verticalAlignment: Text.AlignVCenter
                            Layout.fillHeight: true
                            Layout.preferredWidth: 110
                            Layout.leftMargin: Theme.tableCellPad
                            Layout.rightMargin: Theme.tableCellPad
                        }
                        TableVRule {}
                        Text {
                            text: page.weatherLabel(weather)
                            color: Theme.fgMuted
                            font.pixelSize: Theme.tableCellSize
                            verticalAlignment: Text.AlignVCenter
                            Layout.fillHeight: true
                            Layout.preferredWidth: 80
                            Layout.leftMargin: Theme.tableCellPad
                            Layout.rightMargin: Theme.tableCellPad
                        }
                        TableVRule {}
                        Item {
                            Layout.fillWidth: true
                        }
                        Text {
                            text: Format.bytes(size)
                            color: Theme.fg
                            font.pixelSize: Theme.tableCellSize
                            font.family: "monospace"
                            horizontalAlignment: Text.AlignRight
                            verticalAlignment: Text.AlignVCenter
                            Layout.fillHeight: true
                            Layout.leftMargin: Theme.tableCellPad
                            Layout.rightMargin: Theme.tableEdge
                        }
                    }

                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: 1
                        color: Theme.rule
                    }
                }
            }

            Text {
                Layout.fillWidth: true
                Layout.topMargin: 12
                visible: rows.count === 0
                text: qsTr("No worlds loaded.")
                color: Theme.fgMuted
                font.pixelSize: 12
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}
