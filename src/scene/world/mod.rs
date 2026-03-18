mod decorations;
mod ground;
mod house;
mod style;

use crate::render::TerminalRenderer;
use crate::scene::{ChimneyPosition, Scene, SceneContext, SceneLayout};
use decorations::{DecorationLayout, Decorations};
use ground::Ground;
use house::House;
use std::io;
use style::WorldSceneStyle;

pub struct WorldScene {
    house: House,
    ground: Ground,
    decorations: Decorations,
    width: u16,
    height: u16,
}

impl WorldScene {
    const GROUND_HEIGHT: u16 = 7;

    pub fn new(width: u16, height: u16) -> Self {
        Self {
            house: House,
            ground: Ground,
            decorations: Decorations,
            width,
            height,
        }
    }
}

impl Scene for WorldScene {
    fn id(&self) -> &'static str {
        "world"
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    fn layout(&self) -> SceneLayout {
        let ground_y = self.height.saturating_sub(Self::GROUND_HEIGHT);
        let house_x = (self.width / 2).saturating_sub(House::WIDTH / 2);
        let house_y = ground_y.saturating_sub(House::HEIGHT);
        let chimney_x = house_x + House::CHIMNEY_X_OFFSET;

        SceneLayout {
            ground_y,
            chimney_pos: Some(ChimneyPosition {
                x: chimney_x,
                y: house_y,
            }),
            width: self.width,
            height: self.height,
        }
    }

    fn render(&self, renderer: &mut TerminalRenderer, ctx: &SceneContext<'_>) -> io::Result<()> {
        let layout = self.layout();
        let house_x = (self.width / 2).saturating_sub(self.house.width() / 2);
        let house_y = layout.ground_y.saturating_sub(self.house.height());
        let style = WorldSceneStyle::resolve(ctx);

        self.ground.render(
            renderer,
            self.width,
            Self::GROUND_HEIGHT,
            layout.ground_y,
            &style,
        )?;
        self.house.render(renderer, house_x, house_y, &style)?;
        self.decorations.render(
            renderer,
            &DecorationLayout {
                horizon_y: layout.ground_y,
                house_x,
                house_width: self.house.width(),
                width: self.width,
            },
            &style,
        )?;

        Ok(())
    }
}
