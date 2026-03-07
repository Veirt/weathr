use super::Animation;
use crate::animation::{
    AnimationController, AnimationSystem, FrameCommands, FrameContext, RenderLayer,
};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::Rng;

use std::io;
use std::time::{Duration, Instant};

const FRAME_DELAY: Duration = Duration::from_millis(500);

pub struct SunnyAnimation {
    frames: Vec<Vec<String>>,
}

impl SunnyAnimation {
    pub fn new() -> Self {
        let frames = vec![Self::create_frame_1(), Self::create_frame_2()];

        Self { frames }
    }

    fn create_frame_1() -> Vec<String> {
        vec![
            "      ;   :   ;".to_string(),
            "   .   \\_,!,_/   ,".to_string(),
            "    `.,'     `.,'".to_string(),
            "     /         \\".to_string(),
            "~ -- :         : -- ~".to_string(),
            "     \\         /".to_string(),
            "    ,'`._   _.'`.".to_string(),
            "   '   / `!` \\   `".to_string(),
            "      ;   :   ;".to_string(),
        ]
    }

    fn create_frame_2() -> Vec<String> {
        vec![
            "      .   |   .".to_string(),
            "   ;   \\_,|,_/   ;".to_string(),
            "    `.,'     `.,'".to_string(),
            "     /         \\".to_string(),
            "~ -- |         | -- ~".to_string(),
            "     \\         /".to_string(),
            "    ,'`._   _.'`.".to_string(),
            "   ;   / `|` \\   ;".to_string(),
            "      .   |   .".to_string(),
        ]
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

        let y_offset = if ctx.size.height > 20 { 3 } else { 2 };
        self.controller
            .render_frame(renderer, &self.animation, y_offset)
    }
}
