use crate::animation_manager::AnimationManager;
use crate::app_state::AppState;
use crate::config::{Config, Provider};
use crate::error::WeatherError;
use crate::render::TerminalRenderer;
use crate::scene::WorldScene;
use crate::weather::provider::WeatherProvider;
use crate::weather::provider::met_office::{MetOfficeProvider, MetOfficeProviderConfig};
use crate::weather::{
    OpenMeteoProvider, WeatherClient, WeatherCondition, WeatherData, WeatherLocation,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use serde::Deserialize;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

const REFRESH_INTERVAL: Duration = Duration::from_secs(300);
const INPUT_POLL_FPS: u64 = 30;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / INPUT_POLL_FPS);

fn generate_offline_weather(rng: &mut impl rand::Rng) -> WeatherData {
    use chrono::{Local, Timelike};
    use rand::RngExt;

    let now = Local::now();
    let hour = now.hour();
    let is_day = (6..18).contains(&hour);

    let conditions = [
        WeatherCondition::Clear,
        WeatherCondition::PartlyCloudy,
        WeatherCondition::Cloudy,
        WeatherCondition::Rain,
    ];

    let condition = conditions[rng.random_range(0..conditions.len())];

    WeatherData {
        condition,
        temperature: rng.random_range(10.0..25.0),
        apparent_temperature: rng.random_range(10.0..25.0),
        humidity: rng.random_range(40.0..80.0),
        precipitation: if condition.is_raining() {
            rng.random_range(1.0..5.0)
        } else {
            0.0
        },
        wind_speed: rng.random_range(5.0..15.0),
        wind_direction: rng.random_range(0.0..360.0),
        cloud_cover: rng.random_range(20.0..80.0),
        pressure: rng.random_range(1000.0..1020.0),
        visibility: Some(10000.0),
        is_day,
        moon_phase: Some(0.5),
        timestamp: now.format("%Y-%m-%dT%H:%M:%S").to_string(),
        attribution: "".to_string(),
    }
}

pub struct App {
    state: AppState,
    animations: AnimationManager,
    scene: WorldScene,
    weather_receiver: mpsc::Receiver<Result<WeatherData, WeatherError>>,
    hide_hud: bool,
}

impl App {
    pub fn new(
        config: &Config,
        simulate_condition: Option<String>,
        simulate_night: bool,
        show_leaves: bool,
        term_width: u16,
        term_height: u16,
    ) -> Self {
        let location = WeatherLocation {
            latitude: config.location.latitude,
            longitude: config.location.longitude,
            elevation: None,
        };

        let mut state = AppState::new(
            location,
            config.location.city.clone(),
            config.location.display,
            config.location.hide,
            config.units,
        );
        let mut animations = AnimationManager::new(term_width, term_height, show_leaves);
        let scene = WorldScene::new(term_width, term_height);

        let (tx, rx) = mpsc::channel(1);

        if let Some(ref condition_str) = simulate_condition {
            let simulated_condition =
                condition_str
                    .parse::<WeatherCondition>()
                    .unwrap_or_else(|e| {
                        eprintln!("{}", e);
                        WeatherCondition::Clear
                    });

            let weather = WeatherData {
                condition: simulated_condition,
                temperature: 20.0,
                apparent_temperature: 19.0,
                humidity: 65.0,
                precipitation: if simulated_condition.is_raining() {
                    2.5
                } else {
                    0.0
                },
                wind_speed: if simulated_condition.is_thunderstorm() {
                    45.0
                } else {
                    10.0
                },
                wind_direction: 225.0,
                cloud_cover: 50.0,
                pressure: 1013.0,
                visibility: Some(10000.0),
                is_day: !simulate_night,
                moon_phase: Some(0.5),
                timestamp: "simulated".to_string(),
                attribution: "".to_string(),
            };

            let rain_intensity = weather.condition.rain_intensity();
            let snow_intensity = weather.condition.snow_intensity();

            let wind_speed = weather.wind_speed;
            let wind_direction = weather.wind_direction;

            state.update_weather(weather);
            animations.update_rain_intensity(rain_intensity);
            animations.update_snow_intensity(snow_intensity);
            animations.update_wind(wind_speed as f32, wind_direction as f32);
        } else {

            let provider = config.provider.keys().next().cloned().unwrap_or(Provider::default()); // Pick the first available provider, or default

            let provider: Arc<dyn WeatherProvider> = match provider {
                Provider::OpenMeteo => Arc::new(OpenMeteoProvider::new()),
                Provider::MetOffice => {
                    let provider_config = {
                        if let Some(provider_config) = config.provider.get(&provider) {
                            MetOfficeProviderConfig::deserialize(provider_config.clone()).unwrap()
                        } else { MetOfficeProviderConfig::default() }
                    };

                    Arc::new(MetOfficeProvider::new(provider_config).unwrap())
                },
            };

            let weather_client = WeatherClient::new(provider, REFRESH_INTERVAL);
            let units = config.units;

            tokio::spawn(async move {
                loop {
                    let result = weather_client.get_current_weather(&location, &units).await;
                    if tx.send(result).await.is_err() {
                        break;
                    }
                    tokio::time::sleep(REFRESH_INTERVAL).await;
                }
            });
        }

        Self {
            state,
            animations,
            scene,
            weather_receiver: rx,
            hide_hud: config.hide_hud,
        }
    }

