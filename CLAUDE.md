# Resonance — Claude Code Instructions

## Project

Rust + Bevy particle physics sonification engine. Read `openspec/project.md` for full spec before doing anything.

## Rules

- Follow the spec in `openspec/project.md` exactly for Phase 1 scope
- Use bevy 0.15 (check crates.io for latest 0.15.x)
- Keep physics simple — custom implementation, no physics engine dependency
- For audio: try fundsp first, fall back to rodio + manual sine if it's a blocker
- Prefer working > perfect — Phase 1 must actually run
- Every public function/struct needs a doc comment

## Checkpoints

After each major milestone: `git add -A && git commit -m "checkpoint: ..." && git push`. Always push immediately after committing.

1. `checkpoint: project scaffolded, bevy hello world runs`
2. `checkpoint: particles spawning and rendering`
3. `checkpoint: gravity wells + physics working`
4. `checkpoint: collision detection working`
5. `checkpoint: audio tones on collision`
6. `checkpoint: controls + HUD complete`
7. `checkpoint: github actions CI/CD added`

## GitHub Actions

Must include `.github/workflows/release.yml` that:
- Builds on macOS runners (both arm64 and x86_64 via matrix)
- Triggers on `v*` tags
- Creates a GitHub Release
- Attaches binaries named `resonance-macos-arm64` and `resonance-macos-x86_64`
- Also builds `resonance-linux-x86_64` on ubuntu runner

## Do not

- Add features beyond Phase 1 scope
- Use nightly Rust (stable only)
- Add complex UI frameworks (egui etc) — keyboard controls only for Phase 1
- Implement particle-particle gravity
