use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer};
use crate::render::TerminalRenderer;
use chrono::{Local, Timelike};
use crossterm::style::Color;
use rand::Rng;
use std::io;

const FONT_HEIGHT: u16 = 5;
const GLYPH_SPACING: u16 = 1;
const CLOCK_COLOR: Color = Color::White;
const AMPM_COLOR: Color = Color::White;

/// Rows above `horizon_y` (the ground line) occupied by the house. Mirrors
/// `House::HEIGHT` in `src/scene/world/house.rs`; the clock stays above this so
/// it never covers the house or the bottom-of-frame effects.
const HOUSE_CLEARANCE: u16 = 10;

/// Renders the current local time as large ASCII-art digits in the sky.
///
/// Lives in the `Foreground` layer and is registered last, so it draws on top
/// of weather effects while remaining confined to the sky band.
pub struct TimeDisplaySystem {
    twelve_hour: bool,
}

impl TimeDisplaySystem {
    pub fn new(twelve_hour: bool) -> Self {
        Self { twelve_hour }
    }

    /// The 5-row block-font glyph for a supported character, or `None`.
    fn glyph(ch: char) -> Option<[&'static str; FONT_HEIGHT as usize]> {
        Some(match ch {
            '0' => ["████", "█  █", "█  █", "█  █", "████"],
            '1' => ["  █ ", " ██ ", "  █ ", "  █ ", " ███"],
            '2' => ["████", "   █", "████", "█   ", "████"],
            '3' => ["████", "   █", " ███", "   █", "████"],
            '4' => ["█  █", "█  █", "████", "   █", "   █"],
            '5' => ["████", "█   ", "████", "   █", "████"],
            '6' => ["████", "█   ", "████", "█  █", "████"],
            '7' => ["████", "   █", "  █ ", " █  ", " █  "],
            '8' => ["████", "█  █", "████", "█  █", "████"],
            '9' => ["████", "█  █", "████", "   █", "████"],
            ':' => [" ", "█", " ", "█", " "],
            _ => return None,
        })
    }

    fn glyph_width(ch: char) -> u16 {
        match ch {
            ':' => 1,
            _ => 4,
        }
    }

    /// Total rendered width (in columns) of `text` in the block font.
    fn rendered_width(text: &str) -> u16 {
        let mut width = 0u16;
        for (i, ch) in text.chars().enumerate() {
            if i > 0 {
                width += GLYPH_SPACING;
            }
            width += Self::glyph_width(ch);
        }
        width
    }

    /// Builds the `HH:MM` string and, in 12-hour mode, the AM/PM suffix.
    fn format_time(&self, hour: u32, minute: u32) -> (String, Option<&'static str>) {
        if self.twelve_hour {
            let suffix = if hour < 12 { "AM" } else { "PM" };
            let h12 = match hour % 12 {
                0 => 12,
                h => h,
            };
            (format!("{:02}:{:02}", h12, minute), Some(suffix))
        } else {
            (format!("{:02}:{:02}", hour, minute), None)
        }
    }
}

impl AnimationSystem for TimeDisplaySystem {
    fn id(&self) -> &'static str {
        "time_display"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::Foreground
    }

    fn update(&mut self, _ctx: &FrameContext<'_>, _rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        // Time is read fresh in `render`; nothing to advance here.
    }

    fn render(&mut self, renderer: &mut TerminalRenderer, ctx: &FrameContext<'_>) -> io::Result<()> {
        let now = Local::now();
        let (text, ampm) = self.format_time(now.hour(), now.minute());

        // Sky band is rows 0..sky_bottom; the house occupies the rows below it.
        let sky_bottom = ctx.horizon_y.saturating_sub(HOUSE_CLEARANCE);
        // Need room for the art plus a one-row top margin; otherwise skip
        // entirely rather than clip into the house (e.g. very short terminals).
        if sky_bottom < FONT_HEIGHT + 1 {
            return Ok(());
        }

        let art_width = Self::rendered_width(&text);
        if art_width >= ctx.size.width {
            return Ok(());
        }

        let start_x = (ctx.size.width - art_width) / 2;
        // Center vertically within the usable sky band (rows 1..sky_bottom).
        let top_row = 1 + (sky_bottom - 1 - FONT_HEIGHT) / 2;

        // Blit each glyph, treating spaces as transparent so the sky and
        // weather effects show through the gaps.
        let mut cursor_x = start_x;
        for ch in text.chars() {
            if let Some(glyph) = Self::glyph(ch) {
                for (row, line) in glyph.iter().enumerate() {
                    let y = top_row + row as u16;
                    for (col, cell) in line.chars().enumerate() {
                        if cell != ' ' {
                            renderer.render_char(cursor_x + col as u16, y, cell, CLOCK_COLOR)?;
                        }
                    }
                }
            }
            cursor_x += Self::glyph_width(ch) + GLYPH_SPACING;
        }

        // AM/PM in plain text at the top-right corner of the art block.
        if let Some(suffix) = ampm {
            let suffix_x = start_x + art_width + 1;
            if suffix_x + suffix.len() as u16 <= ctx.size.width {
                renderer.render_line_colored(suffix_x, top_row, suffix, AMPM_COLOR)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_24_hour() {
        let sys = TimeDisplaySystem::new(false);
        assert_eq!(sys.format_time(0, 5), ("00:05".to_string(), None));
        assert_eq!(sys.format_time(13, 9), ("13:09".to_string(), None));
        assert_eq!(sys.format_time(23, 59), ("23:59".to_string(), None));
    }

    #[test]
    fn format_12_hour() {
        let sys = TimeDisplaySystem::new(true);
        assert_eq!(sys.format_time(0, 0), ("12:00".to_string(), Some("AM")));
        assert_eq!(sys.format_time(11, 30), ("11:30".to_string(), Some("AM")));
        assert_eq!(sys.format_time(12, 0), ("12:00".to_string(), Some("PM")));
        assert_eq!(sys.format_time(13, 45), ("01:45".to_string(), Some("PM")));
        assert_eq!(sys.format_time(23, 1), ("11:01".to_string(), Some("PM")));
    }

    #[test]
    fn rendered_width_matches_layout() {
        // 5 glyphs (4,4,1,4,4) + 4 spacings = 21 columns.
        assert_eq!(TimeDisplaySystem::rendered_width("12:34"), 21);
    }
}
