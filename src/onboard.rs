use std::fmt;
use std::time::Duration;

use console::Style;
use dialoguer::{Confirm, FuzzySelect, Input, Select};
use serde::Deserialize;

use crate::config::{Config, Location};
use crate::error::OnboardError;
use crate::weather::types::{PrecipitationUnit, TemperatureUnit, WeatherUnits, WindSpeedUnit};

const GEOCODING_API_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";

// ── Geocoding API types ──────────────────────────────────────────────

#[derive(Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Deserialize)]
struct GeocodingResult {
    name: String,
    latitude: f64,
    longitude: f64,
    country: Option<String>,
    admin1: Option<String>,
    population: Option<u64>,
    country_code: Option<String>,
}

impl fmt::Display for GeocodingResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = vec![self.name.clone()];

        if let Some(ref admin1) = self.admin1 {
            if *admin1 != self.name {
                parts.push(admin1.clone());
            }
        }

        if let Some(ref country) = self.country {
            parts.push(country.clone());
        } else if let Some(ref code) = self.country_code {
            parts.push(code.clone());
        }

        let location = parts.join(", ");
        let coords = format!("{:.4}, {:.4}", self.latitude, self.longitude);

        if let Some(pop) = self.population {
            if pop > 0 {
                return write!(f, "{location} ({coords}) - pop. {pop}");
            }
        }

        write!(f, "{location} ({coords})")
    }
}

// ── Styling helpers ──────────────────────────────────────────────────

fn print_banner() {
    let cyan = Style::new().cyan().bold();
    let dim = Style::new().dim();

    println!();
    println!(
        "{}",
        cyan.apply_to("┌───────────────────────────────────────┐")
    );
    println!(
        "{}",
        cyan.apply_to("│      Welcome to weathr setup!         │")
    );
    println!(
        "{}",
        cyan.apply_to("│  Let's configure your weather app.    │")
    );
    println!(
        "{}",
        cyan.apply_to("└───────────────────────────────────────┘")
    );
    println!();
    println!(
        "{}",
        dim.apply_to("  Tip: existing values are shown as defaults. Press Enter to keep them.")
    );
    println!();
}

fn print_section(title: &str) {
    let accent = Style::new().cyan().bold();
    let line = "─".repeat(40 - title.len().min(38));
    println!();
    println!("{}", accent.apply_to(format!("── {title} {line}")));
    println!();
}

fn print_success(config_path: &std::path::Path) {
    let green = Style::new().green().bold();
    let bold = Style::new().bold();

    println!();
    println!(
        "{}",
        green.apply_to("── All set! ────────────────────────────")
    );
    println!();
    println!("  Config saved to {}", bold.apply_to(config_path.display()));
    println!();
    println!("  Run {} to start!", green.apply_to("weathr"));
    println!();
}

fn print_error(msg: &str) {
    let red = Style::new().red().bold();
    println!("  {} {msg}", red.apply_to("Error:"));
}

fn current_hint(value: impl fmt::Display) -> String {
    Style::new()
        .dim()
        .apply_to(format!("[current: {value}]"))
        .to_string()
}

// ── Location method ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum LocationMethod {
    Coordinates,
    CitySearch,
    AutoDetect,
}

const LOCATION_METHODS: &[LocationMethod] = &[
    LocationMethod::Coordinates,
    LocationMethod::CitySearch,
    LocationMethod::AutoDetect,
];

impl fmt::Display for LocationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationMethod::Coordinates => write!(f, "Enter coordinates (latitude/longitude)"),
            LocationMethod::CitySearch => write!(f, "Search by city name"),
            LocationMethod::AutoDetect => write!(f, "Use auto-detection (IP-based)"),
        }
    }
}

// ── Geocoding API ────────────────────────────────────────────────────

async fn search_cities(query: &str) -> Result<Vec<GeocodingResult>, OnboardError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| OnboardError::GeocodingError(crate::error::NetworkError::ClientCreation(e)))?;

    let url = url::Url::parse_with_params(
        GEOCODING_API_URL,
        &[("name", query), ("count", "10"), ("language", "en")],
    )
    .expect("static base URL should be valid");

    let response = client.get(url).send().await.map_err(|e| {
        OnboardError::GeocodingError(crate::error::NetworkError::from_reqwest(
            e,
            GEOCODING_API_URL,
            10,
        ))
    })?;

    let body: GeocodingResponse = response.json::<GeocodingResponse>().await.map_err(|e| {
        OnboardError::GeocodingError(crate::error::NetworkError::from_reqwest(
            e,
            GEOCODING_API_URL,
            10,
        ))
    })?;

    body.results
        .filter(|r: &Vec<GeocodingResult>| !r.is_empty())
        .ok_or_else(|| OnboardError::NoGeocodingResults(query.to_string()))
}

