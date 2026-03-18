use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::Rng;

use std::io;

const MOON_PHASES: [&str; 8] = [
    include_str!("assets/moon/phase_0.txt"),
    include_str!("assets/moon/phase_1.txt"),
    include_str!("assets/moon/phase_2.txt"),
    include_str!("assets/moon/phase_3.txt"),
    include_str!("assets/moon/phase_4.txt"),
    include_str!("assets/moon/phase_5.txt"),
    include_str!("assets/moon/phase_6.txt"),
    include_str!("assets/moon/phase_7.txt"),
];

pub struct MoonSystem {
    phase: f64, // 0.0 = New, 0.25 = First Quarter, 0.5 = Full, 0.75 = Last Quarter
    x: u16,
    y: u16,
}

impl MoonSystem {
    pub fn new(terminal_width: u16, terminal_height: u16, phase: Option<f64>) -> Self {
        Self {
            phase: phase.unwrap_or(0.5),
            x: (terminal_width / 4) + 10,
            y: (terminal_height / 4) + 2,
        }
    }

    pub fn set_phase(&mut self, phase: f64) {
        self.phase = phase;
    }

    pub fn update(&mut self, terminal_width: u16, terminal_height: u16) {
        self.x = (terminal_width / 4 * 3).min(terminal_width.saturating_sub(15));
        self.y = (terminal_height / 4).max(2);
    }

    pub fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        let step = (self.phase * 8.0).round() as usize % 8;
        let art = MOON_PHASES[step];

        for (i, line) in art.lines().enumerate() {
            let y = self.y + i as u16;
            for (j, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue; // Transparent (Sky)
                }

                let x = self.x + j as u16;

                if ch == '~' {
                    // Opaque Moon Body (hides stars) - Render as space but overwrite what's there
                    renderer.render_char(x, y, ' ', Color::White)?;
                } else {
                    // Texture/Outline
                    renderer.render_char(x, y, ch, Color::White)?;
                }
            }
        }
        Ok(())
    }
}

impl AnimationSystem for MoonSystem {
    fn id(&self) -> &'static str {
        "moon"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::Background
    }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        !ctx.conditions.sun.is_day
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.update(size.width, size.height);
    }

    fn on_moon_phase(&mut self, phase: f64) {
        self.set_phase(phase);
    }

    fn update(
        &mut self,
        ctx: &FrameContext<'_>,
        _rng: &mut dyn Rng,
        _commands: &mut FrameCommands,
    ) {
        self.update(ctx.size.width, ctx.size.height);
    }

    fn render(
        &mut self,
        renderer: &mut TerminalRenderer,
        _ctx: &FrameContext<'_>,
    ) -> io::Result<()> {
        MoonSystem::render(self, renderer)
    }
}
