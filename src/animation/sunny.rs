use super::Animation;
use crate::animation::{
    AnimationController, AnimationSystem, FrameCommands, FrameContext, RenderLayer,
};
use crate::render::TerminalRenderer;
use crate::weather::types::CelestialEvents;
use chrono::{Local, NaiveTime};
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
        let y_offset = Self::dynamic_y(
            Local::now().time(),
            &ctx.conditions.sun,
            ctx.horizon_y,
            default_y,
            ctx.size.height,
        );
        self.controller
            .render_frame(renderer, &self.animation, y_offset)
    }
}
