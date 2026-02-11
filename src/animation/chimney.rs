use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::prelude::*;
use std::io;

const MAX_PARTICLES: usize = 200;

struct SmokeParticle {
    x: f32,
    y: f32,
    age: u32,
    max_age: u32,
    drift: f32,
}

impl SmokeParticle {
    fn new(chimney_x: u16, chimney_y: u16, rng: &mut impl Rng) -> Self {
        let drift = (rng.random::<f32>() - 0.5) * 0.15;
        let max_age = 30 + (rng.random::<u32>() % 15);

        Self {
            x: chimney_x as f32 + (rng.random::<f32>() - 0.5) * 2.0,
            y: chimney_y as f32,
            age: 0,
            max_age,
            drift,
        }
    }

    fn update(&mut self) {
        self.age += 1;
        self.y -= 0.2;
        self.x += self.drift;
    }

    fn is_alive(&self) -> bool {
        self.age < self.max_age
    }

    fn get_color(&self) -> Color {
        let life_ratio = self.age as f32 / self.max_age as f32;
        if life_ratio < 0.3 {
            Color::White
        } else if life_ratio < 0.6 {
            Color::Grey
        } else {
            Color::DarkGrey
        }
    }
}

pub struct ChimneySmoke {
    particles: Vec<SmokeParticle>,
    spawn_counter: u32,
    spawn_rate: u32,
}

impl ChimneySmoke {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(MAX_PARTICLES),
            spawn_counter: 0,
            spawn_rate: 8,
        }
    }

    pub fn update(&mut self, chimney_x: u16, chimney_y: u16, rng: &mut impl Rng) {
        for particle in &mut self.particles {
            particle.update();
        }

        self.particles.retain(|p| p.is_alive() && p.y >= 0.0);

        self.spawn_counter += 1;
        if self.spawn_counter >= self.spawn_rate && self.particles.len() < MAX_PARTICLES {
            self.spawn_counter = 0;
            self.particles
                .push(SmokeParticle::new(chimney_x, chimney_y, rng));
        }
    }

    pub fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        for particle in &self.particles {
            let x = particle.x as i16;
            let y = particle.y as i16;

            if x >= 0 && y >= 0 {
                let display_char = match particle.age {
                    0..=6 => 'o',
                    7..=14 => '.',
                    15..=25 => '~',
                    _ => '·',
                };

                renderer.render_char(x as u16, y as u16, display_char, particle.get_color())?;
            }
        }
        Ok(())
    }
}

impl Default for ChimneySmoke {
    fn default() -> Self {
        Self::new()
    }
}
