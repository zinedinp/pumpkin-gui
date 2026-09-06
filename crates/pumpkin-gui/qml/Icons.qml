pragma Singleton

import QtQuick

// The qrc prefix comes from the QML module URI declared in build.rs; keeping the paths in one
// place stops it from being retyped at every call site.
QtObject {
    readonly property string base: "qrc:/qt/qml/org/pumpkin/gui/qml/icons/"

    readonly property url op: base + "op.svg"
    readonly property url kick: base + "kick.svg"
    readonly property url ban: base + "ban.svg"
    readonly property url copy: base + "copy.svg"
}
