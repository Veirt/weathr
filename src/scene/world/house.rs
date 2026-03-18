use crate::render::TerminalRenderer;
use crate::scene::world::style::WorldSceneStyle;
use std::io;

const HOUSE_ASCII: &str = include_str!("assets/house.txt");

pub struct House;

impl House {
    pub const WIDTH: u16 = 64;
    pub const HEIGHT: u16 = 10;
    pub const CHIMNEY_X_OFFSET: u16 = 12;

    pub fn width(&self) -> u16 {
        Self::WIDTH
    }

    pub fn height(&self) -> u16 {
        Self::HEIGHT
    }

    pub fn render(
        &self,
        renderer: &mut TerminalRenderer,
        x: u16,
        y: u16,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        for (i, line) in HOUSE_ASCII.lines().enumerate() {
            let row = y + i as u16;

            match i {
                // Chimney top + roof slopes
                0..=3 => {
                    for (j, ch) in line.chars().enumerate() {
                        if ch != ' ' {
                            renderer.render_char(x + j as u16, row, ch, style.roof)?;
                        }
                    }
                }
                // Roof ridge
                4 => {
                    for (j, ch) in line.chars().enumerate() {
                        if ch != ' ' {
                            renderer.render_char(x + j as u16, row, ch, style.roof)?;
                        }
                    }
                }
                // Upper and mid window rows
                5..=7 => {
                    for (j, ch) in line.chars().enumerate() {
                        if ch != ' ' {
                            let color = match ch {
                                '[' | ']' => style.window,
                                '|' | '.' | '_' => style.wood,
                                '(' | ')' => style.door,
                                '=' => style.trim,
                                _ => style.wood,
                            };
                            renderer.render_char(x + j as u16, row, ch, color)?;
                        }
                    }
                }
                // Base wall / fence
                8 => {
                    for (j, ch) in line.chars().enumerate() {
                        if ch != ' ' {
                            let color = match ch {
                                '=' | '|' => style.trim,
                                '(' | ')' => style.door,
                                _ => style.wood,
                            };
                            renderer.render_char(x + j as u16, row, ch, color)?;
                        }
                    }
                }
                // Grass / path row
                9 => {
                    for (j, ch) in line.chars().enumerate() {
                        if ch != ' ' {
                            let color = match ch {
                                '^' => style.grass_primary,
                                '=' => style.trim,
                                _ => crossterm::style::Color::Reset,
                            };
                            renderer.render_char(x + j as u16, row, ch, color)?;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
