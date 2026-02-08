use crate::render::TerminalRenderer;
use crossterm::style::Color;
use std::io;

#[derive(Default)]
pub struct House;

impl House {
    pub fn height(&self) -> u16 {
        self.get_ascii().len() as u16
    }

    pub fn width(&self) -> u16 {
        self.get_ascii().iter().map(|l| l.len()).max().unwrap_or(0) as u16
    }

    pub fn door_offset(&self) -> u16 {
        18 // Hardcoded based on ASCII art structure
    }

    pub fn get_ascii(&self) -> Vec<&'static str> {
        vec![
            "          (                  ",
            "                             ",
            "            )                ",
            "          ( _   _._          ",
            "           |_|-'_~_`-._      ",
            "        _.-'-_~_-~_-~-_`-._  ",
            "    _.-'_~-_~-_-~-_~_~-_~-_`-._",
            "   ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~",
            "     |  []  []   []   []  [] |",
            "     |           __    ___   |",
            "   ._|  []  []  | .|  [___]  |_._._._._._._._._._._._._._._._._.",
            "   |=|________()|__|()_______|=|=|=|=|=|=|=|=|=|=|=|=|=|=|=|=|=|",
            " ^^^^^^^^^^^^^^^ === ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^",
        ]
    }

    pub fn render(&self, renderer: &mut TerminalRenderer, x: u16, y: u16) -> io::Result<()> {
        let ascii = self.get_ascii();

        for (i, line) in ascii.iter().enumerate() {
            let row = y + i as u16;

            match i {
                0..=6 => {
                    for (j, ch) in line.chars().enumerate() {
                        let col = x + j as u16;
                        let color = if i < 4 && (ch == '(' || ch == ')' || ch == '_') {
                            Color::DarkGrey
                        } else if i < 4 {
                            Color::Grey
                        } else {
                            Color::DarkRed
                        };
                        renderer.render_char(col, row, ch, color)?;
                    }
                }
                7 => {
                    renderer.render_line_colored(x, row, line, Color::DarkRed)?;
                }
                8..=10 => {
                    for (j, ch) in line.chars().enumerate() {
                        let col = x + j as u16;
                        let color = if ch == '[' || ch == ']' {
                            Color::Cyan
                        } else if ch == '|' || ch == '.' || ch == '_' {
                            Color::Rgb {
                                r: 210,
                                g: 180,
                                b: 140,
                            }
                        } else if ch == '(' || ch == ')' {
                            Color::Rgb {
                                r: 139,
                                g: 69,
                                b: 19,
                            }
                        } else if ch == '=' {
                            Color::DarkGrey
                        } else {
                            Color::Rgb {
                                r: 210,
                                g: 180,
                                b: 140,
                            }
                        };
                        renderer.render_char(col, row, ch, color)?;
                    }
                }
                11 => {
                    for (j, ch) in line.chars().enumerate() {
                        let col = x + j as u16;
                        let color = if ch == '=' || ch == '|' {
                            Color::DarkGrey
                        } else if ch == '(' || ch == ')' {
                            Color::Rgb {
                                r: 139,
                                g: 69,
                                b: 19,
                            }
                        } else {
                            Color::Rgb {
                                r: 210,
                                g: 180,
                                b: 140,
                            }
                        };
                        renderer.render_char(col, row, ch, color)?;
                    }
                }
                12 => {
                    for (j, ch) in line.chars().enumerate() {
                        let col = x + j as u16;
                        let color = if ch == '^' {
                            Color::Green
                        } else if ch == '=' {
                            Color::DarkGrey
                        } else {
                            Color::Reset
                        };
                        renderer.render_char(col, row, ch, color)?;
                    }
                }
                _ => {
                    renderer.render_line_colored(x, row, line, Color::Yellow)?;
                }
            }
        }
        Ok(())
    }
}
