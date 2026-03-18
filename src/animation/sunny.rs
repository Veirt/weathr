use super::Animation;
use crate::animation::{
    AnimationController, AnimationSystem, FrameCommands, FrameContext, RenderLayer,
};
use crate::render::TerminalRenderer;
use crate::weather::types::CelestialEvents;
use chrono::{DateTime, NaiveDateTime, NaiveTime};
use crossterm::style::Color;
use rand::Rng;

use std::io;
use std::time::{Duration, Instant};

const FRAME_DELAY: Duration = Duration::from_millis(500);

const SUN_FRAMES: [&str; 2] = [
    include_str!("assets/sun_0.txt"),
    include_str!("assets/sun_1.txt"),
];

pub struct SunnyAnimation {
    frames: Vec<Vec<String>>,
}

impl SunnyAnimation {
    pub fn new() -> Self {
        let frames = SUN_FRAMES
            .iter()
            .map(|src| src.lines().map(|l| l.to_string()).collect())
            .collect();
        Self { frames }
    }
}

impl Animation for SunnyAnimation {
    fn get_frame(&self, frame_number: usize) -> &[String] {
        &self.frames[frame_number % self.frames.len()]
    }

    fn frame_count(&self) -> usize {
        self.frames.len()
    }

    fn get_color(&self) -> Color {
        Color::Yellow
    }
}

impl Default for SunnyAnimation {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SunSystem {
    animation: SunnyAnimation,
    controller: AnimationController,
    last_frame_time: Instant,
}

impl SunSystem {
    pub fn new() -> Self {
        Self {
            animation: SunnyAnimation::new(),
            controller: AnimationController::new(),
            last_frame_time: Instant::now(),
        }
    }

    fn sun_y(
        now: NaiveTime,
        lowest: NaiveTime,
        highest: NaiveTime,
        horizon_y: u16,
        default_y: u16,
    ) -> u16 {
        use std::f64::consts::PI;

        const BUILDING_BIAS: u16 = 5;

        let half_period = (highest - lowest).num_seconds().unsigned_abs() as f64;
        if half_period == 0.0 {
            return default_y;
        }

        let dist_from_peak = (now - highest).num_seconds().unsigned_abs() as f64;
        let progress = (dist_from_peak / half_period).clamp(0.0, 1.0);
        let range = horizon_y
            .saturating_sub(default_y)
            .saturating_sub(BUILDING_BIAS) as f64;
        let offset = range * (1.0 - (progress * PI).cos()) / 2.0;

        default_y + offset.round() as u16
    }

    fn dynamic_y(
        now: NaiveTime,
        sun: &CelestialEvents,
        horizon_y: u16,
        default_y: u16,
        hidden_y: u16,
    ) -> u16 {
        let (Some(begin_twilight), Some(upper_transit), Some(end_twilight)) =
            (sun.begin_twilight, sun.upper_transit, sun.end_twilight)
        else {
            return default_y;
        };

        if now < upper_transit {
            Self::sun_y(now, begin_twilight, upper_transit, horizon_y, default_y)
        } else if now < end_twilight {
            Self::sun_y(now, end_twilight, upper_transit, horizon_y, default_y)
        } else if now > end_twilight {
            hidden_y
        } else {
            default_y
        }
    }
}

impl Default for SunSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationSystem for SunSystem {
    fn id(&self) -> &'static str {
        "sun"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::Background
    }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        !ctx.conditions.is_raining && !ctx.conditions.is_thunderstorm && !ctx.conditions.is_snowing
    }

    fn update(
        &mut self,
        _ctx: &FrameContext<'_>,
        _rng: &mut dyn Rng,
        _commands: &mut FrameCommands,
    ) {
        if self.last_frame_time.elapsed() >= FRAME_DELAY {
            self.controller.next_frame(&self.animation);
            self.last_frame_time = Instant::now();
        }
    }

    fn render(
        &mut self,
        renderer: &mut TerminalRenderer,
        ctx: &FrameContext<'_>,
    ) -> io::Result<()> {
        if !ctx.state.should_show_sun()
            || ctx.conditions.is_raining
            || ctx.conditions.is_thunderstorm
            || ctx.conditions.is_snowing
        {
            return Ok(());
        }

        let default_y = if ctx.size.height > 20 { 3 } else { 2 };
        let y_offset = Self::resolved_sun_y(ctx, default_y);
        self.controller
            .render_frame(renderer, &self.animation, y_offset)
    }
}

