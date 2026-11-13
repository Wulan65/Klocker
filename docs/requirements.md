# Requirements: Klocker

## Purpose

Provide a minimal, low-resource Windows screen blanking application that presents a black
screen with a bouncing clock and date until deliberate keyboard or mouse-button
input wakes it.

## Current Scope

- Provide a Windows executable written in Rust and named `klocker`.
- Launch without a console window; show startup failures in a Windows error
  dialog.
- Cover the virtual desktop, including all monitors, with a fullscreen black
  window.
- Display a bouncing clock and date overlay, formatted with the current Windows
  user's regional time and date settings.
- Choose one of eight equal-speed starting directions randomly at launch.
- Render the clock at 120 px and the date at 20 px, with 15 px of visible space
  between them and text color `#CCCCCC` (80% white).
- Omit seconds and check the system-formatted time and date every five seconds.
- Use glyph-ink bounds for minimal spacing and edge bouncing. Size the clock
  frame to `00:00` or the wider formatted time when needed, without extra margin.
- Move the cached text overlay without repainting it; redraw only when its
  formatted time or date changes.
- Wake on keyboard key-down or mouse-button-down input. Mouse movement alone
  must not wake the application.
- Consume the wake input so it is not delivered to the previously active
  application. For mouse buttons, consume both press and release and keep the
  blank screen visible until all pressed buttons are released.
- Keep the implementation small and idle with minimal resource use.

## Technical Requirements

- Use Cargo and stable Rust.
- Keep application source code under `src/`.
- Avoid external dependencies where practical.
- Keep generated build output out of source control.

## Quality and Acceptance Criteria

- `cargo build` completes successfully.
- `cargo test` completes successfully.
- `cargo fmt --check` reports no formatting changes.
- Any added behavior has focused automated tests where practical.
- Update the README when setup or usage changes.

## Requirements and Build Log Maintenance

- Before each implementation/build iteration, review this file and treat it as the source of truth for agreed behavior.
- When a requirement is added, changed, or clarified, update this file to match the current agreement.
- Keep requirements synchronized with intentional user-facing behavior changes; do not rewrite unrelated requirements.
- For every delivered build, append one brief entry to the bottom of `docs/changes.md` describing what changed and why.
- Timestamp each entry in local time using `YYYY-MM-DD HH:MM`.