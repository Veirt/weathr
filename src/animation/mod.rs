pub mod birds;
pub mod chimney;
pub mod clouds;
pub mod fireflies;
pub mod fog;
pub mod leaves;
pub mod moon;
pub mod raindrops;
pub mod snow;
pub mod stars;
pub mod sunny;
pub mod thunderstorm;

use crate::render::TerminalRenderer;
use crossterm::style::Color;
use std::io;

pub trait Animation {
    fn get_frame(&self, frame_number: usize) -> &[String];
    fn frame_count(&self) -> usize;

    fn get_color(&self) -> Color {
        Color::Reset
    }
}

pub struct AnimationController {
    current_frame: usize,
}

impl AnimationController {
    pub fn new() -> Self {
        Self { current_frame: 0 }
    }

    pub fn next_frame<A: Animation>(&mut self, animation: &A) -> usize {
        self.current_frame = (self.current_frame + 1) % animation.frame_count();
        self.current_frame
    }

    pub fn render_frame<A: Animation>(
        &self,
        renderer: &mut TerminalRenderer,
        animation: &A,
        y_offset: u16,
    ) -> io::Result<()> {
        let frame = animation.get_frame(self.current_frame);
        let color = animation.get_color();
        renderer.render_centered_colored(frame, y_offset, color)
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.current_frame = 0;
    }
}

impl Default for AnimationController {
    fn default() -> Self {
        Self::new()
    }
}
