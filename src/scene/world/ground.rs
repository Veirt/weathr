use crate::render::TerminalRenderer;
use crate::scene::world::style::WorldSceneStyle;
use std::io;

pub struct Ground;

impl Ground {
    pub fn render(
        &self,
        renderer: &mut TerminalRenderer,
        width: u16,
        height: u16,
        y_start: u16,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        let width = width as usize;
        let height = height as usize;

        for y in 0..height {
            for x in 0..width {
                let (ch, color) = if y == 0 {
                    let r = pseudo_rand(x, y);
                    if r < 5 {
                        (
                            '*',
                            style.flower_colors[(x + y) % style.flower_colors.len()],
                        )
                    } else if r < 15 {
                        (',', style.grass_secondary)
                    } else {
                        ('^', style.grass_primary)
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
                    (ch, style.soil)
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