// ── Prompt helpers ───────────────────────────────────────────────────

fn prompt_location_method() -> Result<LocationMethod, OnboardError> {
    let items: Vec<String> = LOCATION_METHODS.iter().map(|m| m.to_string()).collect();

    let selection = Select::new()
        .with_prompt("How would you like to set your location?")
        .items(&items)
        .default(0)
        .interact_opt()
        .map_err(|e| OnboardError::PromptError(e.to_string()))?
        .ok_or(OnboardError::Cancelled)?;

    Ok(LOCATION_METHODS[selection])
}

fn prompt_latitude(current: f64) -> Result<f64, OnboardError> {
    Input::new()
        .with_prompt(format!("Latitude (-90 to 90) {}", current_hint(current)))
        .default(current)
        .validate_with(|input: &f64| {
            if *input >= -90.0 && *input <= 90.0 {
                Ok(())
            } else {
                Err("Latitude must be between -90 and 90")
            }
        })
        .interact()
        .map_err(|e| OnboardError::PromptError(e.to_string()))
}

fn prompt_longitude(current: f64) -> Result<f64, OnboardError> {
    Input::new()
        .with_prompt(format!("Longitude (-180 to 180) {}", current_hint(current)))
        .default(current)
        .validate_with(|input: &f64| {
            if *input >= -180.0 && *input <= 180.0 {
                Ok(())
            } else {
                Err("Longitude must be between -180 and 180")
            }
        })
        .interact()
        .map_err(|e| OnboardError::PromptError(e.to_string()))
}

fn prompt_city_name() -> Result<String, OnboardError> {
    Input::new()
        .with_prompt("City name")
        .validate_with(|input: &String| {
            if input.trim().len() >= 2 {
                Ok(())
            } else {
                Err("Please enter at least 2 characters")
            }
        })
        .interact_text()
        .map_err(|e| OnboardError::PromptError(e.to_string()))
}

enum CitySelection {
    Selected(usize),
    SearchAgain,
}

fn prompt_select_city(results: &[GeocodingResult]) -> Result<CitySelection, OnboardError> {
    let dim = Style::new().dim().italic();
    let mut items: Vec<String> = results.iter().map(|r| r.to_string()).collect();
    items.push(format!("{}", dim.apply_to("-- Search again --")));

    let selection = FuzzySelect::new()
        .with_prompt("Select a city")
        .items(&items)
        .default(0)
        .interact_opt()
        .map_err(|e| OnboardError::PromptError(e.to_string()))?
        .ok_or(OnboardError::Cancelled)?;

    if selection == results.len() {
        Ok(CitySelection::SearchAgain)
    } else {
        Ok(CitySelection::Selected(selection))
    }
}

fn yes_no(val: bool) -> &'static str {
    if val { "yes" } else { "no" }
}

fn prompt_auto_location(current: bool) -> Result<bool, OnboardError> {
    Confirm::new()
        .with_prompt(format!(
            "Enable automatic IP-based location detection? {}",
            current_hint(yes_no(current))
        ))
        .default(current)
        .interact_opt()
        .map_err(|e| OnboardError::PromptError(e.to_string()))?
        .ok_or(OnboardError::Cancelled)
}

fn prompt_hide_location(current: bool) -> Result<bool, OnboardError> {
    Confirm::new()
        .with_prompt(format!(
            "Hide location coordinates in the UI? {}",
            current_hint(yes_no(current))
        ))
        .default(current)
        .interact_opt()
        .map_err(|e| OnboardError::PromptError(e.to_string()))?
        .ok_or(OnboardError::Cancelled)
}

fn prompt_temperature_unit(current: TemperatureUnit) -> Result<TemperatureUnit, OnboardError> {
    let options = ["Celsius (°C)", "Fahrenheit (°F)"];
    let default = match current {
        TemperatureUnit::Celsius => 0,
        TemperatureUnit::Fahrenheit => 1,
    };

    let selection = Select::new()
        .with_prompt(format!(
            "Temperature unit {}",
            current_hint(match current {
                TemperatureUnit::Celsius => "celsius",
                TemperatureUnit::Fahrenheit => "fahrenheit",
            })
        ))
        .items(options)
        .default(default)
        .interact_opt()
        .map_err(|e| OnboardError::PromptError(e.to_string()))?
        .ok_or(OnboardError::Cancelled)?;

    Ok(match selection {
        0 => TemperatureUnit::Celsius,
        _ => TemperatureUnit::Fahrenheit,
    })
}

