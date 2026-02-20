mod animation;
mod animation_manager;
mod app;
mod app_state;
mod cache;
mod config;
mod error;
mod geolocation;
mod render;
mod scene;
mod weather;

use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use config::Config;
use crossterm::{
    cursor, execute,
    style::ResetColor,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
use render::TerminalRenderer;
use std::{io, panic};

const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n\nWeather data provided by Open-Meteo.com (https://open-meteo.com/)\n",
    "Data licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)"
);

fn info(silent: bool, msg: &str) {
    if !silent {
        println!("{}", msg);
    }
}

const ABOUT: &str = concat!(
    "Terminal-based ASCII weather application\n\n",
    "Weather data provided by Open-Meteo.com (https://open-meteo.com/)\n",
    "Data licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)"
);

#[derive(Parser)]
#[command(version, long_version = LONG_VERSION, about = ABOUT, long_about = None)]
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

    #[arg(long, value_name = "SHELL", value_enum)]
    pub completions: Option<Shell>,
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

    if let Some(shell) = cli.completions {
        let mut cmd = Cli::command();
        let mut out = io::stdout();
        generate(shell, &mut cmd, "weathr", &mut out);
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
