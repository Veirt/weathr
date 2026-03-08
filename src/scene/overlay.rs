use crate::render::TerminalRenderer;
use crate::scene::{SceneContext, SceneLayout};
use std::collections::HashMap;
use std::io;

#[allow(dead_code)]
pub trait SceneOverlay: Send + Sync {
    fn id(&self) -> &'static str;
    fn update_size(&mut self, width: u16, height: u16);
    /// render decoration on top of the base scene.
    fn render(
        &self,
        renderer: &mut TerminalRenderer,
        ctx: &SceneContext<'_>,
        layout: &SceneLayout,
    ) -> io::Result<()>;
}

pub struct OverlayRegistry {
    overlays: HashMap<&'static str, Box<dyn SceneOverlay>>,
}

impl OverlayRegistry {
    pub fn new() -> Self {
        Self {
            overlays: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn register(&mut self, overlay: Box<dyn SceneOverlay>) {
        self.overlays.insert(overlay.id(), overlay);
    }

    pub fn get(&self, id: &str) -> Option<&dyn SceneOverlay> {
        self.overlays.get(id).map(|b| b.as_ref())
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, id: &str) -> Option<&mut dyn SceneOverlay> {
        self.overlays
            .get_mut(id)
            .map(|b| -> &mut dyn SceneOverlay { b.as_mut() })
    }
}

impl Default for OverlayRegistry {
    fn default() -> Self {
        Self::new()
    }
}