fn prompt_wind_speed_unit(current: WindSpeedUnit) -> Result<WindSpeedUnit, OnboardError> {
    let options = ["km/h", "m/s", "mph", "knots"];
    let default = match current {
        WindSpeedUnit::Kmh => 0,
        WindSpeedUnit::Ms => 1,
        WindSpeedUnit::Mph => 2,
        WindSpeedUnit::Kn => 3,
    };

    let selection = Select::new()
        .with_prompt(format!(
            "Wind speed unit {}",
            current_hint(match current {
                WindSpeedUnit::Kmh => "km/h",
                WindSpeedUnit::Ms => "m/s",
                WindSpeedUnit::Mph => "mph",
                WindSpeedUnit::Kn => "knots",
            })
        ))
        .items(options)
        .default(default)
        .interact_opt()
        .map_err(|e| OnboardError::PromptError(e.to_string()))?
        .ok_or(OnboardError::Cancelled)?;

    Ok(match selection {
        0 => WindSpeedUnit::Kmh,
        1 => WindSpeedUnit::Ms,
        2 => WindSpeedUnit::Mph,
        _ => WindSpeedUnit::Kn,
    })
}

fn prompt_precipitation_unit(
    current: PrecipitationUnit,
) -> Result<PrecipitationUnit, OnboardError> {
    let options = ["mm", "inch"];
    let default = match current {
        PrecipitationUnit::Mm => 0,
        PrecipitationUnit::Inch => 1,
    };

    let selection = Select::new()
        .with_prompt(format!(
            "Precipitation unit {}",
            current_hint(match current {
                PrecipitationUnit::Mm => "mm",
                PrecipitationUnit::Inch => "inch",
            })
        ))
        .items(options)
        .default(default)
        .interact_opt()
        .map_err(|e| OnboardError::PromptError(e.to_string()))?
        .ok_or(OnboardError::Cancelled)?;

    Ok(match selection {
        0 => PrecipitationUnit::Mm,
        _ => PrecipitationUnit::Inch,
    })
}

fn prompt_hide_hud(current: bool) -> Result<bool, OnboardError> {
    Confirm::new()
        .with_prompt(format!(
            "Hide the HUD (status line)? {}",
            current_hint(yes_no(current))
        ))
        .default(current)
        .interact_opt()
        .map_err(|e| OnboardError::PromptError(e.to_string()))?
        .ok_or(OnboardError::Cancelled)
}

fn prompt_silent(current: bool) -> Result<bool, OnboardError> {
    Confirm::new()
        .with_prompt(format!(
            "Run silently (suppress non-error output)? {}",
            current_hint(yes_no(current))
        ))
        .default(current)
        .interact_opt()
        .map_err(|e| OnboardError::PromptError(e.to_string()))?
        .ok_or(OnboardError::Cancelled)
}

// ── Main onboarding flow ─────────────────────────────────────────────

