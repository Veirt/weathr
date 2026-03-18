use crate::render::TerminalRenderer;
use crate::scene::world::style::WorldSceneStyle;
use std::io;

const TREE_ASCII: &str = include_str!("assets/tree.txt");
const FENCE_ASCII: &str = include_str!("assets/fence.txt");
const MAILBOX_ASCII: &str = include_str!("assets/mailbox.txt");
const PINE_TREE_ASCII: &str = include_str!("assets/pine_tree.txt");

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
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        self.render_tree(renderer, layout, style)?;
        self.render_fence(renderer, layout, style)?;
        self.render_mailbox(renderer, layout, style)?;

        if layout.width > 120 {
            self.render_pine_tree(renderer, layout, style)?;
        }

        Ok(())
    }

    fn render_tree(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        let tree_x = layout.house_x.saturating_sub(20);
        if tree_x == 0 {
            return Ok(());
        }
        let line_count = TREE_ASCII.lines().count() as u16;
        let tree_y = layout.horizon_y.saturating_sub(line_count);
        render_art(renderer, TREE_ASCII, tree_x, tree_y, style.tree_foliage)
    }

    fn render_fence(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        let fence_x = layout.house_x + layout.house_width + 2;
        if fence_x >= layout.width {
            return Ok(());
        }
        let line_count = FENCE_ASCII.lines().count() as u16;
        let fence_y = layout.horizon_y.saturating_sub(line_count);
        render_art(renderer, FENCE_ASCII, fence_x, fence_y, style.fence)
    }

    fn render_mailbox(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        let tree_x = layout.house_x.saturating_sub(20);
        let Some(mailbox_x) = tree_x.checked_sub(10) else {
            return Ok(());
        };
        if mailbox_x >= layout.width {
            return Ok(());
        }
        let line_count = MAILBOX_ASCII.lines().count() as u16;
        let mailbox_y = layout.horizon_y.saturating_sub(line_count);
        render_art(renderer, MAILBOX_ASCII, mailbox_x, mailbox_y, style.mailbox)
    }

    fn render_pine_tree(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        let pine_x = layout.house_x + layout.house_width + 18;
        if pine_x + 10 >= layout.width {
            return Ok(());
        }
        let line_count = PINE_TREE_ASCII.lines().count() as u16;
        let pine_y = layout.horizon_y.saturating_sub(line_count);
        render_art(
            renderer,
            PINE_TREE_ASCII,
            pine_x,
            pine_y,
            style.tree_foliage,
        )
    }
}

fn render_art(
    renderer: &mut TerminalRenderer,
    ascii: &str,
    x: u16,
    y: u16,
    color: crossterm::style::Color,
) -> io::Result<()> {
    for (i, line) in ascii.lines().enumerate() {
        for (j, ch) in line.chars().enumerate() {
            if ch != ' ' {
                renderer.render_char(x + j as u16, y + i as u16, ch, color)?;
            }
        }
    }
    Ok(())
}
