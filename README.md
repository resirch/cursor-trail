# cursor-trail

A Windows 11 overlay that adds a customizable cursor trail and an optional hanging avatar connected by a string. The avatar swings when you move the cursor and falls naturally under gravity. Set gravity to `0` to let it float freely.

## Features

- Smooth fading **line** cursor trail with customizable color, length, and width
- Optional hanging avatar image on a string with pendulum-style physics
- Adjustable gravity (including zero-gravity floating)
- **System tray icon** with quick toggles and a settings window
- Live config reload — edit the TOML file while the app is running
- Click-through overlay so it never blocks mouse input

## Requirements

- Windows 10/11
- Rust 1.74+ (for building from source)

## Build

```powershell
cd E:\Projects\cursor-trail
cargo build --release
```

The binary will be at `target\release\cursor-trail.exe`.

## Quick start

1. Generate a default config:

```powershell
cargo run --release -- --init
```

This writes `%APPDATA%\cursor-trail\config.toml`.

2. (Optional) Add an avatar image and set `image_path` in the config.

3. Run the overlay:

```powershell
cargo run --release
```

Press `Ctrl+C` in the terminal to quit, or use **Quit** from the system tray menu.

## System tray

When running, a tray icon appears in the notification area. Right-click it for:

| Menu item | Action |
|-----------|--------|
| **Settings** | Opens the settings window (sliders, color pickers, save/apply) |
| **Toggle Trail** | Enable/disable the cursor trail |
| **Toggle Avatar** | Enable/disable the hanging avatar |
| **Quit** | Exit the application |

## Configuration

Copy `config.example.toml` as a reference. All settings live in one TOML file.

### Trail

| Key | Description |
|-----|-------------|
| `enabled` | Turn the trail on or off |
| `color` | RGBA color array `[r, g, b, a]` |
| `max_length` | Maximum trail length in pixels (how far it can extend behind the cursor) |
| `width` | Line thickness in pixels |
| `taper` | Sharp tail point (`0` = uniform, `1` = needle; higher keeps full width only near the cursor) |
| `kill_time` | Seconds for the tail to catch up to the cursor when stopped (`0` = disabled). The trail grows from the cursor as you move — it does not snap back to full length |

### Avatar

| Key | Description |
|-----|-------------|
| `enabled` | Turn the hanging avatar on or off |
| `image_path` | Path to a PNG/JPEG/GIF/WebP image (optional — uses a default circle if omitted) |
| `string_length` | Total rope length in pixels |
| `string_slack` | How much the rope can compress and stretch while bending |
| `size` | Avatar display size in pixels |
| `gravity` | Downward pull in pixels/s² (`0` = floating). Default 900 |
| `damping` | Swing decay (`0` = endless swing, `1` = instant stop) |
| `string_color` | RGBA color of the string |
| `string_width` | String thickness in pixels |

### Window

| Key | Description |
|-----|-------------|
| `fps` | Overlay redraw rate (15–360). Higher is smoother but uses more CPU |

## Custom config path

```powershell
cursor-trail.exe --config C:\path\to\my-config.toml
```

## How the physics work

The avatar hangs from a multi-segment rope that can bend, coil, and stretch. Gravity pulls the whole rope downward (heavier toward the avatar). With `gravity = 0`, it floats freely.

## License

MIT
