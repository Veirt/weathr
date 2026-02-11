# weathr

[![Crates.io](https://img.shields.io/crates/v/weathr.svg)](https://crates.io/crates/weathr)
[![Downloads](https://img.shields.io/crates/d/weathr.svg)](https://crates.io/crates/weathr)
[![License](https://img.shields.io/crates/l/weathr.svg)](https://github.com/veirt/weathr/blob/main/LICENSE)

A terminal weather app with ASCII animations driven by real-time weather data.

Features real-time weather from Open-Meteo with animated rain, snow, thunderstorms, flying airplanes, day/night cycles, and auto-location detection.

## Demo

|                                    Thunderstorm Night                                     |                             Snow                              |
| :---------------------------------------------------------------------------------------: | :-----------------------------------------------------------: |
| <img src="docs/thunderstorm-night.gif" width="600" height="400" alt="Thunderstorm Night"> | <img src="docs/snow.gif" width="600" height="400" alt="Snow"> |

## Contents

- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Privacy](#privacy)
- [Roadmap](#roadmap)
- [License](#license)

## Installation

### Via Cargo

```bash
cargo install weathr
```

### Build from Source

You need Rust installed.

```bash
git clone https://github.com/veirt/weathr.git
cd weathr
cargo install --path .
```

## Configuration

The config file is at `~/.config/weathr/config.toml`.

### Setup

```bash
mkdir -p ~/.config/weathr
```

Edit `~/.config/weathr/config.toml`:

```toml
# Hide the HUD (Heads Up Display) with weather details
hide_hud = false

[location]
# Location coordinates (overridden if auto = true)
latitude = 40.7128
longitude = -74.0060

# Auto-detect location via IP (defaults to true if config missing)
auto = false

# Hide the location name in the UI
hide = false

[units]
# Temperature unit: "celsius" or "fahrenheit"
temperature = "celsius"

# Wind speed unit: "kmh", "ms", "mph", or "kn"
wind_speed = "kmh"

# Precipitation unit: "mm" or "inch"
precipitation = "mm"
```

### Example Locations

```toml
# Tokyo, Japan
latitude = 35.6762
longitude = 139.6503

# Sydney, Australia
latitude = -33.8688
longitude = 151.2093
```

## Usage

Run with real-time weather:

```bash
weathr
```

### CLI Options

Simulate weather conditions for testing:

```bash
# Simulate rain
weathr --simulate rain

# Simulate snow at night
weathr --simulate snow --night

# Clear day with falling leaves
weathr --simulate clear --leaves
```

Available weather conditions:

- Clear Skies: `clear`, `partly-cloudy`, `cloudy`, `overcast`
- Precipitation: `fog`, `drizzle`, `rain`, `freezing-rain`, `rain-showers`
- Snow: `snow`, `snow-grains`, `snow-showers`
- Storms: `thunderstorm`, `thunderstorm-hail`

Override configuration:

```bash
# Use imperial units (°F, mph, inch)
weathr --imperial

# Use metric units (°C, km/h, mm) - default
weathr --metric

# Auto-detect location via IP
weathr --auto-location

# Hide location coordinates
weathr --hide-location

# Hide status HUD
weathr --hide-hud

# Combine flags
weathr --imperial --auto-location
```

### Keyboard Controls

- `q` or `Q` - Quit
- `Ctrl+C` - Exit

### Environment Variables

The application respects several environment variables:

- `NO_COLOR` - When set, disables all color output (accessibility feature)
- `COLORTERM` - Detects truecolor support (values: "truecolor", "24bit")
- `TERM` - Used for terminal capability detection (e.g., "xterm-256color")

Examples:

```bash
# Disable colors for accessibility
NO_COLOR=1 weathr
```

## Privacy

### Location Detection

When using `auto = true` in config or the `--auto-location` flag, the application makes a request to `ipinfo.io` to detect your approximate location based on your IP address.

This is optional. You can disable auto-location and manually specify coordinates in your config file to avoid external API calls.

## Roadmap

- [ ] Support for OpenWeatherMap, WeatherAPI, etc.
- [ ] Pre-built binaries for ARM64 arch.
- [ ] Installation via AUR.
- [ ] Key bindings for manual refresh, speed up animations, pause animations, and toggle HUD.

## License

GPL-3.0-or-later

## Credits

### Weather Data

Weather data provided by [Open-Meteo.com](https://open-meteo.com/) under the [CC BY 4.0 license](https://creativecommons.org/licenses/by/4.0/).

### ASCII Art

- **Source**: https://www.asciiart.eu/
- **House**: Joan G. Stark
- **Airplane**: Joan G. Stark
- **Sun**: Hayley Jane Wakenshaw (Flump)
- **Moon**: Joan G. Stark

_Note: If any ASCII art is uncredited or misattributed, it belongs to the original owner._
