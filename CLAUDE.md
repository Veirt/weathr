# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

weathr is a Rust terminal application that displays animated ASCII weather scenes driven by real-time data from Open-Meteo. It renders at 30 FPS using crossterm with double-buffered rendering, particle systems for precipitation, and layered compositing (background → scene → foreground → HUD).

## Build & Development Commands

```bash
cargo build --release        # Production build (thin LTO, stripped)
cargo test --verbose         # Run all tests (unit + integration)
cargo check                  # Type-check without building
cargo fmt -- --check         # Verify formatting
cargo clippy -- -D warnings  # Lint (CI treats warnings as errors)
```

Run the app:
```bash
cargo run                           # Real weather for configured location
cargo run -- --simulate rain        # Simulate a weather condition
cargo run -- --simulate snow --night  # Simulate with forced night mode
cargo run -- --auto-location        # Auto-detect location via IP
```

Minimum Rust version: 1.85.0 (edition 2024).

## Architecture

### Data Flow

```
CLI (clap) → Config (TOML) → Geolocation (ipinfo.io) → App::run()
                                                            │
OpenMeteo API → WeatherClient (with cache) → AppState → AnimationManager
                                                            │
                                                    TerminalRenderer (double-buffered)
```

### Rendering Pipeline (layered, back-to-front)

1. **Background**: sky gradient, stars, clouds, sun/moon
2. **Scene**: ASCII house (`scene/house.rs`), ground, decorations
3. **Chimney smoke** (between scene and foreground)
4. **Foreground**: rain, snow, fog, leaves, birds, airplanes, fireflies
5. **HUD**: status bar with weather data, location, units

### Key Modules

- `app.rs` — Main event loop: polls input at 30 FPS, fetches weather every 5 min, drives rendering. Keyboard controls (p/+/-/h/r/?) manage pause, speed [0.25-4.0x], HUD, refresh, and help
- `animation_manager.rs` — Orchestrates all animations, decides which to activate based on weather. Speed multiplier parameter threads through render methods to scale animation rates
- `animation/` — Each file is a self-contained animation. All implement the `Animation` trait. Particle systems (rain, snow, fireflies) use physics with wind influence
- `render/mod.rs` — `TerminalRenderer` with cell-level double buffering (only redraws changed cells). Min terminal size: 70x20
- `render/capabilities.rs` — Detects truecolor/256-color/NO_COLOR support
- `weather/client.rs` — `WeatherClient` with async fetch and disk caching
- `weather/provider.rs` — `WeatherProvider` trait; `open_meteo.rs` is the implementation
- `weather/normalizer.rs` — Converts raw API response to `WeatherData`
- `config.rs` — Loads TOML from platform-specific paths (Linux: `~/.config/weathr/`, macOS: `~/Library/Application Support/weathr/`)
- `error.rs` — Comprehensive error types with `user_friendly_message()` methods
- `geolocation.rs` — IP-based location detection with retry logic
- `cache.rs` — Disk cache for location (24h TTL) and weather data (5min TTL)

### Animation System

All animations implement the `Animation` trait (`animation/mod.rs`). `AnimationController` manages frame cycling. Particle-based animations (raindrops, snow, fireflies) maintain their own state vectors and update per-frame with wind, gravity, and randomness.

### Config Precedence

CLI flags override config file values. Config file is optional — defaults to auto-location if missing.

## CI

GitHub Actions runs on push to main and PRs: `cargo check` → `cargo test` → `cargo fmt --check` → `cargo clippy -- -D warnings` → `cargo audit`.

## Testing

Integration tests live in `tests/`. Unit tests are inline in modules (e.g., `config.rs`, `app_state.rs`). Run a single test:

```bash
cargo test test_name
cargo test --test config_integration_test  # Run one integration test file
```
