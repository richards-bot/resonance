# Resonance

A 2D particle physics playground where **the physics generates the music**.

Spawn particles, place gravity wells, and watch collisions produce real-time sine tones tuned to an A minor pentatonic scale. Built with Rust + Bevy 0.15.

---

## Download & Run

Download the latest binary from the [Releases](../../releases/latest) page.

### macOS (Apple Silicon)

```bash
curl -L https://github.com/YOUR_USERNAME/resonance/releases/latest/download/resonance-macos-arm64 -o resonance
xattr -d com.apple.quarantine resonance   # remove quarantine flag
chmod +x resonance
./resonance
```

### macOS (Intel)

```bash
curl -L https://github.com/YOUR_USERNAME/resonance/releases/latest/download/resonance-macos-x86_64 -o resonance
xattr -d com.apple.quarantine resonance
chmod +x resonance
./resonance
```

### Linux (x86_64)

```bash
curl -L https://github.com/YOUR_USERNAME/resonance/releases/latest/download/resonance-linux-x86_64 -o resonance
chmod +x resonance
./resonance
```

> **Linux note:** requires a working audio device (ALSA/PulseAudio) and a display server (X11 or Wayland).

---

## Controls

| Key / Action | Effect |
|---|---|
| **Space** | Spawn 20 particles at random positions |
| **Left click** (empty space) | Place a gravity well |
| **Left click + drag** (on well) | Move the gravity well |
| **Right click** (on well) | Remove the gravity well |
| **C** | Clear all particles |
| **R** | Reset — clear particles and all wells |
| **+** / **=** | Increase gravity well strength |
| **-** | Decrease gravity well strength |

---

## How It Works

- **Particles** spawn with random velocity vectors and a frequency from the A minor pentatonic scale (A, C, D, E, G across three octaves).
- **Colour** maps to pitch — blue = low frequency, red = high frequency.
- **Gravity wells** attract particles with Newtonian gravity (F = G·m/r²). Left-click anywhere to place one.
- **Collisions** trigger a 150 ms sine tone. Collision speed sets the volume; both particle frequencies play for harmonic richness.
- **Voice limiting** caps simultaneous tones at 16 to prevent audio overload.
- **Motion trails** show each particle's recent path.

---

## Build from Source

Requires Rust stable (1.70+).

```bash
git clone https://github.com/YOUR_USERNAME/resonance
cd resonance

# macOS / Linux with ALSA headers:
cargo build --release

# Linux without ALSA (silent mode — visual only):
cargo build --release --no-default-features

./target/release/resonance
```

### Linux build dependencies

```bash
sudo apt-get install -y pkg-config libasound2-dev libudev-dev libxkbcommon-dev
```

---

## Architecture

```
src/
  main.rs              — Bevy App setup
  physics/
    particles.rs       — Particle component, spawn, integration
    gravity.rs         — Gravity well + force system
    collision.rs       — Elastic collision detection & response
  audio/
    synth.rs           — Sine tone voice pool (background thread)
    scale.rs           — A minor pentatonic frequency table
  rendering/
    particles.rs       — Sprite visuals + motion trails (gizmos)
    wells.rs           — Pulsing gravity well rings (gizmos)
    ui.rs              — HUD (particle count, controls hint)
  input/
    mod.rs             — Mouse + keyboard input
```

---

## Releases

GitHub Actions builds release binaries automatically on `v*` tag pushes:

| Target | Binary |
|---|---|
| macOS Apple Silicon | `resonance-macos-arm64` |
| macOS Intel | `resonance-macos-x86_64` |
| Linux x86_64 | `resonance-linux-x86_64` |

To cut a release:

```bash
git tag v0.1.0
git push origin v0.1.0
```
