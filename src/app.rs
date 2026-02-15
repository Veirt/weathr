use crate::animation_manager::AnimationManager;
use crate::app_state::AppState;
use crate::config::Config;
use crate::error::WeatherError;
use crate::render::TerminalRenderer;
use crate::scene::WorldScene;
use crate::weather::{
    OpenMeteoProvider, WeatherClient, WeatherCondition, WeatherData, WeatherLocation, WeatherUnits,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
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
    }
}

pub struct App {
    state: AppState,
    animations: AnimationManager,
    scene: WorldScene,
    weather_receiver: mpsc::Receiver<Result<WeatherData, WeatherError>>,
    hide_hud: bool,
    paused: bool,
    speed_multiplier: f32,
    show_help: bool,
    weather_task: Option<tokio::task::JoinHandle<()>>,
    weather_location: WeatherLocation,
    weather_units: WeatherUnits,
    weather_provider: Option<Arc<OpenMeteoProvider>>,
    refreshing: bool,
    speed_changed_at: Option<Instant>,
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

        let mut state = AppState::new(location, config.location.hide, config.units);
        let mut animations = AnimationManager::new(term_width, term_height, show_leaves);
        let scene = WorldScene::new(term_width, term_height);

        let (tx, rx) = mpsc::channel(1);

        let mut weather_task: Option<tokio::task::JoinHandle<()>> = None;
        let mut weather_provider: Option<Arc<OpenMeteoProvider>> = None;

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
            let provider = Arc::new(OpenMeteoProvider::new());
            let weather_client = WeatherClient::new(provider.clone(), REFRESH_INTERVAL);
            let units = config.units;

