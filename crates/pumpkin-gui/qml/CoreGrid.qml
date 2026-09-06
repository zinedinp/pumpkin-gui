import QtQuick
import QtQuick.Layouts
import org.pumpkin.gui

// One bar per CPU core, btop-style: the bar fills with load and shifts green -> yellow -> red.
// Laid out in a grid that reflows so a 4-core laptop and a 32-core server both look sensible.
Item {
    id: grid

    // A QList<double> from the ServerStats QObject, read as a JS sequence.
    property var usages: []
    // The Repeater's model is this count, not `usages` itself. Binding the list directly would
    // recreate every delegate whenever a sample arrives, and a fresh delegate starts at width 0 --
    // which is why the bars used to animate up from nothing instead of moving from their previous
    // value.
    property int coreCount: 0

    property int columns: Math.max(1, Math.min(8, Math.ceil(Math.sqrt(coreCount))))

    readonly property int rowCount: Math.ceil(coreCount / columns)
    implicitHeight: rowCount * 20 + Math.max(0, rowCount - 1) * 4

    function usageAt(index) {
        const value = grid.usages[index];
        return value === undefined ? 0 : value;
    }

    GridLayout {
        anchors.fill: parent
        columns: grid.columns
        columnSpacing: 10
        rowSpacing: 4

        Repeater {
            model: grid.coreCount

            delegate: RowLayout {
                id: core

                required property int index

                readonly property real usage: grid.usageAt(index)

                Layout.fillWidth: true
                spacing: 6

                Text {
                    text: core.index
                    color: Theme.fgMuted
                    font.pixelSize: 10
                    font.family: "monospace"
                    horizontalAlignment: Text.AlignRight
                    Layout.preferredWidth: 18
                }

                Rectangle {
                    id: track

                    Layout.fillWidth: true
                    Layout.preferredHeight: 12
                    radius: 3
                    color: Theme.surfaceAlt

                    // Anchored to the track by id rather than sized off `parent.width`: inside a
                    // layout the parent's width is still 0 when the binding first runs.
                    Rectangle {
                        anchors.left: parent.left
                        anchors.top: parent.top
                        anchors.bottom: parent.bottom
                        width: track.width * Math.max(0, Math.min(1, core.usage / 100))
                        radius: parent.radius
                        color: Theme.loadColor(core.usage / 100)

                        // Now that delegates survive a sample, this animates between readings
                        // rather than restarting from zero.
                        Behavior on width {
                            enabled: Theme.animations
                            NumberAnimation {
                                duration: 220
                                easing.type: Easing.OutQuad
                            }
                        }
                        Behavior on color {
                            enabled: Theme.animations
                            ColorAnimation {
                                duration: 220
                            }
                        }
                    }
                }

                Text {
                    text: Math.round(core.usage) + "%"
                    color: Theme.fgMuted
                    font.pixelSize: 10
                    font.family: "monospace"
                    horizontalAlignment: Text.AlignRight
                    Layout.preferredWidth: 30
                }
            }
        }
    }
}
