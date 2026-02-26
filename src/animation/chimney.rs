use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::prelude::*;
use std::io;

const MAX_PARTICLES: usize = 200;
const MIN_PARTICLE_MAX_AGE: u32 = 70;
const PARTICLE_MAX_AGE_VARIANCE: u32 = 30;
const PARTICLE_VERTICAL_SPEED: f32 = 0.1;
const PARTICLE_DRIFT_SCALE: f32 = 0.08;
const PARTICLE_SPAWN_JITTER_X: f32 = 1.6;
const DEFAULT_SPAWN_RATE: u32 = 12;

struct SmokeParticle {
    x: f32,
    y: f32,
    age: u32,
    max_age: u32,
    drift: f32,
}

impl SmokeParticle {
    fn new(chimney_x: u16, chimney_y: u16, rng: &mut impl Rng) -> Self {
        let drift = (rng.random::<f32>() - 0.5) * PARTICLE_DRIFT_SCALE;
        let max_age = MIN_PARTICLE_MAX_AGE + (rng.random::<u32>() % PARTICLE_MAX_AGE_VARIANCE);

        Self {
            x: chimney_x as f32 + (rng.random::<f32>() - 0.5) * PARTICLE_SPAWN_JITTER_X,
            y: chimney_y as f32,
            age: 0,
            max_age,
            drift,
        }
    }

    fn update(&mut self) {
        self.age += 1;
        self.y -= PARTICLE_VERTICAL_SPEED;
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
            spawn_rate: DEFAULT_SPAWN_RATE,
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
