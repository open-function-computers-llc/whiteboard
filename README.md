# Whiteboard

A distraction-free drawing surface for live screenshare calls (Zoom / Google Meet /
Microsoft Teams / etc.).

It is the "open GIMP, make a blank canvas, find the pencil tool, fight with brush
settings, then close everything" workflow, replaced with a single blank white window that
launches in well under a second. Draw with the mouse while you talk, then hit `Esc`. Nothing
is ever saved.

Built with native [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/eframe)
(glow/OpenGL backend) — **one small native binary, no webview, no Electron, no Node runtime**.
The release binary is ~7 MB and idles in tens of MB of RAM, which is the whole point: lean
and fast to bring up on a call.

## Why not Tauri / Electron

The UI is a blank canvas with six color swatches — there is no DOM worth a webview. A
webview stack would add a WebKit/Chromium process and a Node build step just to draw lines.
Native egui gives faster launch and a far smaller footprint for this exact use case.

## Features

- Blank white, borderless, maximized, resizable window (titlebar hidden)
- Freehand drawing with the mouse (drag to draw, click for a dot)
- Six colors: Red, Blue, Black, Green, Purple, Orange
- Infinite canvas with **pan** and **zoom**
- Zero persistence — nothing touches disk
- `Esc` quits immediately, no "save changes?" prompt

## Controls

| Action | Input |
| --- | --- |
| Draw | Left-drag |
| Dot | Left-click |
| Pick color | Click a swatch, or `1`–`6`, or `R` `B` `K` `G` `P` `O` |
| Pan | Hold `Space` + move the mouse |
| Zoom | `Ctrl` + mouse wheel (zooms toward the cursor) |
| Hide / show the on-screen chrome | `Tab` |
| Quit | `Esc` |

Color keys map to the palette order: `1`/`R` Red · `2`/`B` Blue · `3`/`K` Black ·
`4`/`G` Green · `5`/`P` Purple · `6`/`O` Orange.

`Tab` hides the palette and the hint line so you are left with a completely blank canvas.

## Prerequisites

- A recent stable **Rust** toolchain (edition 2024; developed on 1.98).
  Install via [rustup](https://rustup.rs) if you don't have it.
- **Linux:** an OpenGL-capable system plus X11/Wayland client libraries (the default
  `glow` + `x11` + `wayland` feature set). On a Debian/Ubuntu-based system:

  ```sh
  sudo apt install build-essential pkg-config libgl1-mesa-dev libegl1
  ```

  Fedora: `sudo dnf install gcc pkgconf mesa-libGL-devel mesa-libEGL-devel`.
  Arch: `sudo pacman -S base-devel mesa libglvnd`.

  macOS and Windows need no extra system packages (system OpenGL is used).

## Build

From the repository root:

```sh
# debug (fast to compile, for iterating)
cargo build

# optimized release (this is what you want to actually run)
cargo build --release
```

The finished binary lands at `target/release/whiteboard`.

Run it without installing:

```sh
cargo run --release
# or
./target/release/whiteboard
```

## Install

The simplest path is Cargo's installer, which drops the binary on your `PATH`
(`~/.cargo/bin/whiteboard`):

```sh
cargo install --path .
```

Then launch it with:

```sh
whiteboard
```

To uninstall:

```sh
cargo uninstall whiteboard
```

### Manual install

Copy the release binary anywhere on your `PATH`:

```sh
sudo install -Dm755 target/release/whiteboard /usr/local/bin/whiteboard
```

### Desktop launcher (Linux, optional)

Drop a `.desktop` entry so it shows up in your app launcher. Write
`~/.local/share/applications/whiteboard.desktop`:

```ini
[Desktop Entry]
Type=Application
Name=Whiteboard
Comment=Blank whiteboard for screenshare calls
Exec=whiteboard
Terminal=false
Categories=Utility;
```

`Exec` should point at wherever the binary lives (`whiteboard` if it is on your `PATH`,
otherwise the absolute path).

## Troubleshooting

- **Blank / black window or GL init error** — a graphics driver or missing OpenGL library.
  Confirm `glxinfo`/`eglinfo` report a working GL context and install the Mesa/dev packages
  from Prerequisites above.
- **`Ctrl`+wheel does nothing** — some desktop environments intercept `Ctrl`+scroll for their
  own zoom; check mouse/accessibility shortcuts.
- **Wayland quirks** — the app builds both `x11` and `wayland`. If a Wayland session behaves
  oddly (window focus, maximization), try an X11/XWayland session.
