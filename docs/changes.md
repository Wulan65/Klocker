2026-11-09 09:08 - Project Created. Added the Rust starter app and clarified its requirements and build checks.
2026-11-09 09:17 - Renamed the application to Klocker and replaced the starter CLI with a low-resource fullscreen black screen that wakes only on keyboard or mouse-button presses.
2026-11-09 09:23 - Added a low-level mouse hook to consume the full mouse click, including release, before closing the blank screen; this prevents clicks leaking to the previously active application.
2026-11-09 09:56 - Added a localized bouncing clock and date overlay that only repaints when the displayed text changes.
2026-11-09 10:02 - Increased the clock to 120 px, tightened the clock/date spacing, and extended vertical bounce bounds to the text edge.
2026-11-09 10:12 - Set a 15 px visible clock/date gap, removed seconds, and reduced system time/date polling to every five seconds.
2026-11-09 10:25 - Switched to glyph-tight bounds, used `00:00` as the minimum clock width, and aligned the bounce to visible text edges.
2026-11-09 10:32 - Randomized the clock's initial direction using the Windows system RNG while keeping its speed constant.
2026-11-13 16:34 - Switched the Windows executable to the GUI subsystem to prevent a console window flashing at startup; startup failures now appear in an error dialog.