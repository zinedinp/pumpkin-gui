pragma Singleton

import QtQuick

// Shared value formatting, so a byte count reads the same on every page.
QtObject {
    function bytes(value) {
        if (value === undefined || value === null || value < 0)
            return "–";

        const units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
        let size = value;
        let unit = 0;
        while (size >= 1024 && unit < units.length - 1) {
            size /= 1024;
            unit += 1;
        }
        // Bytes and KiB have no meaningful fraction; larger units do.
        return size.toFixed(unit >= 2 ? 1 : 0) + " " + units[unit];
    }

    function rate(bytesPerSecond) {
        if (bytesPerSecond === undefined || bytesPerSecond === null || bytesPerSecond < 0)
            return "–";
        return bytes(bytesPerSecond) + "/s";
    }

    function duration(totalSeconds) {
        if (totalSeconds === undefined || totalSeconds < 0)
            return "–";

        const seconds = Math.floor(totalSeconds);
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);

        if (days > 0)
            return days + "d " + hours + "h";
        if (hours > 0)
            return hours + "h " + minutes + "m";
        if (minutes > 0)
            return minutes + "m " + (seconds % 60) + "s";
        return seconds + "s";
    }

    function gameTime(timeOfDay) {
        const day = Math.floor(timeOfDay / 24000) + 1;
        const ticks = ((timeOfDay % 24000) + 24000) % 24000;
        const totalMinutes = (ticks * 60 / 1000 + 360) % 1440;
        const hours = Math.floor(totalMinutes / 60);
        const minutes = Math.floor(totalMinutes % 60);
        return qsTr("Day %1 · %2:%3").arg(day).arg(("0" + hours).slice(-2)).arg(("0" + minutes).slice(-2));
    }
}
