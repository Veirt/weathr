use crate::render::TerminalRenderer;
use crossterm::style::Color;
use std::io;

#[derive(Default)]
pub struct Ground;

impl Ground {
    pub fn render(
        &self,
        renderer: &mut TerminalRenderer,
        width: u16,
        height: u16,
        y_start: u16,
        path_center: u16,
        is_day: bool,
    ) -> io::Result<()> {
        let width = width as usize;
        let height = height as usize;
        let path_center = path_center as usize;

        let grass_colors = if is_day {
            [Color::Green, Color::DarkGreen]
        } else {
            [Color::DarkGreen, Color::Rgb { r: 0, g: 50, b: 0 }]
        };

        let flower_colors = if is_day {
            vec![Color::Magenta, Color::Red, Color::Cyan, Color::Yellow]
        } else {
            vec![
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

        let path_color = if is_day {
            Color::Rgb {
                r: 180,
                g: 160,
                b: 120,
            }
        } else {
            Color::Rgb {
                r: 100,
                g: 90,
                b: 70,
            }
        };

        // Simple seeded RNG for deterministic decoration positions
        fn pseudo_rand(x: usize, y: usize) -> u32 {
            ((x as u32 ^ 0x5DEECE6).wrapping_mul(y as u32 ^ 0xB)) % 100
        }

        for y in 0..height {
            let path_width = 4 + y; // Widens as it comes closer
            let path_start = path_center.saturating_sub(path_width / 2);
            let path_end = path_center + path_width / 2;

            for x in 0..width {
                let is_path = x >= path_start && x <= path_end;

                let (ch, color) = if y == 0 {
                    // Top layer: Grass/Flowers and Path
                    if is_path {
                        ('=', path_color)
                    } else {
                        // Grass with random flowers
                        let r = pseudo_rand(x, y);
                        if r < 5 {
                            // 5% chance of flower
                            let f_idx = (x + y) % flower_colors.len();
                            ('*', flower_colors[f_idx])
                        } else if r < 15 {
                            // 10% chance of distinct grass blade
                            (',', grass_colors[1])
                        } else {
                            ('^', grass_colors[0])
                        }
                    }
                } else {
                    // Lower layers: Soil/Rocks and Path
                    if is_path {
                        ('=', path_color)
                    } else {
                        // Soil pattern
                        let r = pseudo_rand(x, y);
                        let ch = if r < 20 {
                            '~'
                        } else if r < 25 {
                            '.'
                        } else {
                            ' '
                        };
                        (ch, soil_color)
                    }
                };

                renderer.render_char(x as u16, y_start + y as u16, ch, color)?;
            }
        }
        Ok(())
    }
}
