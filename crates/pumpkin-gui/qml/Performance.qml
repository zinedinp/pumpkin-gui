import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.pumpkin.gui

ScrollView {
    id: page

    required property var stats

    contentWidth: availableWidth
    clip: true

    ColumnLayout {
        width: page.availableWidth
        spacing: Theme.gap

        // Tick health first: it is the metric that decides whether the server is actually well.
        Card {
            title: qsTr("TICK PERFORMANCE")
            Layout.fillWidth: true

            ColumnLayout {
                width: parent.width
                spacing: 8

                RowLayout {
                    spacing: 24

                    ColumnLayout {
                        spacing: 0
                        Text {
                            text: page.stats.tps.toFixed(2)
                            color: Theme.tpsColor(page.stats.tps)
                            font.pixelSize: 30
                            font.bold: true
                        }
                        Text {
                            text: qsTr("TPS")
                            color: Theme.fgMuted
                            font.pixelSize: 11
                        }
                    }

                    ColumnLayout {
                        spacing: 0
                        Text {
                            text: page.stats.mspt.toFixed(2) + " ms"
                            color: Theme.loadColor(page.stats.mspt / page.stats.tickBudgetMs)
                            font.pixelSize: 30
                            font.bold: true
                        }
                        Text {
                            text: qsTr("MSPT (budget %1 ms)").arg(page.stats.tickBudgetMs.toFixed(0))
                            color: Theme.fgMuted
                            font.pixelSize: 11
                        }
                    }

                    Item {
                        Layout.fillWidth: true
                    }
                }

                TickGraph {
                    Layout.fillWidth: true
                    samples: page.stats.tickTimesMs
                    budgetMs: page.stats.tickBudgetMs
                }
            }
        }

        Card {
            title: page.stats.cpuTempC < 0 ? qsTr("CPU — %1 %").arg(page.stats.cpuTotal.toFixed(1)) : qsTr("CPU — %1 %  ·  %2 °C").arg(page.stats.cpuTotal.toFixed(1)).arg(page.stats.cpuTempC.toFixed(0))
            Layout.fillWidth: true

            CoreGrid {
                width: parent.width
                usages: page.stats.cpuPerCore
                coreCount: page.stats.cpuCoreCount
            }
        }

        Card {
            title: qsTr("NETWORK")
            Layout.fillWidth: true

            RowLayout {
                width: parent.width
                spacing: 32

                ColumnLayout {
                    spacing: 0
                    Text {
                        text: "\u2193 " + Format.rate(page.stats.netInBps)
                        color: Theme.fg
                        font.pixelSize: 18
                        font.family: "monospace"
                    }
                    Text {
                        text: qsTr("inbound")
                        color: Theme.fgMuted
                        font.pixelSize: 11
                    }
                }

                ColumnLayout {
                    spacing: 0
                    Text {
                        text: "\u2191 " + Format.rate(page.stats.netOutBps)
                        color: Theme.fg
                        font.pixelSize: 18
                        font.family: "monospace"
                    }
                    Text {
                        text: qsTr("outbound")
                        color: Theme.fgMuted
                        font.pixelSize: 11
                    }
                }

                Item {
                    Layout.fillWidth: true
                }

                RowLayout {
                    spacing: 16
                    Layout.fillWidth: false
                    Layout.alignment: Qt.AlignRight | Qt.AlignVCenter

                    CopyableText {
                        visible: page.stats.javaAddress !== ""
                        value: page.stats.javaAddress
                        label: qsTr("Java Edition")
                        Layout.fillWidth: false
                    }

                    CopyableText {
                        visible: page.stats.bedrockAddress !== ""
                        value: page.stats.bedrockAddress
                        label: qsTr("Bedrock Edition")
                        Layout.fillWidth: false
                    }
                }
            }
        }
    }
}