pub async fn run() -> Result<(), OnboardError> {
    print_banner();

    // Phase 1: Resolve paths and load existing config
    let config_path = Config::get_config_path()?;
    let config_dir = Config::get_config_dir()?;

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).map_err(|e| OnboardError::CreateDirError {
            path: config_dir.display().to_string(),
            source: e,
        })?;
    }

    let mut config = if config_path.exists() {
        let dim = Style::new().dim();
        println!(
            "  {}",
            dim.apply_to(format!(
                "Found existing config at {}",
                config_path.display()
            ))
        );
        Config::load_from_path(&config_path).unwrap_or_default()
    } else {
        Config::default()
    };

    // Phase 2: Interactive prompts

    // ── Location ─────────────────────────────────────────────
    print_section("Location");

    let method = prompt_location_method()?;

    match method {
        LocationMethod::Coordinates => {
            config.location.latitude = prompt_latitude(config.location.latitude)?;
            config.location.longitude = prompt_longitude(config.location.longitude)?;
            config.location.auto = prompt_auto_location(config.location.auto)?;
        }
        LocationMethod::CitySearch => loop {
            let city = prompt_city_name()?;

            let searching = Style::new().dim();
            println!(
                "  {}",
                searching.apply_to(format!("Searching for \"{city}\"..."))
            );

            match search_cities(&city).await {
                Ok(results) => match prompt_select_city(&results)? {
                    CitySelection::Selected(idx) => {
                        let selected = &results[idx];

                        config.location = Location {
                            latitude: selected.latitude,
                            longitude: selected.longitude,
                            auto: false,
                            hide: config.location.hide,
                            ..Default::default()
                        };

                        let green = Style::new().green();
                        println!(
                            "  {} {:.4}, {:.4}",
                            green.apply_to("Selected:"),
                            selected.latitude,
                            selected.longitude,
                        );
                        break;
                    }
                    CitySelection::SearchAgain => {
                        println!();
                        continue;
                    }
                },
                Err(OnboardError::NoGeocodingResults(query)) => {
                    print_error(&format!(
                        "No results found for \"{query}\". Try a different search."
                    ));
                    continue;
                }
                Err(e) => {
                    print_error(&format!("Search failed: {e}. Using current coordinates."));
                    break;
                }
            }
        },
        LocationMethod::AutoDetect => {
            config.location.auto = true;
        }
    }

    config.location.hide = prompt_hide_location(config.location.hide)?;

    // ── Units ────────────────────────────────────────────────
    print_section("Units");

    config.units = WeatherUnits {
        temperature: prompt_temperature_unit(config.units.temperature)?,
        wind_speed: prompt_wind_speed_unit(config.units.wind_speed)?,
        precipitation: prompt_precipitation_unit(config.units.precipitation)?,
    };

    // ── Display ──────────────────────────────────────────────
    print_section("Display");

    config.hide_hud = prompt_hide_hud(config.hide_hud)?;
    config.silent = prompt_silent(config.silent)?;

    // Phase 3: Validate and save
    if let Err(e) = config.validate() {
        print_error(&format!("Invalid config: {e}"));
        return Err(OnboardError::Config(e));
    }

    config.save(&config_path)?;

    print_success(&config_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── GeocodingResult Display ──────────────────────────────

    #[test]
    fn test_geocoding_result_display_full() {
        let result = GeocodingResult {
            name: "Berlin".to_string(),
            latitude: 52.5244,
            longitude: 13.4105,
            country: Some("Germany".to_string()),
            admin1: Some("Berlin".to_string()),
            population: Some(3_426_354),
            country_code: Some("DE".to_string()),
        };
        // admin1 == name, so admin1 is omitted
        assert_eq!(
            result.to_string(),
            "Berlin, Germany (52.5244, 13.4105) - pop. 3426354"
        );
    }

    #[test]
    fn test_geocoding_result_display_different_admin1() {
        let result = GeocodingResult {
            name: "Munich".to_string(),
            latitude: 48.1351,
            longitude: 11.5820,
            country: Some("Germany".to_string()),
            admin1: Some("Bavaria".to_string()),
            population: Some(1_471_508),
            country_code: Some("DE".to_string()),
        };
        assert_eq!(
            result.to_string(),
            "Munich, Bavaria, Germany (48.1351, 11.5820) - pop. 1471508"
        );
    }

    #[test]
    fn test_geocoding_result_display_no_population() {
        let result = GeocodingResult {
            name: "Smallville".to_string(),
            latitude: 40.0,
            longitude: -80.0,
            country: Some("United States".to_string()),
            admin1: Some("Kansas".to_string()),
            population: None,
            country_code: None,
        };
        assert_eq!(
            result.to_string(),
            "Smallville, Kansas, United States (40.0000, -80.0000)"
        );
    }

    #[test]
    fn test_geocoding_result_display_zero_population() {
        let result = GeocodingResult {
            name: "Nowhere".to_string(),
            latitude: 0.0,
            longitude: 0.0,
            country: Some("Country".to_string()),
            admin1: None,
            population: Some(0),
            country_code: None,
        };
        // pop == 0 is treated as "no population data"
        assert_eq!(result.to_string(), "Nowhere, Country (0.0000, 0.0000)");
    }

    #[test]
    fn test_geocoding_result_display_country_code_fallback() {
        let result = GeocodingResult {
            name: "Place".to_string(),
            latitude: 1.0,
            longitude: 2.0,
            country: None,
            admin1: None,
            population: None,
            country_code: Some("XX".to_string()),
        };
        assert_eq!(result.to_string(), "Place, XX (1.0000, 2.0000)");
    }

    #[test]
    fn test_geocoding_result_display_minimal() {
        let result = GeocodingResult {
            name: "Place".to_string(),
            latitude: 1.0,
            longitude: 2.0,
            country: None,
            admin1: None,
            population: None,
            country_code: None,
        };
        assert_eq!(result.to_string(), "Place (1.0000, 2.0000)");
    }

    // ── yes_no ───────────────────────────────────────────────

    #[test]
    fn test_yes_no() {
        assert_eq!(yes_no(true), "yes");
        assert_eq!(yes_no(false), "no");
    }
}
