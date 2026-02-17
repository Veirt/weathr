pub mod decorations;
pub mod ground;
pub mod house;
pub mod skyline;

use crate::render::TerminalRenderer;
use crate::weather::WeatherConditions;
use std::io;

enum SceneMode {
    /// Default house + decorations (town/suburban)
    Town,
    /// City-specific or generic skyline
    Skyline(skyline::Skyline),
}

pub struct WorldScene {
    mode: SceneMode,
    house: house::House,
    ground: ground::Ground,
    decorations: decorations::Decorations,
    width: u16,
    height: u16,
}

impl WorldScene {
    pub const GROUND_HEIGHT: u16 = 7;

    pub fn new(width: u16, height: u16, city: Option<&str>) -> Self {
        let mode = if let Some(id) = skyline::resolve_skyline(city) {
            SceneMode::Skyline(skyline::Skyline::new(id))
        } else {
            SceneMode::Town
        };

        Self {
            mode,
            house: house::House,
            ground: ground::Ground,
            decorations: decorations::Decorations::new(),
            width,
            height,
        }
    }

    pub fn has_chimney(&self) -> bool {
        matches!(self.mode, SceneMode::Town)
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

        // Ground always renders
        self.ground.render(
            renderer,
            self.width,
            Self::GROUND_HEIGHT,
            horizon_y,
            conditions.is_day,
        )?;

        match &self.mode {
            SceneMode::Town => {
                let house_width = self.house.width();
                let house_height = self.house.height();
                let house_x = (self.width / 2).saturating_sub(house_width / 2);
                let house_y = horizon_y.saturating_sub(house_height);

                self.house
                    .render(renderer, house_x, house_y, conditions.is_day)?;

                self.decorations.render(
                    renderer,
                    &decorations::DecorationRenderConfig {
                        horizon_y,
                        house_x,
                        house_width,
                        width: self.width,
                        is_day: conditions.is_day,
                    },
                )?;
            }
            SceneMode::Skyline(s) => {
                let sky_x = (self.width / 2).saturating_sub(s.width() / 2);
                let sky_y = horizon_y.saturating_sub(s.height());
                s.render(renderer, sky_x, sky_y, conditions.is_day)?;
            }
        }

        Ok(())
    }
}
