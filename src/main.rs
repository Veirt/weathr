mod config;
mod display;
mod render;
mod animation;
mod weather;

use animation::{raindrops::RaindropSystem, sunny::SunnyAnimation, thunderstorm::ThunderstormSystem, AnimationController};
use clap::Parser;
use config::Config;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use display::AsciiDisplay;
use render::TerminalRenderer;
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use weather::{OpenMeteoProvider, WeatherClient, WeatherCondition, WeatherData, WeatherLocation, WeatherUnits};

const REFRESH_INTERVAL: Duration = Duration::from_secs(300);
const FRAME_DELAY: Duration = Duration::from_millis(500);

#[derive(Parser)]
#[command(version, about = "Terminal-based ASCII weather application", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "CONDITION", help = "Simulate weather condition (clear, rain, drizzle, snow, etc.)")]
    simulate: Option<String>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            eprintln!("\nContinuing with default location (Berlin: 52.52°N, 13.41°E)");
            eprintln!("\nTo customize, create a config file at:");
            eprintln!("  $XDG_CONFIG_HOME/weathr/config.toml");
            eprintln!("  or ~/.config/weathr/config.toml");
            eprintln!("\nExample config.toml:");
            eprintln!("  [location]");
            eprintln!("  latitude = 52.52");
            eprintln!("  longitude = 13.41");
            eprintln!();
            Config::default()
        }
    };

    let mut renderer = TerminalRenderer::new()?;
    renderer.init()?;

    let result = run_app(&config, &mut renderer, cli.simulate).await;

    renderer.cleanup()?;

    result
}

