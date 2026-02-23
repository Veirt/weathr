use clap::Parser;
use clap::builder::{PossibleValue, PossibleValuesParser};
use clap_complete::Shell;

use crate::weather::WeatherCondition;

const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n\nWeather data provided by Open-Meteo.com (https://open-meteo.com/)\n",
    "Data licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)\n\n",
    "Geocoding powered by Nominatim/OpenStreetMap (https://nominatim.openstreetmap.org/)\n",
    "Data \u{00a9} OpenStreetMap contributors, ODbL (https://www.openstreetmap.org/copyright)"
);

const ABOUT: &str = concat!(
    "Terminal-based ASCII weather application\n\n",
    "Weather data provided by Open-Meteo.com (https://open-meteo.com/)\n",
    "Data licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)\n\n",
    "Geocoding powered by Nominatim/OpenStreetMap (https://nominatim.openstreetmap.org/)\n",
    "Data \u{00a9} OpenStreetMap contributors, ODbL (https://www.openstreetmap.org/copyright)"
);

fn simulate_parser() -> PossibleValuesParser {
    PossibleValuesParser::new(
        WeatherCondition::ALL
            .iter()
            .map(|c| PossibleValue::new(c.as_str()).help(c.description())),
    )
}

#[derive(Parser)]
#[command(version, long_version = LONG_VERSION, about = ABOUT, long_about = None)]
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "CONDITION",
        value_parser = simulate_parser(),
        help = "Simulate weather condition (clear, rain, drizzle, snow, etc.)"
    )]
    pub simulate: Option<String>,

    #[arg(
        short,
        long,
        help = "Simulate night time (for testing moon, stars, fireflies)"
    )]
    pub night: bool,

    #[arg(short, long, help = "Enable falling autumn leaves")]
    pub leaves: bool,

    #[arg(long, help = "Auto-detect location via IP (uses ipinfo.io)")]
    pub auto_location: bool,

    #[arg(long, help = "Hide location coordinates in UI")]
    pub hide_location: bool,

    #[arg(long, help = "Hide HUD (status line)")]
    pub hide_hud: bool,

    #[arg(
        long,
        conflicts_with = "metric",
        help = "Use imperial units (°F, mph, inch)"
    )]
    pub imperial: bool,

    #[arg(
        long,
        conflicts_with = "imperial",
        help = "Use metric units (°C, km/h, mm)"
    )]
    pub metric: bool,

    #[arg(long, help = "Run silently (suppress non-error output)")]
    pub silent: bool,

    #[arg(long, value_name = "SHELL", value_enum)]
    pub completions: Option<Shell>,

    #[cfg(target_os = "windows")]
    #[arg(long, help = "Spawn as a borderless fullscreen window")]
    pub fullscreen: bool,

    #[cfg(target_os = "windows")]
    #[arg(long, help = "Don't respawn into conhost when --fullscreen is enabled")]
    pub conhost: bool,
}

pub fn extract_simulate_missing_value(err: clap::Error) -> clap::Error {
    let msg = err.to_string();
    if msg.contains("--simulate") && msg.contains("value is required") {
        err
    } else {
        err.exit()
    }
}

pub fn print_simulate_help() {
    let mut current_group = "";

    eprintln!("Available weather conditions:");
    for condition in WeatherCondition::ALL {
        let group = condition.group();
        if group != current_group {
            eprintln!();
            eprintln!("  {}:", group);
            current_group = group;
        }
        eprintln!(
            "    {:<18} - {}",
            condition.as_str(),
            condition.description()
        );
    }

    eprintln!();
    eprintln!("Examples:");
    eprintln!("  weathr --simulate rain");
    eprintln!("  weathr --simulate snow --night");
    eprintln!("  weathr -s thunderstorm -n");
}
