use crate::render::TerminalRenderer;
use crate::scene::SceneContext;
use crossterm::style::Color;
use std::io;

const DOOR_COLOR: Color = Color::Rgb {
    r: 139,
    g: 69,
    b: 19,
};
const DOOR_COLOR_NIGHT: Color = Color::Rgb {
    r: 80,
    g: 40,
    b: 10,
};

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

    fn ascii() -> &'static [&'static str] {
        &[
            "            _   _._          ",
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
        ctx: &SceneContext<'_>,
    ) -> io::Result<()> {
        let is_day = ctx.conditions.is_day;
        let palette = ctx.palette;

        let wood_color = if is_day {
            palette.accent_secondary
        } else {
            Color::Rgb {
                r: 100,
                g: 70,
                b: 50,
            }
        };
        let roof_color = if is_day {
            palette.accent_primary
        } else {
            Color::DarkMagenta
        };
        let door_color = if is_day { DOOR_COLOR } else { DOOR_COLOR_NIGHT };
        let window_color = if is_day { Color::Cyan } else { Color::Yellow };
        let grass_color = if is_day {
            palette.ground_day
        } else {
            palette.ground_night
        };

        for (i, line) in Self::ascii().iter().enumerate() {
            let row = y + i as u16;

            match i {
                // Chimney top + roof slopes
                0..=3 => {
                    for (j, ch) in line.chars().enumerate() {
                        if ch != ' ' {
                            renderer.render_char(x + j as u16, row, ch, roof_color)?;
                        }
                    }
                }
                // Roof ridge
                4 => {
                    for (j, ch) in line.chars().enumerate() {
                        if ch != ' ' {
                            renderer.render_char(x + j as u16, row, ch, roof_color)?;
                        }
                    }
                }
                // Upper and mid window rows
                5..=7 => {
                    for (j, ch) in line.chars().enumerate() {
                        if ch != ' ' {
                            let color = match ch {
                                '[' | ']' => window_color,
                                '|' | '.' | '_' => wood_color,
                                '(' | ')' => door_color,
                                '=' => Color::DarkGrey,
                                _ => wood_color,
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
                                '=' | '|' => Color::DarkGrey,
                                '(' | ')' => door_color,
                                _ => wood_color,
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
                                '^' => grass_color,
                                '=' => Color::DarkGrey,
                                _ => Color::Reset,
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
