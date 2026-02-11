use crate::render::TerminalRenderer;
use crate::weather::types::SnowIntensity;
use crossterm::style::Color;
use rand::prelude::*;
use std::io;

struct Snowflake {
    x: f32,
    y: f32,
    speed_y: f32,
    speed_x: f32,
    sway_offset: f32,
    character: char,
    color: Color,
}

pub struct SnowSystem {
    flakes: Vec<Snowflake>,
    terminal_width: u16,
    terminal_height: u16,
    intensity: SnowIntensity,
    wind_x: f32,
}

impl SnowSystem {
    pub fn new(terminal_width: u16, terminal_height: u16, intensity: SnowIntensity) -> Self {
        let flakes_capacity = match intensity {
            SnowIntensity::Light => (terminal_width / 4) as usize,
            SnowIntensity::Medium => (terminal_width / 2) as usize,
            SnowIntensity::Heavy => terminal_width as usize,
        };

        let mut system = Self {
            flakes: Vec::with_capacity(flakes_capacity),
            terminal_width,
            terminal_height,
            intensity,
            wind_x: 0.0,
        };
        // Initialize with some default wind
        let wind_dir = if rand::random::<bool>() { 0.2 } else { -0.2 };
        system.set_intensity_with_dir(intensity, wind_dir);
        system
    }

    pub fn set_intensity(&mut self, intensity: SnowIntensity) {
        // Preserve direction but update magnitude based on intensity if needed
        let current_dir = if self.wind_x >= 0.0 { 1.0 } else { -1.0 };
        self.set_intensity_with_dir(intensity, current_dir);
    }

    pub fn set_intensity_with_dir(&mut self, intensity: SnowIntensity, direction_multiplier: f32) {
        self.intensity = intensity;
        let base_wind = match intensity {
            SnowIntensity::Light => 0.05,
            SnowIntensity::Medium => 0.1,
            SnowIntensity::Heavy => 0.2,
        };
        self.wind_x = base_wind * direction_multiplier;
    }

    pub fn set_wind(&mut self, speed_kmh: f32, direction_deg: f32) {
        let speed_factor = speed_kmh / 20.0;
        let direction_rad = direction_deg.to_radians();
        let x_component = -direction_rad.sin();
        self.wind_x = speed_factor * x_component;
    }

    fn spawn_flake(&mut self, rng: &mut impl Rng) {
        // Spawn across a wider area to account for wind blowing them in
        let x = (rng.random::<u32>() % (self.terminal_width as u32 * 3)) as f32
            - (self.terminal_width as f32);

        let z_index = if rng.random::<bool>() { 1 } else { 0 };

        let (base_speed_y, chars) = match self.intensity {
            SnowIntensity::Light => (if z_index == 1 { 0.15 } else { 0.08 }, vec!['.', '·']),
            SnowIntensity::Medium => (if z_index == 1 { 0.2 } else { 0.1 }, vec!['.', '·', '*']),
            SnowIntensity::Heavy => (if z_index == 1 { 0.3 } else { 0.15 }, vec!['*', '.', '·']),
        };

        let char_idx = (rng.random::<u32>() as usize) % chars.len();

        self.flakes.push(Snowflake {
            x,
            y: 0.0,
            speed_y: base_speed_y + (rng.random::<f32>() * 0.05),
            speed_x: self.wind_x + (rng.random::<f32>() * 0.1 - 0.05),
            sway_offset: rng.random::<f32>() * 100.0, // Random phase for sway
            character: chars[char_idx],
            color: if z_index == 1 {
                Color::White
            } else {
                Color::DarkGrey
            },
        });
    }

    pub fn update(&mut self, terminal_width: u16, terminal_height: u16, rng: &mut impl Rng) {
        self.terminal_width = terminal_width;
        self.terminal_height = terminal_height;

        let target_count = match self.intensity {
            SnowIntensity::Light => (terminal_width / 4) as usize,
            SnowIntensity::Medium => (terminal_width / 2) as usize,
            SnowIntensity::Heavy => terminal_width as usize,
        };

        if self.flakes.len() < target_count {
            let spawn_rate = match self.intensity {
                SnowIntensity::Light => 1,
                SnowIntensity::Medium => 2,
                SnowIntensity::Heavy => 4,
            };
            for _ in 0..spawn_rate {
                self.spawn_flake(rng);
            }
        }

        self.flakes.retain_mut(|flake| {
            flake.y += flake.speed_y;

            // Add horizontal sway
            let sway = (flake.y * 0.2 + flake.sway_offset).sin() * 0.05;
            flake.x += flake.speed_x + sway;

            // Hit ground or out of bounds
            if flake.y >= (terminal_height - 1) as f32 {
                return false;
            }

            // Out of horizontal bounds (allow some buffer)
            if flake.x < -20.0 || flake.x > (terminal_width as f32 + 20.0) {
                return false;
            }

            true
        });
    }

    pub fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        for flake in &self.flakes {
            let x = flake.x as i16;
            let y = flake.y as i16;

            if x >= 0 && x < self.terminal_width as i16 && y >= 0 && y < self.terminal_height as i16
            {
                renderer.render_char(x as u16, y as u16, flake.character, flake.color)?;
            }
        }
        Ok(())
    }
}
