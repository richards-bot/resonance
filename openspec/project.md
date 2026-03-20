# Resonance — Particle Physics Sonification Engine

## What is this?

A 2D particle physics playground written in Rust where **the physics generates the music**. Every particle has a musical identity. Collisions, gravity wells, and emergent clustering produce real-time audio — generative music you control with physics.

## Target Users

- The developer (Ricky) — ex professional musician, loves creative tools
- Eventually: anyone interested in generative music / interactive art

## Core Concept

- Spawn particles that have frequencies mapped to a **pentatonic scale** (always sounds good)
- Mouse creates/moves **gravity wells** that attract particles
- **Collisions** trigger audio tones — velocity determines volume, mass affects pitch
- **Dense clusters** create richer harmonic texture
- The emergent soundscape comes entirely from the physics — no pre-written score

---

## Phase 1 Scope (build this)

### Visual
- 2D canvas, dark background
- Particles rendered as glowing circles, coloured by their base frequency (hue maps to pitch)
- Motion trails (short fading tail on each particle)
- Gravity wells shown as subtle pulsing rings
- Particle count display (top left)

### Physics
- Newtonian gravity towards each well (F = G*m/r²)
- Particle-particle elastic collision detection and response
- Linear drag / damping (particles slow over time without gravity)
- Particles spawn at random positions with random velocity vectors

### Audio
- Each particle has a frequency assigned at spawn, quantised to **A minor pentatonic** (A, C, D, E, G across 3 octaves)
- On collision: trigger a short sine tone at the particle's frequency
  - Volume proportional to collision velocity (quiet tap = soft note, hard collision = loud note)
  - Duration: ~150ms with quick attack and exponential decay
- Limit simultaneous voices to 16 to prevent audio overload
- Audio output via the system's default audio device

### Controls
- **Left click (empty space)**: spawn a gravity well
- **Left click + drag (on existing well)**: move it
- **Right click on well**: remove it
- **Space**: spawn 20 particles at random positions
- **C**: clear all particles
- **R**: reset everything (clear particles + wells)
- **+/-**: increase/decrease gravity well strength

### Distribution
- GitHub Actions CI/CD pipeline that:
  - Builds on every push to `main`
  - On any tag push (`v*`), creates a GitHub Release
  - Attaches compiled binaries for:
    - macOS arm64 (Apple Silicon) — `resonance-macos-arm64`
    - macOS x86_64 (Intel) — `resonance-macos-x86_64`
    - Linux x86_64 — `resonance-linux-x86_64`
  - README has a "Download & Run" section with direct links to latest release

---

## Tech Stack

### Rendering + Window
Use **bevy 0.15** (latest stable). Excellent 2D support, great performance, cross-platform, active ecosystem.

### Audio
Use **bevy_audio** (built into bevy) with **fundsp** for real-time synthesis, OR use **rodio** with manual sine wave generation if fundsp integration is complex. 

**Preference**: if fundsp adds complexity, use rodio with a simple sine wave AudioSource implementation. Working audio > perfect audio.

### Physics
Implement custom 2D physics — don't use a physics engine. The simulation is simple enough:
- Gravity wells: apply force to each particle each frame
- Collisions: broad phase (spatial hashing or simple O(n²) for <500 particles), narrow phase circle-circle
- Integration: semi-implicit Euler (stable and simple)

---

## Architecture

```
src/
  main.rs           — App setup, bevy App builder
  physics/
    mod.rs          — Physics plugin
    particles.rs    — Particle component, spawn system
    gravity.rs      — Gravity well component + force system
    collision.rs    — Collision detection + response
  audio/
    mod.rs          — Audio plugin
    synth.rs        — Sine tone generation, voice management
    scale.rs        — Pentatonic scale frequency lookup
  rendering/
    mod.rs          — Rendering plugin
    particles.rs    — Particle visual + trail rendering
    wells.rs        — Gravity well visual
    ui.rs           — HUD (particle count, controls hint)
  input/
    mod.rs          — Mouse + keyboard input handling
```

---

## Non-Goals (Phase 1)

- No WASM / browser build
- No mic input (Phase 2)
- No saving/loading of scenes
- No particle-particle gravity (only well-to-particle)
- No UI panels or sliders (keyboard controls only)

---

## Definition of Done (Phase 1)

- [ ] App launches and renders particles on screen
- [ ] Particles respond to gravity wells
- [ ] Collisions produce audible tones
- [ ] Controls all work as specified
- [ ] GitHub Actions builds and uploads Mac arm64 + x86_64 binaries on tag push
- [ ] README has download instructions
- [ ] App runs on macOS without any additional dependencies (self-contained binary)