    pub async fn run(&mut self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        let mut rng = rand::rng();
        let mut attribution= "Awaiting weather data".to_string();
        loop {
            match self.weather_receiver.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(weather) => {
                            let rain_intensity = weather.condition.rain_intensity();
                            let snow_intensity = weather.condition.snow_intensity();
                            let fog_intensity = weather.condition.fog_intensity();
                            let wind_speed = weather.wind_speed;
                            let wind_direction = weather.wind_direction;
                            attribution = weather.attribution.clone();

                            self.state.update_weather(weather);
                            self.animations.update_rain_intensity(rain_intensity);
                            self.animations.update_snow_intensity(snow_intensity);
                            self.animations.update_fog_intensity(fog_intensity);
                            self.animations
                                .update_wind(wind_speed as f32, wind_direction as f32);

                        }
                        Err(error) => {
                            let error_msg = match &error {
                                WeatherError::Network(net_err) => net_err.user_friendly_message(),
                                _ => format!("Failed to fetch weather: {}", error),
                            };

                            if self.state.current_weather.is_none() {
                                attribution = format!("Provider failed with {error_msg} - Simulating");
                                let offline_weather = generate_offline_weather(&mut rng);
                                let rain_intensity = offline_weather.condition.rain_intensity();
                                let snow_intensity = offline_weather.condition.snow_intensity();
                                let fog_intensity = offline_weather.condition.fog_intensity();
                                let wind_speed = offline_weather.wind_speed;
                                let wind_direction = offline_weather.wind_direction;

                                self.state.update_weather(offline_weather);
                                self.state.set_offline_mode(true);
                                self.animations.update_rain_intensity(rain_intensity);
                                self.animations.update_snow_intensity(snow_intensity);
                                self.animations.update_fog_intensity(fog_intensity);
                                self.animations
                                    .update_wind(wind_speed as f32, wind_direction as f32);
                            } else {
                                self.state.set_offline_mode(true);
                                attribution = format!("Provider failed with {error_msg}");
                            }
                        }
                    }
                },
                Err(e) => {
                    if e == mpsc::error::TryRecvError::Disconnected {
                        attribution = "Provider failed".to_string();
                    }
                },
            }

            renderer.clear()?;

            let (term_width, term_height) = renderer.get_size();

            self.animations.render_background(
                renderer,
                &self.state.weather_conditions,
                &self.state,
                term_width,
                term_height,
                &mut rng,
            )?;

            self.scene
                .render(renderer, &self.state.weather_conditions)?;

            self.animations.render_chimney_smoke(
                renderer,
                &self.state.weather_conditions,
                term_width,
                term_height,
                &mut rng,
            )?;

            self.animations.render_foreground(
                renderer,
                &self.state.weather_conditions,
                term_width,
                term_height,
                &mut rng,
            )?;

            self.state.update_loading_animation();
            self.state.update_cached_info();

            if !self.hide_hud {
                renderer.render_line_colored(
                    2,
                    1,
                    &self.state.cached_weather_info,
                    crossterm::style::Color::Cyan,
                )?;
            }

            // Render attribution - Required by some providers this undoes commit: 55a3dbf84ec70de3614f3b8a044f15d488e9d149
            let attribution_x = if term_width > attribution.len() as u16 {
                term_width - attribution.len() as u16 - 2
            } else {
                0
            };
            let attribution_y = if term_height > 0 { term_height - 1 } else { 0 };
            renderer.render_line_colored(
                attribution_x,
                attribution_y,
                &attribution,
                crossterm::style::Color::DarkGrey,
            )?;

            renderer.flush()?;

            if event::poll(FRAME_DURATION)? {
                match event::read()? {
                    Event::Resize(width, height) => {
                        renderer.manual_resize(width, height)?;
                    }
                    Event::Key(key_event) => match key_event.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        KeyCode::Char('c')
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            let (term_width, term_height) = renderer.get_size();
            self.scene.update_size(term_width, term_height);

            self.animations
                .update_sunny_animation(&self.state.weather_conditions);
        }

        Ok(())
    }
}
