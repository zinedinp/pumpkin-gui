import QtQuick
import org.pumpkin.gui

// The server's rolling window of the last 100 tick durations, oldest on the left.
//
// The sampler unrotates the server's circular buffer before handing it over, so index order is
// chronological here.
Item {
    id: graph

    // Tick durations in milliseconds (QList<double> as a JS sequence).
    property var samples: []
    // How long a tick may take at the configured tick rate. The scale only grows past it when
    // ticks actually overrun.
    property real budgetMs: 50
    // The true max of the window; the Y scale still floors at the budget so idle servers do not
    // stretch 2 ms ticks to the top of the plot.
    readonly property real samplePeak: {
        let max = 0;
        for (let i = 0; i < samples.length; ++i)
            max = Math.max(max, samples[i]);
        return max;
    }
    readonly property real peak: Math.max(budgetMs, samplePeak)

    implicitHeight: 90

    Rectangle {
        anchors.fill: parent
        color: Theme.surfaceAlt
        radius: 4
    }

    // Laid out by hand: a Row would force y = 0, so the bars would hang from the top of the
    // plot instead of growing from the baseline.
    Item {
        id: plot

        anchors.fill: parent
        anchors.margins: 2

        Repeater {
            model: graph.samples

            delegate: Rectangle {
                required property int index
                required property real modelData

                readonly property real barWidth: (plot.width - (graph.samples.length - 1)) / graph.samples.length

                width: barWidth
                height: Math.max(1, plot.height * Math.min(1, modelData / graph.peak))
                x: index * (barWidth + 1)
                y: plot.height - height
                color: Theme.loadColor(modelData / graph.budgetMs)
            }
        }
    }

    // Ticks that cross this line missed the configured tick rate. When nothing overruns it sits
    // at the top of the scale (budget == peak).
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: 2
        anchors.rightMargin: 2
        height: 1
        color: Theme.border
        y: 2 + (parent.height - 4) * (1 - graph.budgetMs / graph.peak)
        visible: graph.samples.length > 0
    }

    // Sits on a pill because tall bars reach the top of the plot and would otherwise run
    // straight through the text.
    Rectangle {
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: 3
        width: peakLabel.width + 10
        height: peakLabel.height + 4
        radius: 3
        color: Theme.surface
        opacity: 0.85
        visible: graph.samples.length > 0

        Text {
            id: peakLabel
            anchors.centerIn: parent
            text: qsTr("peak %1 ms").arg(graph.samplePeak.toFixed(0))
            color: Theme.fgMuted
            font.pixelSize: 10
            font.family: "monospace"
        }
    }
}
