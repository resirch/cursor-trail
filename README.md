# cursor-trail

A Windows 11 overlay that adds a customizable cursor trail and an optional hanging avatar connected by a string. The avatar swings when you move the cursor and falls naturally under gravity. Set gravity to `0` to let it float freely.

## Features

- Smooth fading cursor trail with customizable color, size, length, shape, and fade speed
- Optional hanging avatar image on a string with pendulum-style physics
- Adjustable gravity (including zero-gravity floating)
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

Press `Ctrl+C` in the terminal to quit.

## Configuration

Copy `config.example.toml` as a reference. All settings live in one TOML file.

### Trail

| Key | Description |
|-----|-------------|
| `enabled` | Turn the trail on or off |
| `color` | RGBA color array `[r, g, b, a]` |
| `max_points` | Maximum trail segments |
| `point_size` | Base size of each trail mark |
| `fade_speed` | How quickly trail marks fade out |
| `spacing` | Minimum pixel distance between new trail points |
| `shape` | `circle`, `square`, or `star` |

### Avatar

| Key | Description |
|-----|-------------|
| `enabled` | Turn the hanging avatar on or off |
| `image_path` | Path to a PNG/JPEG/GIF/WebP image (optional — uses a default circle if omitted) |
| `string_length` | Length of the string in pixels |
| `size` | Avatar display size in pixels |
| `gravity` | Downward acceleration in pixels/s². Use `0` for floating |
| `damping` | Velocity damping (0–1). Higher = less swing |
| `string_color` | RGBA color of the string |
| `string_width` | String thickness in pixels |
| `swing_boost` | How strongly cursor movement affects the swing |

### Window

| Key | Description |
|-----|-------------|
| `fps` | Target render frame rate |
| `click_through` | When true, mouse clicks pass through the overlay |

## Custom config path

```powershell
cursor-trail.exe --config C:\path\to\my-config.toml
```

## How the physics work

The avatar uses Verlet-style integration with a distance constraint to the cursor anchor. Gravity pulls it downward each frame, and the string length constraint keeps it tethered. When you move the cursor quickly, momentum carries the avatar into a natural swing. With `gravity = 0`, only damping and cursor movement affect motion — the avatar drifts and orbits freely at the end of its string.

## License

MIT
