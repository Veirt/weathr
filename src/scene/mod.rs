pub mod decorations;
pub mod ground;
pub mod house;

use crate::render::TerminalRenderer;
use crate::weather::WeatherConditions;
use std::io;

pub struct WorldScene {
    house: house::House,
    ground: ground::Ground,
    decorations: decorations::Decorations,
    width: u16,
    height: u16,
}

impl WorldScene {
    pub const GROUND_HEIGHT: u16 = 8;

    pub fn new(width: u16, height: u16) -> Self {
        let house = house::House;
        let ground = ground::Ground;
        let decorations = decorations::Decorations::new();

        Self {
            house,
            ground,
            decorations,
            width,
            height,
        }
    }

    pub fn update_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    pub fn render(
        &self,
        renderer: &mut TerminalRenderer,
        conditions: &WeatherConditions,
    ) -> io::Result<()> {
        let horizon_y = self.height.saturating_sub(Self::GROUND_HEIGHT);

        // House position
        let house_width = self.house.width();
        let house_height = self.house.height();
        let house_x = (self.width / 2).saturating_sub(house_width / 2);
        let house_y = horizon_y.saturating_sub(house_height);

        // Door/Path alignment
        let door_offset = self.house.door_offset();
        let path_center = house_x + door_offset;

        // Render Ground
        self.ground.render(
            renderer,
            self.width,
            Self::GROUND_HEIGHT,
            horizon_y,
            path_center,
            conditions.is_day,
        )?;

        // Render House
        self.house
            .render(renderer, house_x, house_y, conditions.is_day)?;

        // Render Decorations
        self.decorations.render(
            renderer,
            &crate::scene::decorations::DecorationRenderConfig {
                horizon_y,
                house_x,
                house_width,
                path_center,
                width: self.width,
                is_day: conditions.is_day,
            },
        )?;

        Ok(())
    }
}
