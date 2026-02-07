pub mod rain;
pub mod raindrops;
pub mod sunny;
pub mod thunderstorm;

use crate::render::TerminalRenderer;
use std::io;

pub trait Animation {
    fn get_frame(&self, frame_number: usize) -> Vec<String>;
    fn frame_count(&self) -> usize;

    #[allow(dead_code)]
    fn frame_delay_ms(&self) -> u64;
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
        renderer.render_centered(&frame, y_offset)
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
