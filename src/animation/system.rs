use crate::app_state::AppState;
use crate::render::TerminalRenderer;
use crate::weather::{FogIntensity, RainIntensity, SnowIntensity, WeatherConditions};
use rand::Rng;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderLayer {
    Background,
    PostScene,
    Foreground,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wind {
    pub speed_kmh: f32,
    pub direction_deg: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FrameCommands {
    pub flash_screen: bool,
}

pub struct FrameContext<'a> {
    pub size: TerminalSize,
    pub horizon_y: u16,
    pub conditions: &'a WeatherConditions,
    pub state: &'a AppState,
    pub show_leaves: bool,
    pub chimney: Option<ChimneyPosition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChimneyPosition {
    pub x: u16,
    pub y: u16,
}

pub trait AnimationSystem {
    fn id(&self) -> &'static str;
    fn layer(&self) -> RenderLayer;

    fn is_active(&self, _ctx: &FrameContext<'_>) -> bool {
        true
    }

    fn on_resize(&mut self, _size: TerminalSize) {}
    fn on_wind(&mut self, _wind: Wind) {}
    fn on_rain_intensity(&mut self, _intensity: RainIntensity) {}
    fn on_snow_intensity(&mut self, _intensity: SnowIntensity) {}
    fn on_fog_intensity(&mut self, _intensity: FogIntensity) {}
    fn on_moon_phase(&mut self, _phase: f64) {}

    fn update(&mut self, ctx: &FrameContext<'_>, rng: &mut dyn Rng, commands: &mut FrameCommands);
    fn render(&mut self, renderer: &mut TerminalRenderer, ctx: &FrameContext<'_>)
    -> io::Result<()>;
}
