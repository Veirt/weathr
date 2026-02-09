use crate::render::TerminalRenderer;
use crate::weather::types::FogIntensity;
use crossterm::style::Color;
use std::io;

struct FogParticle {
    x: f32,
    y: f32,
    speed_y: f32,
    speed_x: f32,
    drift_offset: f32,
    character: char,
    color: Color,
}

pub struct FogSystem {
    particles: Vec<FogParticle>,
    terminal_width: u16,
    terminal_height: u16,
    intensity: FogIntensity,
    drift_x: f32,
}

impl FogSystem {
    pub fn new(terminal_width: u16, terminal_height: u16, intensity: FogIntensity) -> Self {
        let mut system = Self {
            particles: Vec::new(),
            terminal_width,
            terminal_height,
            intensity,
            drift_x: 0.0,
        };
        let drift_dir = if rand::random::<bool>() { 0.05 } else { -0.05 };
        system.set_intensity_with_dir(intensity, drift_dir);
        system
    }

    pub fn set_intensity(&mut self, intensity: FogIntensity) {
        let current_dir = if self.drift_x >= 0.0 { 1.0 } else { -1.0 };
        self.set_intensity_with_dir(intensity, current_dir);
    }

    pub fn set_intensity_with_dir(&mut self, intensity: FogIntensity, direction_multiplier: f32) {
        self.intensity = intensity;
        let base_drift = match intensity {
            FogIntensity::Light => 0.02,
            FogIntensity::Medium => 0.05,
            FogIntensity::Heavy => 0.08,
        };
        self.drift_x = base_drift * direction_multiplier;
    }

    fn spawn_particle(&mut self) {
        let x = (rand::random::<u32>() % (self.terminal_width as u32 * 3)) as f32
            - (self.terminal_width as f32);

        let y = if rand::random::<bool>() {
            0.0
        } else {
            (rand::random::<u32>() % self.terminal_height as u32) as f32
        };

        let depth = rand::random::<u8>() % 3;

        let (base_speed_y, chars, color) = match self.intensity {
            FogIntensity::Light => (
                if depth == 0 { 0.02 } else { 0.01 },
                vec!['.', '·'],
                if depth == 0 {
                    Color::Grey
                } else {
                    Color::DarkGrey
                },
            ),
            FogIntensity::Medium => (
                if depth == 0 {
                    0.03
                } else if depth == 1 {
                    0.02
                } else {
                    0.01
                },
                vec!['.', '·', ':'],
                match depth {
                    0 => Color::White,
                    1 => Color::Grey,
                    _ => Color::DarkGrey,
                },
            ),
            FogIntensity::Heavy => (
                if depth == 0 {
                    0.04
                } else if depth == 1 {
                    0.03
                } else {
                    0.02
                },
                vec!['.', '·', ':', '░'],
                match depth {
                    0 => Color::White,
                    1 => Color::Grey,
                    _ => Color::DarkGrey,
                },
            ),
        };

        let char_idx = (rand::random::<u32>() as usize) % chars.len();

        self.particles.push(FogParticle {
            x,
            y,
            speed_y: base_speed_y + (rand::random::<f32>() * 0.01),
            speed_x: self.drift_x + (rand::random::<f32>() * 0.03 - 0.015),
            drift_offset: rand::random::<f32>() * 100.0,
            character: chars[char_idx],
            color,
        });
    }

    pub fn update(&mut self, terminal_width: u16, terminal_height: u16) {
        self.terminal_width = terminal_width;
        self.terminal_height = terminal_height;

        let target_count = match self.intensity {
            FogIntensity::Light => (terminal_width as usize * terminal_height as usize) / 8,
            FogIntensity::Medium => (terminal_width as usize * terminal_height as usize) / 4,
            FogIntensity::Heavy => (terminal_width as usize * terminal_height as usize) / 2,
        };

        if self.particles.len() < target_count {
            let spawn_rate = match self.intensity {
                FogIntensity::Light => 2,
                FogIntensity::Medium => 4,
                FogIntensity::Heavy => 8,
            };
            for _ in 0..spawn_rate {
                self.spawn_particle();
            }
        }

        self.particles.retain_mut(|particle| {
            particle.y += particle.speed_y;

            let drift = (particle.y * 0.1 + particle.drift_offset).sin() * 0.03;
            particle.x += particle.speed_x + drift;

            if particle.y >= (terminal_height - 1) as f32 {
                particle.y = 0.0;
                particle.x = (rand::random::<u32>() % terminal_width as u32) as f32;
            }

            if particle.x < -20.0 || particle.x > (terminal_width as f32 + 20.0) {
                return false;
            }

            true
        });
    }

    pub fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        for particle in &self.particles {
            let x = particle.x as i16;
            let y = particle.y as i16;

            if x >= 0 && x < self.terminal_width as i16 && y >= 0 && y < self.terminal_height as i16
            {
                renderer.render_char(x as u16, y as u16, particle.character, particle.color)?;
            }
        }
        Ok(())
    }
}
