mod animation;
mod animation_manager;
mod app;
mod app_state;
mod cache;
mod cli;
mod config;
mod error;
mod geolocation;
mod render;
mod scene;
mod theme;
mod weather;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::Cli;
use config::Config;
use crossterm::{
    cursor, execute,
    style::ResetColor,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
use render::TerminalRenderer;
use std::{io, panic};
use theme::ThemeRegistry;

fn info(silent: bool, msg: &str) {
    if !silent {
        println!("{}", msg);
    }
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
            let err = cli::extract_simulate_missing_value(err);
            eprintln!("{}", err);
            eprintln!();
            cli::print_simulate_help();
            std::process::exit(1);
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
            eprintln!("\nFix or recreate it at:");
            eprintln!(
                "  Linux: ~/.config/weathr/config.toml (or $XDG_CONFIG_HOME/weathr/config.toml)"
            );
            eprintln!("  macOS: ~/Library/Application Support/weathr/config.toml");
            eprintln!("  Windows: %APPDATA%\\weathr\\config.toml");
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

    let lat_from_env = std::env::var(config::ENV_LATITUDE).is_ok();
    let lon_from_env = std::env::var(config::ENV_LONGITUDE).is_ok();
    if lat_from_env || lon_from_env {
        info(
            config.silent,
            &format!(
                "Location overridden via environment: ({:.4}, {:.4})",
                config.location.latitude, config.location.longitude
            ),
        );
    }

    if !config.location.auto
        && config.location.latitude == config::default_latitude()
        && config.location.longitude == config::default_longitude()
        && !lat_from_env
        && !lon_from_env
    {
        eprintln!("Warning: No location set, defaulting to Berlin (52.52, 13.41).");
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
                config.location.city = geo_loc.city;
            }
            Err(e) => {
                eprintln!("{}", e.user_friendly_message());
            }
        }
    }

    // Resolve city name via reverse geocoding when needed but not yet known
    if config.location.city.is_none()
        && !config.location.hide
        && matches!(
            config.location.display,
            config::LocationDisplay::City | config::LocationDisplay::Mixed
        )
    {
        info(config.silent, "Resolving city name...");
        if let Some(city) = geolocation::reverse_geocode(
            config.location.latitude,
            config.location.longitude,
            &config.location.city_name_language,
        )
        .await
        {
            info(config.silent, &format!("City resolved: {}", city));
            config.location.city = Some(city);
        }
    }

    let mut theme_registry = ThemeRegistry::new();
    let theme_id = config.normalized_theme();
    if theme_registry.set_active(theme_id).is_err() {
        eprintln!(
            "Warning: theme '{}' is not registered, falling back to 'default'.",
            theme_id
        );
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
        theme_registry,
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
