mod animation;
mod animation_manager;
mod app;
mod app_state;
mod cache;
mod config;
mod error;
mod geocoding;
mod geolocation;
mod render;
mod scene;
mod weather;

use clap::Parser;
use config::Config;
use crossterm::{
    cursor, execute,
    style::ResetColor,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
use render::TerminalRenderer;
use std::path::PathBuf;
use std::{io, panic};

const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n\nWeather data by Open-Meteo.com (https://open-meteo.com/)\n",
    "Licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)"
);

fn info(silent: bool, msg: &str) {
    if !silent {
        println!("{}", msg);
    }
}

#[derive(Parser)]
#[command(version, long_version = LONG_VERSION, about = "Terminal-based ASCII weather application", long_about = None, trailing_var_arg = true)]
struct Cli {
    #[arg(
        short,
        long,
        value_name = "CONDITION",
        help = "Simulate weather condition (clear, rain, drizzle, snow, etc.)"
    )]
    simulate: Option<String>,

    #[arg(
        short,
        long,
        help = "Simulate night time (for testing moon, stars, fireflies)"
    )]
    night: bool,

    #[arg(short, long, help = "Enable falling autumn leaves")]
    leaves: bool,

    #[arg(long, help = "Auto-detect location via IP (uses ipinfo.io)")]
    auto_location: bool,

    #[arg(long, help = "Hide location coordinates in UI")]
    hide_location: bool,

    #[arg(long, help = "Hide HUD (status line)")]
    hide_hud: bool,

    #[arg(
        long,
        conflicts_with = "metric",
        help = "Use imperial units (°F, mph, inch)"
    )]
    imperial: bool,

    #[arg(
        long,
        conflicts_with = "imperial",
        help = "Use metric units (°C, km/h, mm)"
    )]
    metric: bool,

    #[arg(long, help = "Run silently (suppress non-error output)")]
    silent: bool,

    #[arg(long, help = "Set a default city for weather lookups and save to config")]
    set_default: bool,

    #[arg(
        short,
        long,
        value_name = "SECONDS",
        help = "Run for a specified duration then exit"
    )]
    duration: Option<u64>,

    #[arg(long, help = "Add weathr to your shell startup file")]
    install_shell: bool,

    #[arg(long, help = "Remove weathr from your shell startup file")]
    uninstall_shell: bool,

    /// City name for weather lookup (e.g. weathr london)
    #[arg(trailing_var_arg = true, num_args = 0..)]
    city: Vec<String>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show, ResetColor);
        default_hook(info);
    }));

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let err_str = err.to_string();
            if err_str.contains("--simulate") && err_str.contains("value is required") {
                eprintln!("{}", err);
                eprintln!();
                eprintln!("Available weather conditions:");
                eprintln!();
                eprintln!("  Clear Skies:");
                eprintln!("    clear              - Clear sunny sky");
                eprintln!("    partly-cloudy      - Partial cloud coverage");
                eprintln!("    cloudy             - Cloudy sky");
                eprintln!("    overcast           - Overcast sky");
                eprintln!();
                eprintln!("  Precipitation:");
                eprintln!("    fog                - Foggy conditions");
                eprintln!("    drizzle            - Light drizzle");
                eprintln!("    rain               - Rain");
                eprintln!("    freezing-rain      - Freezing rain");
                eprintln!("    rain-showers       - Rain showers");
                eprintln!();
                eprintln!("  Snow:");
                eprintln!("    snow               - Snow");
                eprintln!("    snow-grains        - Snow grains");
                eprintln!("    snow-showers       - Snow showers");
                eprintln!();
                eprintln!("  Storms:");
                eprintln!("    thunderstorm       - Thunderstorm");
                eprintln!("    thunderstorm-hail  - Thunderstorm with hail");
                eprintln!();
                eprintln!("Examples:");
                eprintln!("  weathr --simulate rain");
                eprintln!("  weathr --simulate snow --night");
                eprintln!("  weathr -s thunderstorm -n");
                std::process::exit(1);
            } else {
                err.exit();
            }
        }
    };

    // Handle --install-shell / --uninstall-shell
    if cli.install_shell || cli.uninstall_shell {
        let shell = std::env::var("SHELL").unwrap_or_default();
        let home = std::env::var("HOME").unwrap_or_default();

        let rc_file: Option<PathBuf> = if shell.contains("zsh") {
            Some(PathBuf::from(format!("{}/.zshrc", home)))
        } else if shell.contains("bash") {
            let bash_profile = PathBuf::from(format!("{}/.bash_profile", home));
            let bashrc = PathBuf::from(format!("{}/.bashrc", home));
            if bash_profile.exists() {
                Some(bash_profile)
            } else {
                Some(bashrc)
            }
        } else if shell.contains("fish") {
            Some(PathBuf::from(format!(
                "{}/.config/fish/config.fish",
                home
            )))
        } else {
            None
        };

        if let Some(rc_path) = rc_file {
            if cli.install_shell {
                let snippet = "\n# weathr - terminal weather display\nweathr --duration 5\n";
                let contents = std::fs::read_to_string(&rc_path).unwrap_or_default();
                if contents.contains("weathr") {
                    println!("weathr is already in {}", rc_path.display());
                } else {
                    std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(&rc_path)
                        .and_then(|mut f| {
                            use std::io::Write;
                            f.write_all(snippet.as_bytes())
                        })
                        .expect("Failed to write to shell rc file");
                    println!("Added weathr to {}", rc_path.display());
                    println!("Restart your shell or run: source {}", rc_path.display());
                }
            } else {
                // --uninstall-shell
                let contents = std::fs::read_to_string(&rc_path).unwrap_or_default();
                let filtered: Vec<&str> = contents
                    .lines()
                    .filter(|line| {
                        let trimmed = line.trim();
                        !trimmed.contains("weathr")
                    })
                    .collect();
                let new_contents = filtered.join("\n") + "\n";
                std::fs::write(&rc_path, new_contents).expect("Failed to write to shell rc file");
                println!("Removed weathr from {}", rc_path.display());
            }
        } else {
            eprintln!("Could not detect your shell. Supported: zsh, bash, fish");
        }
        return Ok(());
    }

    let mut config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            eprintln!("\nAuto-detecting location via IP...");
            eprintln!("\nTo customize, create a config file at:");
            eprintln!(
                "  Linux: ~/.config/weathr/config.toml (or $XDG_CONFIG_HOME/weathr/config.toml)"
            );
            eprintln!("  macOS: ~/Library/Application Support/weathr/config.toml");
            eprintln!("\nExample config.toml:");
            eprintln!("  [location]");
            eprintln!("  latitude = 52.52");
            eprintln!("  longitude = 13.41");
            eprintln!("  auto = false  # Set to true to auto-detect location");
            eprintln!();
            Config::default()
        }
    };

    // Track location label for display
    let mut location_label: Option<String> = None;

    // Build label from saved config if present
    if let Some(ref city) = config.location.city {
        let label = if let Some(ref country) = config.location.country {
            format!("{}, {}", city, country)
        } else {
            city.clone()
        };
        location_label = Some(label);
    }

    // Handle city name argument (e.g. `weathr london`)
    let city_query: Option<String> = if !cli.city.is_empty() {
        Some(cli.city.join(" "))
    } else {
        None
    };

    // --set-default: geocode (or auto-detect) and save to config, then exit
    if cli.set_default {
        if let Some(ref city_name) = city_query {
            match geocoding::geocode_city(city_name).await {
                Ok(loc) => {
                    config.location.latitude = loc.latitude;
                    config.location.longitude = loc.longitude;
                    config.location.city = Some(loc.name.clone());
                    config.location.country = loc.country.clone();
                    match config.save() {
                        Ok(path) => {
                            let display = if let Some(ref c) = loc.country {
                                format!("{}, {}", loc.name, c)
                            } else {
                                loc.name.clone()
                            };
                            println!("Default location set to: {}", display);
                            println!("Saved to: {}", path.display());
                        }
                        Err(e) => eprintln!("Failed to save config: {}", e),
                    }
                }
                Err(e) => eprintln!("{}", e.user_friendly_message()),
            }
        } else {
            // No city given — use IP-based auto-detect
            match geolocation::detect_location().await {
                Ok(geo_loc) => {
                    config.location.latitude = geo_loc.latitude;
                    config.location.longitude = geo_loc.longitude;
                    config.location.auto = true;
                    if let Some(ref city) = geo_loc.city {
                        config.location.city = Some(city.clone());
                    }
                    match config.save() {
                        Ok(path) => {
                            let display = geo_loc
                                .city
                                .as_deref()
                                .unwrap_or("auto-detected location");
                            println!("Default location set to: {}", display);
                            println!("Saved to: {}", path.display());
                        }
                        Err(e) => eprintln!("Failed to save config: {}", e),
                    }
                }
                Err(e) => eprintln!("{}", e.user_friendly_message()),
            }
        }
        return Ok(());
    }

    // If a city name was provided, geocode it
    if let Some(ref city_name) = city_query {
        match geocoding::geocode_city(city_name).await {
            Ok(loc) => {
                config.location.latitude = loc.latitude;
                config.location.longitude = loc.longitude;
                config.location.city = Some(loc.name.clone());
                config.location.country = loc.country.clone();
                let label = if let Some(ref c) = loc.country {
                    format!("{}, {}", loc.name, c)
                } else {
                    loc.name.clone()
                };
                location_label = Some(label);
                info(
                    config.silent,
                    &format!(
                        "Weather for: {} ({:.4}, {:.4})",
                        loc.name, loc.latitude, loc.longitude
                    ),
                );
            }
            Err(e) => {
                eprintln!("{}", e.user_friendly_message());
                std::process::exit(1);
            }
        }
    }

    // CLI Overrides
    if cli.auto_location {
        config.location.auto = true;
    }
    if cli.hide_location {
        config.location.hide = true;
    }
    if cli.hide_hud {
        config.hide_hud = true;
    }
    if cli.imperial {
        config.units = weather::WeatherUnits::imperial();
    }
    if cli.metric {
        config.units = weather::WeatherUnits::metric();
    }
    if cli.silent {
        config.silent = true;
    }

    // Auto-detect location if enabled
    if config.location.auto {
        info(config.silent, "Auto-detecting location...");
        match geolocation::detect_location().await {
            Ok(geo_loc) => {
                if let Some(city) = &geo_loc.city {
                    info(
                        config.silent,
                        &format!(
                            "Location detected: {} ({:.4}, {:.4})",
                            city, geo_loc.latitude, geo_loc.longitude
                        ),
                    );
                    if location_label.is_none() {
                        location_label = Some(city.clone());
                    }
                } else {
                    info(
                        config.silent,
                        &format!(
                            "Location detected: {:.4}, {:.4}",
                            geo_loc.latitude, geo_loc.longitude
                        ),
                    );
                }
                config.location.latitude = geo_loc.latitude;
                config.location.longitude = geo_loc.longitude;
            }
            Err(e) => {
                eprintln!("{}", e.user_friendly_message());
            }
        }
    }

    let mut renderer = match TerminalRenderer::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("\n{}\n", e.user_friendly_message());
            std::process::exit(1);
        }
    };

    if let Err(e) = renderer.init() {
        eprintln!("\n{}\n", e.user_friendly_message());
        std::process::exit(1);
    };

    let (term_width, term_height) = renderer.get_size();

    let mut app = app::App::new(
        &config,
        cli.simulate,
        cli.night,
        cli.leaves,
        term_width,
        term_height,
        location_label,
        cli.duration,
    );

    let result = tokio::select! {
        res = app.run(&mut renderer) => res,
        _ = tokio::signal::ctrl_c() => {
            Ok(())
        }
    };

    renderer.cleanup()?;

    if let Err(e) = result {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
