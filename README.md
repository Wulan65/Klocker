# Klocker

A lightweight Windows screen blanking application.

It covers the desktop with a black screen with bouncing clock and date until a key is pressed or a mouse button is clicked.

Mouse movement alone does not wake it, and the wake input is consumed rather than passed to the application that was open beforehand.
After a mouse click, the screen stays black until the button is released so neither half of the click reaches that application.
Bouncing clock and date use the Windows regional format, with 120 px and 20 px text respectively and 15 px of visible space between them.
Time is shown without seconds, and the system time/date are checked every five seconds.
The overlay uses tight glyph bounds, with the clock frame sized to `00:00` or the wider formatted time when needed, and bounces at the visible text edge.
The clock starts in a randomly selected direction each time it launches.

<img width="341" height="217" alt="klocker screenshot" src="https://github.com/user-attachments/assets/3d7ae72d-4e91-4cf5-b15a-243708e14b5b" />


## Requirements

Install the stable Rust toolchain with Cargo on Windows.

## Run

```sh
cargo run
```

The fullscreen black screen covers all monitors. Press any key or mouse button
to close it. A locale-formatted clock and date bounce over the screen. Mouse
movement does not close it.

## Verify

```sh
cargo build
cargo test
cargo fmt --check
```

See [docs/requirements.md](docs/requirements.md) for the project requirements
and [docs/changes.md](docs/changes.md) for the build history.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for
details.
