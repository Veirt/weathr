use crate::render::TerminalRenderer;
use crate::weather::types::RainIntensity;
use crossterm::style::Color;
use rand::prelude::*;
use std::collections::VecDeque;
use std::io;

const MAX_SPLASHES: usize = 100;

struct Raindrop {
    x: f32,
    y: f32,
    speed_y: f32,
    speed_x: f32,
    character: char,
    color: Color,
    z_index: u8,
}

#[derive(Clone, Copy)]
struct Splash {
    x: u16,
    y: u16,
    timer: u8,
    max_timer: u8,
}

pub struct RaindropSystem {
    drops: Vec<Raindrop>,
    splashes: VecDeque<Splash>,
    new_splashes: VecDeque<Splash>,
    terminal_width: u16,
    terminal_height: u16,
    intensity: RainIntensity,
    wind_x: f32,
}

impl RaindropSystem {
    pub fn new(terminal_width: u16, terminal_height: u16, intensity: RainIntensity) -> Self {
        let drops_capacity = match intensity {
            RainIntensity::Drizzle => (terminal_width / 4) as usize,
            RainIntensity::Light => (terminal_width / 2) as usize,
            RainIntensity::Heavy => terminal_width as usize,
            RainIntensity::Storm => (terminal_width as f32 * 1.5) as usize,
        };

        let mut system = Self {
            drops: Vec::with_capacity(drops_capacity),
            splashes: VecDeque::with_capacity(MAX_SPLASHES),
            new_splashes: VecDeque::with_capacity(20),
            terminal_width,
            terminal_height,
            intensity,
            wind_x: 0.0,
        };
        let wind_dir = if rand::random::<bool>() { 1.0 } else { -1.0 };
        system.set_intensity_with_dir(intensity, wind_dir);
        system
    }

    pub fn set_intensity(&mut self, intensity: RainIntensity) {
        let current_dir = if self.wind_x >= 0.0 { 1.0 } else { -1.0 };
        self.set_intensity_with_dir(intensity, current_dir);
    }

    pub fn set_intensity_with_dir(&mut self, intensity: RainIntensity, direction_multiplier: f32) {
        self.intensity = intensity;
        let base_wind = match intensity {
            RainIntensity::Drizzle => 0.05,
            RainIntensity::Light => 0.1,
            RainIntensity::Heavy => 0.15,
            RainIntensity::Storm => 0.8,
        };
        self.wind_x = base_wind * direction_multiplier;
    }

    pub fn set_wind(&mut self, speed_kmh: f32, direction_deg: f32) {
        let speed_factor = speed_kmh / 40.0;
        let direction_rad = direction_deg.to_radians();
        let x_component = -direction_rad.sin();
        self.wind_x = speed_factor * x_component;
    }

    fn spawn_drop(&mut self, rng: &mut impl Rng) {
        let x = (rng.random::<u32>() % (self.terminal_width as u32 * 2)) as f32
            - (self.terminal_width as f32 * 0.5);
        let z_index = if rng.random::<bool>() { 1 } else { 0 };

        let (speed_y, chars, color) = match self.intensity {
            RainIntensity::Drizzle => (
                if z_index == 1 { 0.4 } else { 0.2 },
                vec!['.', ','],
                if z_index == 1 {
                    Color::Cyan
                } else {
                    Color::DarkGrey
                },
            ),
            RainIntensity::Light => (
                if z_index == 1 { 0.7 } else { 0.4 },
                vec!['|', ':', '.'],
                if z_index == 1 {
                    Color::White
                } else {
                    Color::DarkGrey
                },
            ),
            RainIntensity::Heavy => (
                if z_index == 1 { 0.9 } else { 0.6 }, // Slightly faster than Light
                vec!['|', ':'],                       // Vertical density
                if z_index == 1 {
                    Color::Cyan
                } else {
                    Color::DarkGrey // Blue-ish background
                },
            ),
            RainIntensity::Storm => (
                if z_index == 1 { 1.8 } else { 1.2 },
                // Use slant matching wind direction
                if self.wind_x > 0.0 {
                    vec!['\\']
                } else {
                    vec!['/']
                },
                if z_index == 1 {
                    Color::White
                } else {
                    Color::DarkGrey
                },
            ),
        };

        let char_idx = (rng.random::<u32>() as usize) % chars.len();

        self.drops.push(Raindrop {
            x,
            y: 0.0,
            speed_y: speed_y + (rng.random::<f32>() * 0.2),
            speed_x: self.wind_x + (rng.random::<f32>() * 0.1 - 0.05),
            character: chars[char_idx],
            color,
            z_index,
        });
    }

    pub fn update(&mut self, terminal_width: u16, terminal_height: u16, rng: &mut impl Rng) {
        self.terminal_width = terminal_width;
        self.terminal_height = terminal_height;

        let target_count = match self.intensity {
            RainIntensity::Drizzle => (terminal_width / 4) as usize,
            RainIntensity::Light => (terminal_width / 2) as usize,
            RainIntensity::Heavy => terminal_width as usize,
            RainIntensity::Storm => (terminal_width as f32 * 1.5) as usize,
        };

        if self.drops.len() < target_count {
            let spawn_rate = match self.intensity {
                RainIntensity::Drizzle => 1,
                RainIntensity::Light => 2,
                _ => 5,
            };
            for _ in 0..spawn_rate {
                self.spawn_drop(rng);
            }
        }

        // Update drops
        let new_splashes = &mut self.new_splashes;
        let splash_chance = match self.intensity {
            RainIntensity::Drizzle => 0.1,
            RainIntensity::Light => 0.3,
            _ => 0.6,
        };

        self.drops.retain_mut(|drop| {
            drop.y += drop.speed_y;
            drop.x += drop.speed_x;

            // Hit ground?
            if drop.y >= (terminal_height - 1) as f32 {
                if drop.z_index == 1 && rng.random::<f32>() < splash_chance {
                    new_splashes.push_back(Splash {
                        x: drop.x as u16,
                        y: terminal_height - 1,
                        timer: 0,
                        max_timer: 3,
                    });
                }
                return false; // Remove drop
            }

            // Out of horizontal bounds
            if drop.x < -10.0 || drop.x > (terminal_width as f32 + 10.0) {
                return false;
            }

            true
        });

        self.splashes.append(&mut self.new_splashes);

        while self.splashes.len() > MAX_SPLASHES {
            self.splashes.pop_front();
        }

        self.splashes.retain_mut(|splash| {
            splash.timer += 1;
            splash.timer < splash.max_timer
        });
    }

    pub fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        // Render drops
        for drop in &self.drops {
            let x = drop.x as i16;
            let y = drop.y as i16;

            if x >= 0 && x < self.terminal_width as i16 && y >= 0 && y < self.terminal_height as i16
            {
                let ch = if self.intensity == RainIntensity::Storm
                    || self.intensity == RainIntensity::Heavy
                {
                    if drop.speed_x > 0.5 {
                        '\\'
                    } else if drop.speed_x < -0.5 {
                        '/'
                    } else {
                        drop.character
                    }
                } else {
                    drop.character
                };
                renderer.render_char(x as u16, y as u16, ch, drop.color)?;
            }
        }

        // Render splashes
        for splash in &self.splashes {
            if splash.x < self.terminal_width && splash.y < self.terminal_height {
                let ch = match splash.timer {
                    0 => '.',
                    1 => 'o',
                    2 => 'O',
                    _ => ' ',
                };
                renderer.render_char(splash.x, splash.y, ch, Color::White)?;
            }
        }

        Ok(())
    }
}
