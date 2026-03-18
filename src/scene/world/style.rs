use crate::scene::SceneContext;
use crossterm::style::Color;

#[derive(Clone, Copy)]
pub struct WorldSceneStyle {
    pub roof: Color,
    pub wood: Color,
    pub door: Color,
    pub window: Color,
    pub trim: Color,
    pub grass_primary: Color,
    pub grass_secondary: Color,
    pub flower_colors: [Color; 4],
    pub soil: Color,
    pub tree_foliage: Color,
    pub fence: Color,
    pub mailbox: Color,
}

impl WorldSceneStyle {
    pub fn resolve(ctx: &SceneContext<'_>) -> Self {
        let palette = ctx.palette;

        if ctx.conditions.sun.is_day {
            Self {
                roof: palette.accent_primary,
                wood: palette.accent_secondary,
                door: Color::Rgb {
                    r: 139,
                    g: 69,
                    b: 19,
                },
                window: Color::Cyan,
                trim: Color::DarkGrey,
                grass_primary: palette.ground_day,
                grass_secondary: Color::DarkGreen,
                flower_colors: [Color::Magenta, Color::Red, Color::Cyan, Color::Yellow],
                soil: Color::Rgb {
                    r: 101,
                    g: 67,
                    b: 33,
                },
                tree_foliage: Color::DarkGreen,
                fence: Color::White,
                mailbox: Color::Blue,
            }
        } else {
            Self {
                roof: Color::DarkMagenta,
                wood: Color::Rgb {
                    r: 100,
                    g: 70,
                    b: 50,
                },
                door: Color::Rgb {
                    r: 80,
                    g: 40,
                    b: 10,
                },
                window: Color::Yellow,
                trim: Color::DarkGrey,
                grass_primary: palette.ground_night,
                grass_secondary: Color::Rgb { r: 0, g: 50, b: 0 },
                flower_colors: [
                    Color::DarkMagenta,
                    Color::DarkRed,
                    Color::Blue,
                    Color::DarkYellow,
                ],
                soil: Color::Rgb {
                    r: 60,
                    g: 40,
                    b: 20,
                },
                tree_foliage: Color::Rgb { r: 0, g: 50, b: 0 },
                fence: Color::Grey,
                mailbox: Color::DarkBlue,
            }
        }
    }
}
