# Interactive Controls Implementation

## Overview

weathr's keyboard controls (pause, speed adjustment, HUD toggle, manual refresh) are implemented directly in `App::run()` using inline state fields rather than extracted control modules or config-driven key bindings. This design prioritizes simplicity for a small set of hardcoded keys over extensibility for remapping.

## Architecture

Key press events from crossterm flow into `App::run()` match arms. Control state lives on the `App` struct as simple fields (`paused: bool`, `speed_multiplier: f32`, `hide_hud: bool`, `show_help: bool`) because these are UI concerns, not weather data. `AppState` holds weather data and formatting logic. This boundary mirrors the existing `hide_hud` placement and prevents UI state from bleeding into the data layer.

Speed multiplier flows: `App.speed_multiplier` → `AnimationManager.render_*()` → each animation system's `update()` call. Animations apply speed in one of four patterns:

1. **Velocity-based** (rain, snow, fireflies, chimney): multiply `speed_y` and `speed_x` deltas by speed multiplier. These use physics vectors — scaling velocity changes fall/drift rate.
2. **Positional** (leaves, birds, airplanes, fog): multiply position increment (x/y change per frame) by speed multiplier. These move entities by fixed increments.
3. **Scroll/twinkle** (stars, clouds): multiply scroll offset or twinkle rate by speed multiplier.
4. **Frame-based** (sunny): adjust frame delay threshold inversely — `FRAME_DELAY / speed_multiplier`. Higher speed = shorter delay = faster frame cycling.

Manual refresh (`r` key) aborts the existing weather background task and spawns a new one. This is heavier than signaling (creates new task + channel) but eliminates thread-safety concerns and reuses the existing spawn logic verbatim. The spawned task is identical to `App::new()`'s task: fetch weather, send via channel, sleep 5 minutes, repeat.

## Design Decisions

**Inline state over extracted Controls module**: Five key bindings fit comfortably in `App::run()` match arms. An extracted module would add file + enum + trait for trivial toggle/clamp logic. This is acceptable over-engineering for current scope. If key bindings grow significantly (10+ keys, nested modes, config remapping), extraction becomes justified.

**Hardcoded keys over config-driven bindings**: Config-driven keys require a string-to-KeyCode parser, validation, schema changes, and documentation. Hardcoded keys (`p`, `+`, `-`, `h`, `r`, `?`) cover all stated requirements with zero parser complexity. Trade-off: users cannot remap keys without code changes. Acceptable because remapping demand is unproven and config layer can be added later without architectural disruption.

**Separate `paused: bool` over `speed_multiplier = 0.0`**: Pause has different semantics than minimum speed. Users expect pause to freeze all motion instantly. Using `speed_multiplier = 0.0` would require special-casing resume to restore the previous speed value (requires storing pre-pause speed). Separate boolean is clearer: pause is a distinct mode, not a speed setting.

**Abort/respawn for manual refresh over command channel**: The existing weather task uses a one-way `mpsc::channel` (task → app). Adding a command channel would require refactoring the sleep loop to poll a flag periodically, introducing up to 300s delay (the REFRESH_INTERVAL sleep duration). Abort/respawn is instant: `JoinHandle::abort()` marks the task for cancellation (documented as safe by tokio), and spawning a new task reuses the same loop structure. Trade-off: slightly heavier (creates new task) but zero thread-safety risk and no sleep-loop modification.

**Speed multiplier as parameter over delta-time refactor**: The animation system uses implicit per-frame deltas (no delta-time parameter). Proper delta-time would require changing every animation's `update()` method signature and multiplying all physics by `dt`. Instead, speed multiplier scales velocity/position deltas at call sites (`drop.y += drop.speed_y * speed`). This achieves identical results for the fixed 30 FPS loop without invasive refactoring. Trade-off: multiplier is a "scaling hack" rather than proper time-based animation, but the fixed framerate makes delta-time unnecessary.

**Help text at `term_height - 2`**: The HUD renders weather info at `(2, 1)`. Attribution text renders at `(term_width - 32, term_height - 1)` (bottom-right corner). Placing help at `term_height - 2` (one row above attribution) avoids overlap at the minimum supported terminal width (70 columns). Help text renders in DarkGrey to match the attribution text styling and remain unobtrusive. If `term_width < 47` (help text length), the help is truncated with ellipsis. Guard `term_height >= 3` prevents underflow.

**Pause suppresses rendering only, not internal state**: Animation internal state (elapsed counters, particle ages) continues accumulating while paused. Only the render calls are guarded by `if !self.paused {}`. This is simpler than freezing all internal timers. Trade-off: when resumed, animations show their current timeline position (e.g., leaf swaying from mid-cycle) rather than resuming from the exact frozen frame. This feels more natural — pause is "stop rendering" not "stop time."

**Refresh indicator cleared on both success and error**: When weather fetch fails, the existing error handling (`app.rs` lines 174-199) generates offline fallback weather and sends it via the same `mpsc::channel` as successful fetches. Both paths produce an `Ok(weather)` message, so `try_recv()` clears the `[Refreshing...]` indicator in both cases. No timeout needed because the error path always produces a result.

## Invariants

- `speed_multiplier` is always in [0.25, 4.0]. Enforced at mutation site via `f32::clamp(0.25, 4.0)`. Values outside this range are clamped before assignment. 0.25 floor prevents invisible animations; 4.0 ceiling prevents unusable speed.
- `paused` and `speed_multiplier` are independent. Pause state does not modify or constrain speed. Speed can be adjusted while paused; the new speed takes effect when unpaused.
- Weather refresh abort/respawn preserves `WeatherLocation`, `WeatherUnits`, and `Arc<OpenMeteoProvider>`. These are stored on `App` and captured in the respawn closure to ensure refresh fetches the same data as the original task.
- Help text rendering must not exceed `term_width`. If terminal is narrower than 47 characters (help text length), the text is truncated with `...` suffix. Guard prevents buffer overruns.
- Manual refresh is a no-op in simulation mode. `weather_provider` is `None` when `--simulate` flag is active. `refresh_weather()` early-returns if provider is absent.
