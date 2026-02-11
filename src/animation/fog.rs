use crate::render::TerminalRenderer;
use crate::weather::types::FogIntensity;
use crossterm::style::Color;
use rand::prelude::*;
use std::collections::VecDeque;
use std::io;

struct FogWisp {
    x: f32,
    y: f32,
    speed_x: f32,
    character: char,
    color: Color,
    lifetime: u32,
    max_lifetime: u32,
}

impl FogWisp {
    fn new(terminal_width: u16, terminal_height: u16, rng: &mut impl Rng) -> Self {
        let ground_level = terminal_height.saturating_sub(7);
        let fog_zone_top = ground_level.saturating_sub(15);

        let x = rng.random::<f32>() * terminal_width as f32;
        let y = fog_zone_top as f32 + (rng.random::<f32>() * 15.0);

        let chars = ['.', ',', '-', '~'];
        let char_idx = (rng.random::<u32>() as usize) % chars.len();

        let colors = [
            Color::Grey,
            Color::DarkGrey,
            Color::Rgb {
                r: 120,
                g: 120,
                b: 120,
            },
        ];
        let color_idx = (rng.random::<u32>() as usize) % colors.len();

        Self {
            x,
            y,
            speed_x: (rng.random::<f32>() - 0.5) * 0.15,
            character: chars[char_idx],
            color: colors[color_idx],
            lifetime: 0,
            max_lifetime: 100 + (rng.random::<u32>() % 200),
        }
    }

    fn update(&mut self) {
        self.x += self.speed_x;
        self.lifetime += 1;
    }

    fn is_alive(&self, terminal_width: u16) -> bool {
        self.lifetime < self.max_lifetime
            && self.x >= -5.0
            && self.x < (terminal_width as f32 + 5.0)
    }
}

pub struct FogSystem {
    wisps: VecDeque<FogWisp>,
    terminal_width: u16,
    terminal_height: u16,
    intensity: FogIntensity,
    spawn_timer: u32,
}

impl FogSystem {
    pub fn new(terminal_width: u16, terminal_height: u16, intensity: FogIntensity) -> Self {
        let wisps_capacity = match intensity {
            FogIntensity::Light => (terminal_width as f32 * 0.3) as usize,
            FogIntensity::Medium => (terminal_width as f32 * 0.6) as usize,
            FogIntensity::Heavy => terminal_width as usize,
        };

        Self {
            wisps: VecDeque::with_capacity(wisps_capacity),
            terminal_width,
            terminal_height,
            intensity,
            spawn_timer: 0,
        }
    }

    pub fn set_intensity(&mut self, intensity: FogIntensity) {
        self.intensity = intensity;
    }

    pub fn update(&mut self, terminal_width: u16, terminal_height: u16, rng: &mut impl Rng) {
        self.terminal_width = terminal_width;
        self.terminal_height = terminal_height;

        for wisp in &mut self.wisps {
            wisp.update();
        }

        self.wisps.retain(|w| w.is_alive(terminal_width));

        let (target_multiplier, spawn_delay) = match self.intensity {
            FogIntensity::Light => (0.3, 4),
            FogIntensity::Medium => (0.6, 2),
            FogIntensity::Heavy => (1.0, 1),
        };
        let target_count = (terminal_width as f32 * target_multiplier) as usize;

        self.spawn_timer += 1;
        if self.spawn_timer >= spawn_delay && self.wisps.len() < target_count {
            self.spawn_timer = 0;
            for _ in 0..2 {
                if self.wisps.len() < target_count {
                    self.wisps
                        .push_back(FogWisp::new(terminal_width, terminal_height, rng));
                }
            }
        }
    }

    pub fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        for wisp in &self.wisps {
            let x = wisp.x as i16;
            let y = wisp.y as i16;

            if x >= 0 && x < self.terminal_width as i16 && y >= 0 && y < self.terminal_height as i16
            {
                renderer.render_char(x as u16, y as u16, wisp.character, wisp.color)?;
            }
        }
        Ok(())
    }
}
