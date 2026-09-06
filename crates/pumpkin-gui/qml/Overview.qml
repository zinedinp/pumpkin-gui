import QtQuick
import QtQuick.Layouts
import org.pumpkin.gui

// The landing page: the numbers you check at a glance, with the console right underneath.
//
// The console gets the growing half deliberately -- it is the part you actually work in, while
// the stats above only need enough room to be readable.
ColumnLayout {
    id: page

    required property var stats
    required property var consoleController

    spacing: Theme.gap

    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.gap

        StatTile {
            Layout.fillWidth: true
            label: qsTr("TPS")
            value: page.stats.tps.toFixed(2)
            caption: qsTr("%1 of %2 ms per tick").arg(page.stats.mspt.toFixed(1)).arg(page.stats.tickBudgetMs.toFixed(0))
            accent: Theme.tpsColor(page.stats.tps)
        }

        StatTile {
            Layout.fillWidth: true
            label: qsTr("CPU")
            value: page.stats.cpuTotal.toFixed(0) + " %"
            caption: qsTr("%1 cores").arg(page.stats.cpuPerCore.length)
            accent: Theme.loadColor(page.stats.cpuTotal / 100)
            fraction: page.stats.cpuTotal / 100
        }

        StatTile {
            Layout.fillWidth: true
            label: qsTr("MEMORY")
            value: Format.bytes(page.stats.memProcessRss)
            caption: qsTr("system %1 / %2").arg(Format.bytes(page.stats.memSystemUsed)).arg(Format.bytes(page.stats.memSystemTotal))
            accent: Theme.accent
            fraction: page.stats.memSystemTotal > 0 ? page.stats.memSystemUsed / page.stats.memSystemTotal : 0
        }

        StatTile {
            Layout.fillWidth: true
            label: qsTr("STORAGE")
            value: page.stats.worldsSizeBytes < 0 ? qsTr("scanning…") : Format.bytes(page.stats.worldsSizeBytes)
            caption: qsTr("%1 free of %2").arg(Format.bytes(page.stats.diskFree)).arg(Format.bytes(page.stats.diskTotal))
            accent: Theme.accent
            fraction: page.stats.diskTotal > 0 ? 1 - page.stats.diskFree / page.stats.diskTotal : 0
        }

        StatTile {
            Layout.fillWidth: true
            label: qsTr("PLAYERS")
            value: page.stats.playerCount
            caption: qsTr("up %1").arg(Format.duration(page.stats.uptimeSecs))
            accent: Theme.accent
        }
    }

    ConsoleView {
        Layout.fillWidth: true
        Layout.fillHeight: true
        controller: page.consoleController
    }
}
