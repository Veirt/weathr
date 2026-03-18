use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;

use rand::{Rng, RngExt};
use std::io;

struct Bird {
    x: f32,
    y: f32,
    speed: f32,
    character: char,
    flap_state: bool, // true = wings up, false = wings down/flat
    flap_timer: u8,
}

pub struct BirdSystem {
    birds: Vec<Bird>,
    terminal_width: u16,
    terminal_height: u16,
}

impl BirdSystem {
    pub fn new(terminal_width: u16, terminal_height: u16) -> Self {
        Self {
            birds: Vec::with_capacity(3),
            terminal_width,
            terminal_height,
        }
    }

    pub fn update(
        &mut self,
        terminal_width: u16,
        terminal_height: u16,
        rng: &mut (impl Rng + ?Sized),
    ) {
        self.terminal_width = terminal_width;
        self.terminal_height = terminal_height;

        for bird in &mut self.birds {
            bird.x += bird.speed;
            bird.flap_timer += 1;
            if bird.flap_timer > 5 {
                bird.flap_state = !bird.flap_state;
                bird.flap_timer = 0;
            }
            bird.character = if bird.flap_state { 'v' } else { '-' };
        }

        self.birds.retain(|b| b.x < terminal_width as f32);
        if self.birds.len() < 3 && rng.random::<f32>() < 0.01 {
            let spawn_band = (terminal_height / 3).max(1);
            let y = (rng.random::<u16>() % spawn_band) as f32;
            let speed = 0.2 + (rng.random::<f32>() * 0.2);
            self.birds.push(Bird {
                x: 0.0,
                y,
                speed,
                character: 'v',
                flap_state: true,
                flap_timer: 0,
            });
        }
    }

    pub fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        for bird in &self.birds {
            let x = bird.x as u16;
            let y = bird.y as u16;
            if x < self.terminal_width && y < self.terminal_height {
                renderer.render_char(x, y, bird.character, Color::Yellow)?;
            }
        }
        Ok(())
    }
}

impl AnimationSystem for BirdSystem {
    fn id(&self) -> &'static str {
        "birds"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::Background
    }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        ctx.conditions.sun.is_day
            && !ctx.conditions.is_raining
            && !ctx.conditions.is_thunderstorm
            && !ctx.conditions.is_snowing
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.terminal_width = size.width;
        self.terminal_height = size.height;
        self.birds
            .retain(|b| b.x < size.width as f32 && b.y < size.height as f32);
    }

    fn update(&mut self, ctx: &FrameContext<'_>, rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        self.update(ctx.size.width, ctx.size.height, rng);
    }

    fn render(
        &mut self,
        renderer: &mut TerminalRenderer,
        _ctx: &FrameContext<'_>,
    ) -> io::Result<()> {
        BirdSystem::render(self, renderer)
    }
}