impl SunSystem {
    fn parse_weather_time(timestamp: &str) -> Option<NaiveTime> {
        if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
            return Some(dt.time());
        }

        if let Ok(dt) = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S") {
            return Some(dt.time());
        }

        if let Ok(dt) = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M") {
            return Some(dt.time());
        }

        None
    }

    fn weather_time_from_ctx(ctx: &FrameContext<'_>) -> Option<NaiveTime> {
        let weather = ctx.state.current_weather.as_ref()?;
        Self::parse_weather_time(&weather.timestamp)
    }

    fn resolved_sun_y(ctx: &FrameContext<'_>, default_y: u16) -> u16 {
        if let Some(now) = Self::weather_time_from_ctx(ctx) {
            Self::dynamic_y(
                now,
                &ctx.conditions.sun,
                ctx.horizon_y,
                default_y,
                ctx.size.height,
            )
        } else {
            default_y
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::TerminalSize;
    use crate::app_state::AppState;
    use crate::config::LocationDisplay;
    use crate::weather::types::CelestialEvents;
    use crate::weather::{
        WeatherCondition, WeatherConditions, WeatherData, WeatherLocation, WeatherUnits,
    };
    use chrono::NaiveTime;

    fn sample_celestial_events() -> CelestialEvents {
        CelestialEvents {
            is_day: true,
            begin_twilight: Some(NaiveTime::from_hms_opt(5, 30, 0).unwrap()),
            rise: Some(NaiveTime::from_hms_opt(6, 0, 0).unwrap()),
            upper_transit: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            set: Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
            end_twilight: Some(NaiveTime::from_hms_opt(20, 0, 0).unwrap()),
        }
    }

    #[test]
    fn parses_rfc3339_timestamp() {
        let time = SunSystem::parse_weather_time("2024-01-01T12:34:56Z").unwrap();
        assert_eq!(time, NaiveTime::from_hms_opt(12, 34, 56).unwrap());
    }

    #[test]
    fn parses_naive_timestamp() {
        let time = SunSystem::parse_weather_time("2024-01-01T06:15").unwrap();
        assert_eq!(time, NaiveTime::from_hms_opt(6, 15, 0).unwrap());
    }

    #[test]
    fn resolved_y_uses_weather_time() {
        let sun = sample_celestial_events();
        let location = WeatherLocation {
            latitude: 0.0,
            longitude: 0.0,
            elevation: None,
        };
        let units = WeatherUnits::metric();
        let mut state = AppState::new(location, None, LocationDisplay::Coordinates, false, units);
        state.current_weather = Some(WeatherData {
            condition: WeatherCondition::Clear,
            temperature: 20.0,
            precipitation: 0.0,
            wind_speed: 5.0,
            wind_direction: 0.0,
            sun,
            moon_phase: None,
            timestamp: "2024-01-01T21:00:00Z".to_string(),
            attribution: String::new(),
        });
        let mut conditions = WeatherConditions::default();
        conditions.sun = sun;

        let ctx = FrameContext {
            size: TerminalSize {
                width: 80,
                height: 24,
            },
            horizon_y: 18,
            conditions: &conditions,
            state: &state,
            show_leaves: false,
            chimney: None,
        };

        let y = SunSystem::resolved_sun_y(&ctx, 3);
        assert_eq!(y, ctx.size.height);
    }

    #[test]
    fn resolved_y_defaults_without_time() {
        let sun = sample_celestial_events();
        let location = WeatherLocation {
            latitude: 0.0,
            longitude: 0.0,
            elevation: None,
        };
        let units = WeatherUnits::metric();
        let mut state = AppState::new(location, None, LocationDisplay::Coordinates, false, units);
        state.current_weather = Some(WeatherData {
            condition: WeatherCondition::Clear,
            temperature: 20.0,
            precipitation: 0.0,
            wind_speed: 5.0,
            wind_direction: 0.0,
            sun,
            moon_phase: None,
            timestamp: "n/a".to_string(),
            attribution: String::new(),
        });
        let mut conditions = WeatherConditions::default();
        conditions.sun = sun;

        let ctx = FrameContext {
            size: TerminalSize {
                width: 80,
                height: 24,
            },
            horizon_y: 18,
            conditions: &conditions,
            state: &state,
            show_leaves: false,
            chimney: None,
        };

        let y = SunSystem::resolved_sun_y(&ctx, 4);
        assert_eq!(y, 4);
    }
}