async fn run_app(config: &Config, renderer: &mut TerminalRenderer, simulate_condition: Option<String>) -> io::Result<()> {
    let house = AsciiDisplay::render_house();
    let sunny_animation = SunnyAnimation::new();
    let mut animation_controller = AnimationController::new();
    
    let provider = Arc::new(OpenMeteoProvider::new());
    let weather_client = WeatherClient::new(provider, Duration::from_secs(300));
    
    let location = WeatherLocation {
        latitude: config.location.latitude,
        longitude: config.location.longitude,
        elevation: None,
    };
    let units = WeatherUnits::default();
    
    let mut last_update = Instant::now();
    let mut last_frame_time = Instant::now();
    let mut current_weather = None;
    let mut weather_error: Option<String> = None;
    let mut is_raining = false;
    let mut is_thunderstorm = false;
    let (term_width, term_height) = renderer.get_size();
    let mut raindrop_system = RaindropSystem::new(term_width, term_height);
    let mut thunderstorm_system = ThunderstormSystem::new(term_width, term_height);
    
    if let Some(ref condition_str) = simulate_condition {
        let simulated_condition = parse_weather_condition(condition_str);
        is_thunderstorm = matches!(
            simulated_condition,
            WeatherCondition::Thunderstorm | WeatherCondition::ThunderstormHail
        );
        is_raining = !is_thunderstorm && matches!(
            simulated_condition,
            WeatherCondition::Drizzle | WeatherCondition::Rain | 
            WeatherCondition::RainShowers | WeatherCondition::FreezingRain
        );
        current_weather = Some(WeatherData {
            condition: simulated_condition,
            temperature: 20.0,
            apparent_temperature: 19.0,
            humidity: 65.0,
            precipitation: if matches!(simulated_condition, WeatherCondition::Rain | WeatherCondition::Drizzle | WeatherCondition::RainShowers) { 2.5 } else { 0.0 },
            wind_speed: 10.0,
            wind_direction: 180.0,
            cloud_cover: 50.0,
            pressure: 1013.0,
            visibility: Some(10000.0),
            is_day: true,
            timestamp: "simulated".to_string(),
        });
    }

    loop {
        if simulate_condition.is_none() && (current_weather.is_none() || last_update.elapsed() >= REFRESH_INTERVAL) {
            match weather_client.get_current_weather(&location, &units).await {
                Ok(weather) => {
                    is_thunderstorm = matches!(
                        weather.condition,
                        WeatherCondition::Thunderstorm | WeatherCondition::ThunderstormHail
                    );
                    is_raining = !is_thunderstorm && matches!(
                        weather.condition,
                        WeatherCondition::Drizzle | WeatherCondition::Rain | 
                        WeatherCondition::RainShowers | WeatherCondition::FreezingRain
                    );
                    current_weather = Some(weather);
                    weather_error = None;
                }
                Err(e) => {
                    weather_error = Some(format!("Error fetching weather: {}", e));
                }
            }
            last_update = Instant::now();
        }

        renderer.update_size()?;
        let (term_width, term_height) = renderer.get_size();
        
        renderer.clear()?;

        let condition_text = if let Some(ref weather) = current_weather {
            match weather.condition {
                WeatherCondition::Clear => "Clear",
                WeatherCondition::PartlyCloudy => "Partly Cloudy",
                WeatherCondition::Cloudy => "Cloudy",
                WeatherCondition::Overcast => "Overcast",
                WeatherCondition::Fog => "Fog",
                WeatherCondition::Drizzle => "Drizzle",
                WeatherCondition::Rain => "Rain",
                WeatherCondition::FreezingRain => "Freezing Rain",
                WeatherCondition::Snow => "Snow",
                WeatherCondition::SnowGrains => "Snow Grains",
                WeatherCondition::RainShowers => "Rain Showers",
                WeatherCondition::SnowShowers => "Snow Showers",
                WeatherCondition::Thunderstorm => "Thunderstorm",
                WeatherCondition::ThunderstormHail => "Thunderstorm with Hail",
            }
        } else {
            "Loading..."
        };

        let weather_info = if let Some(ref error) = weather_error {
            format!("{} | Location: {:.2}°N, {:.2}°E | Press 'q' to quit", 
                error, location.latitude, location.longitude)
        } else if let Some(ref weather) = current_weather {
            format!("Weather: {} | Temp: {:.1}°C | Location: {:.2}°N, {:.2}°E | Press 'q' to quit",
                condition_text, weather.temperature, location.latitude, location.longitude)
        } else {
            format!("Weather: Loading... | Location: {:.2}°N, {:.2}°E | Press 'q' to quit",
                location.latitude, location.longitude)
        };
        
        renderer.render_line_colored(
            2,
            1,
            &weather_info,
            crossterm::style::Color::Cyan,
        )?;

        if is_thunderstorm {
            thunderstorm_system.update(term_width, term_height);
            thunderstorm_system.render(renderer)?;
        } else if is_raining {
            raindrop_system.update(term_width, term_height);
            raindrop_system.render(renderer)?;
        } else {
            let animation_y = if term_height > 20 { 3 } else { 2 };
            animation_controller.render_frame(renderer, &sunny_animation, animation_y)?;
        }

        let house_y = if term_height > 20 { 10 } else { 9 };
        let house_strings: Vec<String> = house.iter().map(|s| s.to_string()).collect();
        renderer.render_centered(&house_strings, house_y)?;

        renderer.flush()?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        break
                    }
                    _ => {}
                }
            }
        }

        if last_frame_time.elapsed() >= FRAME_DELAY {
            if !is_raining && !is_thunderstorm {
                animation_controller.next_frame(&sunny_animation);
            }
            last_frame_time = Instant::now();
        }
    }

    Ok(())
}

fn parse_weather_condition(input: &str) -> WeatherCondition {
    match input.to_lowercase().as_str() {
        "clear" | "sunny" => WeatherCondition::Clear,
        "partly-cloudy" | "partly_cloudy" | "partlycloudy" => WeatherCondition::PartlyCloudy,
        "cloudy" => WeatherCondition::Cloudy,
        "overcast" => WeatherCondition::Overcast,
        "fog" | "foggy" => WeatherCondition::Fog,
        "drizzle" => WeatherCondition::Drizzle,
        "rain" | "rainy" => WeatherCondition::Rain,
        "freezing-rain" | "freezing_rain" | "freezingrain" => WeatherCondition::FreezingRain,
        "snow" | "snowy" => WeatherCondition::Snow,
        "snow-grains" | "snow_grains" | "snowgrains" => WeatherCondition::SnowGrains,
        "rain-showers" | "rain_showers" | "rainshowers" | "showers" => WeatherCondition::RainShowers,
        "snow-showers" | "snow_showers" | "snowshowers" => WeatherCondition::SnowShowers,
        "thunderstorm" | "thunder" => WeatherCondition::Thunderstorm,
        "thunderstorm-hail" | "thunderstorm_hail" | "hail" => WeatherCondition::ThunderstormHail,
        _ => {
            eprintln!("Unknown weather condition '{}', defaulting to Clear", input);
            WeatherCondition::Clear
        }
    }
}
