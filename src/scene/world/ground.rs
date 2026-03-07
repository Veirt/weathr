use crate::render::TerminalRenderer;
use crate::scene::SceneContext;
use crossterm::style::Color;
use std::io;

pub struct Ground;

impl Ground {
    pub fn render(
        &self,
        renderer: &mut TerminalRenderer,
        width: u16,
        height: u16,
        y_start: u16,
        ctx: &SceneContext<'_>,
    ) -> io::Result<()> {
        let is_day = ctx.conditions.is_day;
        let palette = ctx.palette;

        let width = width as usize;
        let height = height as usize;

        let grass_primary = if is_day {
            palette.ground_day
        } else {
            palette.ground_night
        };
        let grass_secondary = if is_day {
            Color::DarkGreen
        } else {
            Color::Rgb { r: 0, g: 50, b: 0 }
        };

        let flower_colors = if is_day {
            [Color::Magenta, Color::Red, Color::Cyan, Color::Yellow]
        } else {
            [
                Color::DarkMagenta,
                Color::DarkRed,
                Color::Blue,
                Color::DarkYellow,
            ]
        };

        let soil_color = if is_day {
            Color::Rgb {
                r: 101,
                g: 67,
                b: 33,
            }
        } else {
            Color::Rgb {
                r: 60,
                g: 40,
                b: 20,
            }
        };

        for y in 0..height {
            for x in 0..width {
                let (ch, color) = if y == 0 {
                    let r = pseudo_rand(x, y);
                    if r < 5 {
                        ('*', flower_colors[(x + y) % flower_colors.len()])
                    } else if r < 15 {
                        (',', grass_secondary)
                    } else {
                        ('^', grass_primary)
                    }
                } else {
                    let r = pseudo_rand(x, y);
                    let ch = if r < 20 {
                        '~'
                    } else if r < 25 {
                        '.'
                    } else {
                        ' '
                    };
                    (ch, soil_color)
                };

                renderer.render_char(x as u16, y_start + y as u16, ch, color)?;
            }
        }

        Ok(())
    }
}

fn pseudo_rand(x: usize, y: usize) -> u32 {
    ((x as u32 ^ 0x5DEECE6).wrapping_mul(y as u32 ^ 0xB)) % 100
}
