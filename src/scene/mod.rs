pub mod overlay;
pub mod world;

use crate::render::TerminalRenderer;
use crate::theme::Palette;
use crate::weather::WeatherConditions;
use std::collections::HashMap;
use std::io;

pub struct SceneContext<'a> {
    pub conditions: &'a WeatherConditions,
    pub palette: &'a Palette,
}

#[derive(Clone, Copy)]
pub struct SceneLayout {
    pub ground_y: u16,
    pub chimney_pos: Option<ChimneyPosition>,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy)]
pub struct ChimneyPosition {
    pub x: u16,
    pub y: u16,
}

pub trait Scene: Send + Sync {
    fn id(&self) -> &'static str;
    fn update_size(&mut self, width: u16, height: u16);
    fn render(&self, renderer: &mut TerminalRenderer, ctx: &SceneContext<'_>) -> io::Result<()>;
    fn layout(&self) -> SceneLayout;
}

pub struct SceneRegistry {
    scenes: HashMap<&'static str, Box<dyn Scene>>,
}

impl SceneRegistry {
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
        }
    }

    pub fn register(&mut self, scene: Box<dyn Scene>) {
        self.scenes.insert(scene.id(), scene);
    }

    pub fn get(&self, id: &str) -> Option<&dyn Scene> {
        self.scenes.get(id).map(|scene| scene.as_ref())
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut dyn Scene> {
        self.scenes
            .get_mut(id)
            .map(|scene| -> &mut dyn Scene { scene.as_mut() })
    }
}

impl Default for SceneRegistry {
    fn default() -> Self {
        Self::new()
    }
}