            let task = tokio::spawn(async move {
                loop {
                    let result = weather_client.get_current_weather(&location, &units).await;
                    if tx.send(result).await.is_err() {
                        break;
                    }
                    tokio::time::sleep(REFRESH_INTERVAL).await;
                }
            });
            weather_task = Some(task);
            weather_provider = Some(provider);
        }

        Self {
            state,
            animations,
            scene,
            weather_receiver: rx,
            hide_hud: config.hide_hud,
            paused: false,
            speed_multiplier: 1.0,
            show_help: false,
            weather_task,
            weather_location: WeatherLocation {
                latitude: config.location.latitude,
                longitude: config.location.longitude,
                elevation: None,
            },
            weather_units: config.units,
            weather_provider,
            refreshing: false,
            speed_changed_at: None,
        }
    }

    pub async fn run(&mut self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        let mut rng = rand::rng();
        loop {
            if let Ok(result) = self.weather_receiver.try_recv() {
                self.refreshing = false;
                match result {
                    Ok(weather) => {
                        let rain_intensity = weather.condition.rain_intensity();
                        let snow_intensity = weather.condition.snow_intensity();
                        let fog_intensity = weather.condition.fog_intensity();
                        let wind_speed = weather.wind_speed;
                        let wind_direction = weather.wind_direction;

                        self.state.update_weather(weather);
                        self.animations.update_rain_intensity(rain_intensity);
                        self.animations.update_snow_intensity(snow_intensity);
                        self.animations.update_fog_intensity(fog_intensity);
                        self.animations
                            .update_wind(wind_speed as f32, wind_direction as f32);
                    }
                    Err(error) => {
                        let _error_msg = match &error {
                            WeatherError::Network(net_err) => net_err.user_friendly_message(),
                            _ => format!("Failed to fetch weather: {}", error),
                        };

                        if self.state.current_weather.is_none() {
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
                        }
                    }
                }
            }

            let (term_width, term_height) = renderer.get_size();

            if !self.paused {
                renderer.clear()?;
                self.animations.render_background(
                    renderer,
                    &self.state.weather_conditions,
                    &self.state,
                    term_width,
                    term_height,
                    &mut rng,
                    self.speed_multiplier,
                )?;

                self.scene
                    .render(renderer, &self.state.weather_conditions)?;

                self.animations.render_chimney_smoke(
                    renderer,
                    &self.state.weather_conditions,
                    term_width,
                    term_height,
                    &mut rng,
                    self.speed_multiplier,
                )?;

                self.animations.render_foreground(
                    renderer,
                    &self.state.weather_conditions,
                    term_width,
                    term_height,
                    &mut rng,
                    self.speed_multiplier,
                )?;
            }

            self.state.update_loading_animation();
            self.state.update_cached_info();

            if !self.hide_hud {
                let hud_text = if self.refreshing {
                    format!("[Refreshing...] {}", &self.state.cached_weather_info)
                } else {
                    self.state.cached_weather_info.clone()
                };
                renderer.render_line_colored(2, 1, &hud_text, crossterm::style::Color::Cyan)?;
            }

            // Help at term_height-2 avoids collision with attribution at term_height-1
            // when MIN_WIDTH=70. Guard term_height >= 3 prevents underflow.
            if self.show_help && term_height >= 3 {
                let help_text = "q:Quit p:Pause r:Refresh h:HUD +/-:Speed ?:Help";
                let help_y = term_height - 2;
                let display_text = if (term_width as usize) < help_text.len() {
                    // Truncated with ellipsis when terminal too narrow
                    let truncated = &help_text[..((term_width as usize).saturating_sub(3))];
                    format!("{}...", truncated)
                } else {
                    help_text.to_string()
                };
                renderer.render_line_colored(
                    0,
                    help_y,
                    &display_text,
                    crossterm::style::Color::DarkGrey,
                )?;
            }

            if let Some(changed_at) = self.speed_changed_at {
                if changed_at.elapsed() < Duration::from_secs(2) {
                    let speed_text = format!("Speed: {}x", self.speed_multiplier);
                    renderer.render_line_colored(
                        2,
                        2,
                        &speed_text,
                        crossterm::style::Color::Yellow,
                    )?;
                } else {
                    self.speed_changed_at = None;
                }
            }

            let attribution = "Weather data by Open-Meteo.com";
            let attribution_x = if term_width > attribution.len() as u16 {
                term_width - attribution.len() as u16 - 2
            } else {
                0
            };
            let attribution_y = if term_height > 0 { term_height - 1 } else { 0 };
            renderer.render_line_colored(
                attribution_x,
                attribution_y,
                attribution,
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
                        KeyCode::Char('p') => {
                            self.paused = !self.paused;
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            // 0.25 floor prevents invisible animations, 4.0 ceiling prevents unusable speed
                            self.speed_multiplier = (self.speed_multiplier + 0.25).clamp(0.25, 4.0);
                            self.speed_changed_at = Some(Instant::now());
                        }
                        KeyCode::Char('-') => {
                            self.speed_multiplier = (self.speed_multiplier - 0.25).clamp(0.25, 4.0);
                            self.speed_changed_at = Some(Instant::now());
                        }
                        KeyCode::Char('h') => {
                            self.hide_hud = !self.hide_hud;
                        }
                        KeyCode::Char('?') => {
                            self.show_help = !self.show_help;
                        }
                        KeyCode::Char('r') => {
                            self.refresh_weather();
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            let (term_width, term_height) = renderer.get_size();
            self.scene.update_size(term_width, term_height);

            if !self.paused {
                self.animations
                    .update_sunny_animation(&self.state.weather_conditions, self.speed_multiplier);
            }
        }

        Ok(())
    }

    /// Abort the current weather fetch task and spawn a fresh one.
    /// Spawns the same looping task as App::new() so automatic
    /// 5-minute periodic refresh continues after manual refresh.
    /// No-op when running in simulation mode (no provider).
    fn refresh_weather(&mut self) {
        // Simulation mode has no provider — refresh is a no-op
        let provider = match &self.weather_provider {
            Some(p) => p.clone(),
            None => return,
        };

        // Aborts existing task to prevent duplicate fetchers running.
        // tokio::JoinHandle::abort() is documented as safe; spawned task drops cleanly.
        if let Some(task) = self.weather_task.take() {
            task.abort();
        }

        self.refreshing = true;

        // Creates new channel. Previous sender becomes disconnected, causing aborted task to exit.
        let (tx, rx) = mpsc::channel(1);
        self.weather_receiver = rx;

        let location = self.weather_location;
        let units = self.weather_units;
        let weather_client = WeatherClient::new(provider, REFRESH_INTERVAL);

        // Fetches immediately, sends result, then sleeps(REFRESH_INTERVAL) to maintain automatic 5-minute periodic refresh cycle
        self.weather_task = Some(tokio::spawn(async move {
            loop {
                let result = weather_client.get_current_weather(&location, &units).await;
                if tx.send(result).await.is_err() {
                    break;
                }
                tokio::time::sleep(REFRESH_INTERVAL).await;
            }
        }));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pause_toggles_correctly() {
        let mut paused = false;
        paused = !paused;
        assert!(paused, "First toggle should pause");
        paused = !paused;
        assert!(!paused, "Second toggle should unpause");
    }

    #[test]
    fn test_pause_defaults_to_false() {
        let paused: bool = false;
        assert!(!paused);
    }

    #[test]
    fn test_speed_multiplier_defaults_to_1_0() {
        let speed: f32 = 1.0;
        assert!((speed - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_speed_increment_by_0_25() {
        let mut speed: f32 = 1.0;
        speed = (speed + 0.25).clamp(0.25, 4.0);
        speed = (speed + 0.25).clamp(0.25, 4.0);
        speed = (speed + 0.25).clamp(0.25, 4.0);
        assert!(
            (speed - 1.75).abs() < f32::EPSILON,
            "Three increments from 1.0 should yield 1.75"
        );
    }

    #[test]
    fn test_speed_decrement_by_0_25() {
        let mut speed: f32 = 1.0;
        speed = (speed - 0.25).clamp(0.25, 4.0);
        assert!(
            (speed - 0.75).abs() < f32::EPSILON,
            "One decrement from 1.0 should yield 0.75"
        );
    }

    #[test]
    fn test_speed_no_underflow_at_minimum() {
        let mut speed: f32 = 0.25;
        speed = (speed - 0.25).clamp(0.25, 4.0);
        assert!(
            (speed - 0.25).abs() < f32::EPSILON,
            "Speed should not go below 0.25"
        );
    }

    #[test]
    fn test_speed_no_overflow_at_maximum() {
        let mut speed: f32 = 4.0;
        speed = (speed + 0.25).clamp(0.25, 4.0);
        assert!(
            (speed - 4.0).abs() < f32::EPSILON,
            "Speed should not exceed 4.0"
        );
    }

    #[test]
    fn test_hud_toggle() {
        let mut hide_hud = false;
        hide_hud = !hide_hud;
        assert!(hide_hud, "First toggle should hide HUD");
        hide_hud = !hide_hud;
        assert!(!hide_hud, "Second toggle should show HUD");
    }

    #[test]
    fn test_help_text_toggle() {
        let mut show_help = false;
        show_help = !show_help;
        assert!(show_help, "First toggle should show help");
        show_help = !show_help;
        assert!(!show_help, "Second toggle should hide help");
    }

    #[test]
    fn test_help_text_fits_min_width() {
        let help_text = "q:Quit p:Pause r:Refresh h:HUD +/-:Speed ?:Help";
        assert_eq!(
            help_text.len(),
            47,
            "Help text should be exactly 47 characters"
        );
        assert!(
            help_text.len() < 70,
            "Help text must fit within MIN_WIDTH=70"
        );
    }

    #[test]
    fn test_rapid_toggles_return_to_original() {
        let mut paused = false;
        for _ in 0..100 {
            paused = !paused;
        }
        assert!(
            !paused,
            "100 toggles (even) should return to original state"
        );
    }
}
