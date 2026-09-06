import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.pumpkin.gui

ApplicationWindow {
    id: root

    width: 1180
    height: 780
    minimumWidth: 760
    minimumHeight: 520
    visible: true
    title: qsTr("Pumpkin")
    color: Theme.background

    ServerStats {
        id: stats
        Component.onCompleted: Theme.preference = themePreference
    }

    PlayerList {
        id: playerList
    }

    Console {
        id: consoleController
    }

    DevTools {
        id: dev

        Component.onCompleted: Theme.animations = screenshotPath === ""
    }

    function stopAndClose() {
        consoleController.requestStop();
        root.close();
    }

    onClosing: {
        if (dev.screenshotPath === "")
            consoleController.requestStop();
    }

    // Headless capture for development and CI: render, save a PNG, exit. Off unless
    // PUMPKIN_GUI_SCREENSHOT is set. Grabs `shell`, not the window's contentItem: the latter is
    // created from C++ and has no QML engine to grab with.
    Timer {
        running: dev.screenshotPath !== ""
        interval: dev.screenshotDelayMs
        onTriggered: {
            const grabbed = shell.grabToImage(function (result) {
                result.saveToFile(dev.screenshotPath);
                Qt.quit();
            });
            if (!grabbed)
                Qt.quit();
        }
    }

    // Polling instead of pushing: nothing has to cross a thread boundary into a Qt object.
    Timer {
        interval: 500
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: {
            stats.refresh();
            playerList.refresh();
            consoleController.refresh();
        }
    }

    // Ctrl+C / `stop` shut the server down on another thread
    Timer {
        interval: 50
        running: true
        repeat: true
        onTriggered: {
            consoleController.refresh();
            if (consoleController.stopping)
                Qt.quit();
        }
    }

    // A real background rather than relying on the window's own fill: `grabToImage` captures
    // only this item, and anything the item does not paint comes out transparent.
    Rectangle {
        id: shell

        anchors.fill: parent
        color: Theme.background

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            // The header lives inside the layout rather than in ApplicationWindow.header so the whole
            // UI is one QML item tree, which is what makes the headless grab above possible.
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 52
                color: Theme.surface

                Rectangle {
                    anchors.bottom: parent.bottom
                    width: parent.width
                    height: 1
                    color: Theme.rule
                }

                Row {
                    id: headerStatus
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.leftMargin: 16
                    anchors.right: stopButton.left
                    anchors.rightMargin: 12
                    height: parent.height
                    spacing: 16
                    clip: true
                    z: 0

                    Text {
                        text: stats.serverReady ? qsTr("Pumpkin %1").arg(stats.pumpkinVersion) : qsTr("Starting…")
                        color: Theme.fg
                        font.pixelSize: 15
                        font.bold: true
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Badge {
                        label: qsTr("TPS")
                        value: stats.tps.toFixed(1)
                        accent: Theme.tpsColor(stats.tps)
                        visible: stats.serverReady
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Badge {
                        label: qsTr("PLAYERS")
                        value: stats.playerCount
                        accent: Theme.accent
                        visible: stats.serverReady
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Badge {
                        label: qsTr("UPTIME")
                        value: Format.duration(stats.uptimeSecs)
                        accent: Theme.fgMuted
                        visible: stats.serverReady
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }

                ThemedButton {
                    id: stopButton
                    anchors.horizontalCenter: parent.horizontalCenter
                    anchors.verticalCenter: parent.verticalCenter
                    z: 1
                    text: qsTr("Stop Server")
                    accent: Theme.danger
                    enabled: consoleController.hasCommands
                    onClicked: root.stopAndClose()
                }

                ToolButton {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.rightMargin: 12
                    z: 1
                    // The platform style's own palette can stay dark (or light) regardless of
                    // `Theme.dark`, so the glyph needs an explicit colour rather than the
                    // ToolButton default.
                    contentItem: Text {
                        text: Theme.dark ? "☀" : "☾"
                        color: Theme.fg
                        font.pixelSize: 16
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                    onClicked: Theme.toggle()
                    ToolTip.visible: hovered
                    ToolTip.text: Theme.dark ? qsTr("Light theme") : qsTr("Dark theme")
                }
            }

            TabBar {
                id: tabs
                Layout.fillWidth: true
                currentIndex: dev.initialTab

                background: Rectangle {
                    color: Theme.surface

                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: 1
                        color: Theme.rule
                    }
                }

                ThemedTabButton {
                    text: qsTr("Overview")
                }
                ThemedTabButton {
                    text: qsTr("Performance")
                }
                ThemedTabButton {
                    text: qsTr("Worlds")
                }
                ThemedTabButton {
                    text: qsTr("Players")
                }
            }

            StackLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.margins: Theme.gap
                currentIndex: tabs.currentIndex

                Overview {
                    stats: stats
                    consoleController: consoleController
                }
                Performance {
                    stats: stats
                }
                Worlds {
                    stats: stats
                }
                Players {
                    controller: playerList
                }
            }
        }
    }
}
