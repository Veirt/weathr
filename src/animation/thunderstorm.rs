use crate::render::TerminalRenderer;
use crossterm::style::Color;
use std::io;

struct Raindrop {
    x: u16,
    y: f32,
    speed: f32,
    character: char,
}

struct Lightning {
    segments: Vec<LightningSegment>,
    frames_remaining: u8,
    flash_intensity: u8,
}

struct LightningSegment {
    x: u16,
    y: u16,
    character: char,
}

impl Lightning {
    fn new(terminal_width: u16, terminal_height: u16) -> Self {
        let x_start = (terminal_width / 4) + (rand::random::<u16>() % (terminal_width / 2));
        let mut segments = Vec::new();

        let mut x = x_start;
        let mut y = 2;

        let pattern = rand::random::<u8>() % 3;

        match pattern {
            0 => {
                while y < terminal_height.saturating_sub(10) && segments.len() < 15 {
                    segments.push(LightningSegment {
                        x,
                        y,
                        character: '|',
                    });
                    y += 1;
                    if rand::random::<u8>() % 3 == 0 && x > 1 {
                        x = x.saturating_sub(1);
                        segments.push(LightningSegment {
                            x,
                            y,
                            character: '/',
                        });
                    } else if rand::random::<u8>() % 3 == 0 && x < terminal_width - 2 {
                        x += 1;
                        segments.push(LightningSegment {
                            x,
                            y,
                            character: '\\',
                        });
                    }
                    y += 1;
                }
            }
            1 => {
                while y < terminal_height.saturating_sub(10) && segments.len() < 12 {
                    segments.push(LightningSegment {
                        x,
                        y,
                        character: '!',
                    });
                    y += 1;
                    if rand::random::<bool>() && x > 2 {
                        x = x.saturating_sub(1);
                    } else if x < terminal_width - 3 {
                        x += 1;
                    }
                    y += 1;
                }
            }
            _ => {
                let main_height = terminal_height.saturating_sub(12);
                for i in 0..main_height {
                    segments.push(LightningSegment {
                        x,
                        y: 2 + i,
                        character: '|',
                    });
                }

                if x > 3 {
                    for i in 0..4 {
                        segments.push(LightningSegment {
                            x: x.saturating_sub(i + 1),
                            y: 5 + i,
                            character: '/',
                        });
                    }
                }
                if x < terminal_width - 4 {
                    for i in 0..3 {
                        segments.push(LightningSegment {
                            x: x + i + 1,
                            y: 8 + i,
                            character: '\\',
                        });
                    }
                }
            }
        }

        Self {
            segments,
            frames_remaining: 3,
            flash_intensity: 2,
        }
    }

    fn update(&mut self) {
        if self.frames_remaining > 0 {
            self.frames_remaining -= 1;
        }
        if self.flash_intensity > 0 {
            self.flash_intensity -= 1;
        }
    }

    fn is_active(&self) -> bool {
        self.frames_remaining > 0
    }

    fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        let color = if self.flash_intensity > 0 {
            Color::White
        } else {
            Color::Yellow
        };

        for segment in &self.segments {
            renderer.render_char(segment.x, segment.y, segment.character, color)?;
        }

        Ok(())
    }
}

pub struct ThunderstormSystem {
    drops: Vec<Raindrop>,
    lightning: Option<Lightning>,
    terminal_width: u16,
    terminal_height: u16,
    frames_since_lightning: u16,
    next_lightning_in: u16,
}

impl ThunderstormSystem {
    pub fn new(terminal_width: u16, terminal_height: u16) -> Self {
        let drop_count = (terminal_width as usize * terminal_height as usize) / 30;
        let mut drops = Vec::with_capacity(drop_count);

        let characters = ['|', '|', '\'', '.', '`'];

        for i in 0..drop_count {
            drops.push(Raindrop {
                x: (i as u16 * 7) % terminal_width,
                y: ((i as f32 * 3.7) % terminal_height as f32),
                speed: 0.2 + ((i % 7) as f32 * 0.1),
                character: characters[i % characters.len()],
            });
        }

        let next_lightning_in = 60 + (rand::random::<u16>() % 180);

        Self {
            drops,
            lightning: None,
            terminal_width,
            terminal_height,
            frames_since_lightning: 0,
            next_lightning_in,
        }
    }

    pub fn update(&mut self, terminal_width: u16, terminal_height: u16) {
        if self.terminal_width != terminal_width || self.terminal_height != terminal_height {
            *self = Self::new(terminal_width, terminal_height);
            return;
        }

        for drop in &mut self.drops {
            drop.y += drop.speed;

            if drop.y as u16 >= terminal_height {
                drop.y = 0.0;
                drop.x = (drop.x as usize * 13 + 7) as u16 % terminal_width;
            }
        }

        if let Some(ref mut lightning) = self.lightning {
            lightning.update();
            if !lightning.is_active() {
                self.lightning = None;
                self.frames_since_lightning = 0;
                self.next_lightning_in = 80 + (rand::random::<u16>() % 200);
            }
        } else {
            self.frames_since_lightning += 1;
            if self.frames_since_lightning >= self.next_lightning_in {
                self.lightning = Some(Lightning::new(terminal_width, terminal_height));
            }
        }
    }

    pub fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        for drop in &self.drops {
            let y = drop.y as u16;
            if y < self.terminal_height && drop.x < self.terminal_width {
                renderer.render_char(drop.x, y, drop.character, Color::Cyan)?;
            }
        }

        if let Some(ref lightning) = self.lightning {
            lightning.render(renderer)?;
        }

        Ok(())
    }
}
