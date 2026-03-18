use crate::theme::{Palette, Theme, ThemeRegistry};
use crossterm::style::Color;

pub const DEFAULT_PALETTE: Palette = Palette {
    sky_day: Color::Cyan,
    sky_night: Color::DarkBlue,
    ground_day: Color::Green,
    ground_night: Color::DarkGreen,
    accent_primary: Color::DarkRed,
    accent_secondary: Color::Rgb {
        r: 210,
        g: 180,
        b: 140,
    },
    atmosphere: None,
};

fn default_theme() -> Theme {
    Theme {
        id: "default",
        display_name: "Default",
        scene_id: "world",
        overlay_id: None,
        palette: DEFAULT_PALETTE,
    }
}

pub fn register_all(registry: &mut ThemeRegistry) {
    registry.register(default_theme());
}
