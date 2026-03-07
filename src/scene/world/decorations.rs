use crate::render::TerminalRenderer;
use crate::scene::SceneContext;
use crossterm::style::Color;
use std::io;

pub struct Decorations;

pub struct DecorationLayout {
    pub horizon_y: u16,
    pub house_x: u16,
    pub house_width: u16,
    pub width: u16,
}

impl Decorations {
    pub fn render(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        ctx: &SceneContext<'_>,
    ) -> io::Result<()> {
        let is_day = ctx.conditions.is_day;

        self.render_tree(renderer, layout, is_day)?;
        self.render_fence(renderer, layout, is_day)?;
        self.render_mailbox(renderer, layout, is_day)?;

        if layout.width > 120 {
            self.render_pine_tree(renderer, layout, is_day)?;
        }

        Ok(())
    }

    fn render_tree(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        is_day: bool,
    ) -> io::Result<()> {
        let (lines, color) = Self::tree_art(is_day);
        let tree_x = layout.house_x.saturating_sub(20);
        if tree_x == 0 {
            return Ok(());
        }
        let tree_y = layout.horizon_y.saturating_sub(lines.len() as u16);
        render_art(renderer, lines, tree_x, tree_y, color)
    }

    fn render_fence(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        is_day: bool,
    ) -> io::Result<()> {
        let (lines, color) = Self::fence_art(is_day);
        let fence_x = layout.house_x + layout.house_width + 2;
        if fence_x >= layout.width {
            return Ok(());
        }
        let fence_y = layout.horizon_y.saturating_sub(lines.len() as u16);
        render_art(renderer, lines, fence_x, fence_y, color)
    }

    fn render_mailbox(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        is_day: bool,
    ) -> io::Result<()> {
        let tree_x = layout.house_x.saturating_sub(20);
        let (lines, color) = Self::mailbox_art(is_day);
        let mailbox_x = tree_x.saturating_sub(10);
        if mailbox_x >= layout.width {
            return Ok(());
        }
        let mailbox_y = layout.horizon_y.saturating_sub(lines.len() as u16);
        render_art(renderer, lines, mailbox_x, mailbox_y, color)
    }

    fn render_pine_tree(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        is_day: bool,
    ) -> io::Result<()> {
        let (lines, color) = Self::pine_tree_art(is_day);
        let pine_x = layout.house_x + layout.house_width + 18;
        if pine_x + 10 >= layout.width {
            return Ok(());
        }
        let pine_y = layout.horizon_y.saturating_sub(lines.len() as u16);
        render_art(renderer, lines, pine_x, pine_y, color)
    }

    fn tree_art(is_day: bool) -> (&'static [&'static str], Color) {
        (
            &[
                "      ####      ",
                "    ########    ",
                "   ##########   ",
                "    ########    ",
                "      _||_      ",
            ],
            if is_day {
                Color::DarkGreen
            } else {
                Color::Rgb { r: 0, g: 50, b: 0 }
            },
        )
    }

    fn fence_art(is_day: bool) -> (&'static [&'static str], Color) {
        (
            &["|--|--|--|--|", "|  |  |  |  |"],
            if is_day { Color::White } else { Color::Grey },
        )
    }

    fn mailbox_art(is_day: bool) -> (&'static [&'static str], Color) {
        (
            &[" ___ ", "|___|", "  |  "],
            if is_day { Color::Blue } else { Color::DarkBlue },
        )
    }

    fn pine_tree_art(is_day: bool) -> (&'static [&'static str], Color) {
        (
            &[
                "    *    ",
                "   ***   ",
                "  *****  ",
                " ******* ",
                "   |||   ",
            ],
            if is_day {
                Color::DarkGreen
            } else {
                Color::Rgb { r: 0, g: 50, b: 0 }
            },
        )
    }
}

fn render_art(
    renderer: &mut TerminalRenderer,
    lines: &[&str],
    x: u16,
    y: u16,
    color: Color,
) -> io::Result<()> {
    for (i, line) in lines.iter().enumerate() {
        for (j, ch) in line.chars().enumerate() {
            if ch != ' ' {
                renderer.render_char(x + j as u16, y + i as u16, ch, color)?;
            }
        }
    }
    Ok(())
}
