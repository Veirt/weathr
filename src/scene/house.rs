use crate::render::TerminalRenderer;
use crossterm::style::Color;
use std::io;

const WOOD_COLOR: Color = Color::Rgb {
    r: 210,
    g: 180,
    b: 140,
};
const DOOR_COLOR: Color = Color::Rgb {
    r: 139,
    g: 69,
    b: 19,
};

#[derive(Default)]
pub struct House;

impl House {
    pub const WIDTH: u16 = 64;
    pub const HEIGHT: u16 = 13;
    pub const DOOR_OFFSET: u16 = 18;
    pub const CHIMNEY_X_OFFSET: u16 = 10;

    pub fn height(&self) -> u16 {
        Self::HEIGHT
    }

    pub fn width(&self) -> u16 {
        Self::WIDTH
    }

    pub fn door_offset(&self) -> u16 {
        Self::DOOR_OFFSET
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

    pub fn render(
        &self,
        renderer: &mut TerminalRenderer,
        x: u16,
        y: u16,
        is_day: bool,
    ) -> io::Result<()> {
        let ascii = self.get_ascii();

        let wood_color = if is_day {
            WOOD_COLOR
        } else {
            Color::Rgb {
                r: 100,
                g: 70,
                b: 50,
            }
        };
        let roof_color = if is_day {
            Color::DarkRed
        } else {
            Color::DarkMagenta
        };
        let window_color = if is_day { Color::Cyan } else { Color::Yellow };

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
                            roof_color
                        };
                        renderer.render_char(col, row, ch, color)?;
                    }
                }
                7 => {
                    renderer.render_line_colored(x, row, line, roof_color)?;
                }
                8..=10 => {
                    for (j, ch) in line.chars().enumerate() {
                        let col = x + j as u16;
                        let color = if ch == '[' || ch == ']' {
                            window_color
                        } else if ch == '|' || ch == '.' || ch == '_' {
                            wood_color
                        } else if ch == '(' || ch == ')' {
                            DOOR_COLOR
                        } else if ch == '=' {
                            Color::DarkGrey
                        } else {
                            wood_color
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
                            DOOR_COLOR
                        } else {
                            wood_color
                        };
                        renderer.render_char(col, row, ch, color)?;
                    }
                }
                12 => {
                    for (j, ch) in line.chars().enumerate() {
                        let col = x + j as u16;
                        let color = if ch == '^' {
                            if is_day {
                                Color::Green
                            } else {
                                Color::DarkGreen
                            }
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
