use crate::animation_manager::AnimationManager;
use crate::app_state::AppState;
use crate::config::Config;
use crate::render::TerminalRenderer;
use crate::scene::WorldScene;
use crate::weather::{
    OpenMeteoProvider, WeatherClient, WeatherCondition, WeatherData, WeatherLocation, WeatherUnits,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

const REFRESH_INTERVAL: Duration = Duration::from_secs(300);
const INPUT_POLL_FPS: u64 = 30;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / INPUT_POLL_FPS);

pub struct App {
    state: AppState,
    animations: AnimationManager,
    scene: WorldScene,
    weather_receiver: mpsc::Receiver<Result<WeatherData, String>>,
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

        let mut state = AppState::new(location);
        let animations = AnimationManager::new(term_width, term_height, show_leaves);
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
                wind_speed: 10.0,
                wind_direction: 180.0,
                cloud_cover: 50.0,
                pressure: 1013.0,
                visibility: Some(10000.0),
                is_day: !simulate_night,
                moon_phase: Some(0.5),
                timestamp: "simulated".to_string(),
            };

            state.update_weather(weather);
        } else {
            let provider = Arc::new(OpenMeteoProvider::new());
            let weather_client = WeatherClient::new(provider, REFRESH_INTERVAL);
            let units = WeatherUnits::default();

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
        }
    }

    pub async fn run(&mut self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        loop {
            if let Ok(result) = self.weather_receiver.try_recv() {
                match result {
                    Ok(weather) => {
                        let rain_intensity = weather.condition.rain_intensity();
                        let snow_intensity = weather.condition.snow_intensity();

                        self.state.update_weather(weather);
                        self.animations.update_rain_intensity(rain_intensity);
                        self.animations.update_snow_intensity(snow_intensity);
                    }
                    Err(e) => {
                        self.state
                            .set_weather_error(format!("Error fetching weather: {}", e));
                    }
                }
            }

            renderer.clear()?;

            let (term_width, term_height) = renderer.get_size();

            self.animations.render_background(
                renderer,
                &self.state.weather_conditions,
                &self.state,
                term_width,
                term_height,
            )?;

            self.scene
                .render(renderer, &self.state.weather_conditions)?;

            self.animations.render_chimney_smoke(
                renderer,
                &self.state.weather_conditions,
                term_width,
                term_height,
            )?;

            self.animations.render_foreground(
                renderer,
                &self.state.weather_conditions,
                term_width,
                term_height,
            )?;

            self.state.update_loading_animation();
            self.state.update_cached_info();

            renderer.render_line_colored(
                2,
                1,
                &self.state.cached_weather_info,
                crossterm::style::Color::Cyan,
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
