use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::prelude::*;
use std::io;

struct Leaf {
    x: f32,
    y: f32,
    fall_speed: f32,
    sway_speed: f32,
    sway_phase: f32,
    sway_amplitude: f32,
    rotation: u8,
    color: Color,
    character: char,
}

impl Leaf {
    fn new(terminal_width: u16, spawn_at_top: bool, rng: &mut impl Rng) -> Self {
        let x = rng.random::<f32>() * terminal_width as f32;
        let y = if spawn_at_top {
            -(rng.random::<f32>() * 5.0)
        } else {
            rng.random::<f32>() * terminal_width as f32
        };

        let fall_speed = 0.15 + (rng.random::<f32>() * 0.2);
        let sway_speed = 0.05 + (rng.random::<f32>() * 0.1);
        let sway_phase = rng.random::<f32>() * std::f32::consts::PI * 2.0;
        let sway_amplitude = 0.5 + (rng.random::<f32>() * 1.5);

        let colors = [
            Color::Rgb {
                r: 255,
                g: 165,
                b: 0,
            }, // Orange
            Color::Rgb {
                r: 218,
                g: 165,
                b: 32,
            }, // Golden
            Color::Rgb {
                r: 184,
                g: 134,
                b: 11,
            }, // Dark golden
            Color::Rgb {
                r: 205,
                g: 92,
                b: 92,
            }, // Indian red
            Color::Rgb {
                r: 160,
                g: 82,
                b: 45,
            }, // Sienna brown
            Color::Rgb {
                r: 139,
                g: 69,
                b: 19,
            }, // Saddle brown
        ];
        let color = colors[(rng.random::<u32>() % colors.len() as u32) as usize];

        let chars = ['*', '+', ',', '.', '~'];
        let character = chars[(rng.random::<u32>() % chars.len() as u32) as usize];

        Self {
            x,
            y,
            fall_speed,
            sway_speed,
            sway_phase,
            sway_amplitude,
            rotation: 0,
            color,
            character,
        }
    }

    fn update(&mut self) {
        self.y += self.fall_speed;

        self.sway_phase += self.sway_speed;
        if self.sway_phase > std::f32::consts::PI * 2.0 {
            self.sway_phase -= std::f32::consts::PI * 2.0;
        }

        let sway_offset = self.sway_phase.sin() * self.sway_amplitude;
        self.x += sway_offset * 0.1;

        self.rotation = ((self.sway_phase * 2.0).sin() * 4.0) as u8;
    }

    fn is_offscreen(&self, terminal_height: u16) -> bool {
        self.y > terminal_height as f32
    }

    fn get_character(&self) -> char {
        match self.rotation % 4 {
            0 => self.character,
            1 => {
                if self.character == '*' {
                    '+'
                } else {
                    self.character
                }
            }
            2 => {
                if self.character == '+' {
                    '*'
                } else {
                    self.character
                }
            }
            _ => self.character,
        }
    }
}

pub struct FallingLeaves {
    leaves: Vec<Leaf>,
    spawn_counter: u32,
    spawn_rate: u32,
    terminal_width: u16,
    terminal_height: u16,
}

impl FallingLeaves {
    pub fn new(terminal_width: u16, terminal_height: u16) -> Self {
        let mut rng = rand::rng();
        let initial_count = std::cmp::max(5, terminal_width / 10);

        let max_capacity = std::cmp::max(10, terminal_width / 8) as usize;
        let mut leaves = Vec::with_capacity(max_capacity);

        for _ in 0..initial_count {
            leaves.push(Leaf::new(terminal_width, false, &mut rng));
        }

        Self {
            leaves,
            spawn_counter: 0,
            spawn_rate: 15,
            terminal_width,
            terminal_height,
        }
    }

    pub fn update(&mut self, terminal_width: u16, terminal_height: u16, rng: &mut impl Rng) {
        self.terminal_width = terminal_width;
        self.terminal_height = terminal_height;

        for leaf in &mut self.leaves {
            leaf.update();
        }

        self.leaves.retain(|l| !l.is_offscreen(terminal_height));

        self.spawn_counter += 1;
        if self.spawn_counter >= self.spawn_rate {
            self.spawn_counter = 0;
            if rng.random::<f32>() < 0.7 {
                self.leaves.push(Leaf::new(terminal_width, true, rng));
            }
        }

        let max_leaves = std::cmp::max(10, terminal_width / 8) as usize;
        if self.leaves.len() > max_leaves {
            self.leaves.truncate(max_leaves);
        }
    }

    pub fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        for leaf in &self.leaves {
            let x = leaf.x as i16;
            let y = leaf.y as i16;

            if x >= 0 && y >= 0 && x < self.terminal_width as i16 && y < self.terminal_height as i16
            {
                renderer.render_char(x as u16, y as u16, leaf.get_character(), leaf.color)?;
            }
        }
        Ok(())
    }
}
