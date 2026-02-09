use crate::animation::{
    AnimationController, birds::BirdSystem, chimney::ChimneySmoke, clouds::CloudSystem,
    fireflies::FireflySystem, leaves::FallingLeaves, moon::MoonSystem, raindrops::RaindropSystem,
    snow::SnowSystem, stars::StarSystem, sunny::SunnyAnimation, thunderstorm::ThunderstormSystem,
};
use crate::app_state::AppState;
use crate::render::TerminalRenderer;
use crate::scene::WorldScene;
use crate::scene::house::House;
use crate::weather::WeatherConditions;
use crate::weather::{RainIntensity, SnowIntensity};
use std::io;
use std::time::{Duration, Instant};

const FRAME_DELAY: Duration = Duration::from_millis(500);

pub struct AnimationManager {
    raindrop_system: RaindropSystem,
    snow_system: SnowSystem,
    thunderstorm_system: ThunderstormSystem,
    cloud_system: CloudSystem,
    bird_system: BirdSystem,
    star_system: StarSystem,
    moon_system: MoonSystem,
    chimney_smoke: ChimneySmoke,
    firefly_system: FireflySystem,
    falling_leaves: FallingLeaves,
    sunny_animation: SunnyAnimation,
    animation_controller: AnimationController,
    last_frame_time: Instant,
    show_leaves: bool,
}

impl AnimationManager {
    pub fn new(term_width: u16, term_height: u16, show_leaves: bool) -> Self {
        Self {
            raindrop_system: RaindropSystem::new(term_width, term_height, RainIntensity::Light),
            snow_system: SnowSystem::new(term_width, term_height, SnowIntensity::Light),
            thunderstorm_system: ThunderstormSystem::new(term_width, term_height),
            cloud_system: CloudSystem::new(term_width, term_height),
            bird_system: BirdSystem::new(term_width, term_height),
            star_system: StarSystem::new(term_width, term_height),
            moon_system: MoonSystem::new(term_width, term_height),
            chimney_smoke: ChimneySmoke::new(),
            firefly_system: FireflySystem::new(term_width, term_height),
            falling_leaves: FallingLeaves::new(term_width, term_height),
            sunny_animation: SunnyAnimation::new(),
            animation_controller: AnimationController::new(),
            last_frame_time: Instant::now(),
            show_leaves,
        }
    }

    pub fn update_rain_intensity(&mut self, intensity: RainIntensity) {
        self.raindrop_system.set_intensity(intensity);
    }

    pub fn update_snow_intensity(&mut self, intensity: SnowIntensity) {
        self.snow_system.set_intensity(intensity);
    }

    pub fn render_background(
        &mut self,
        renderer: &mut TerminalRenderer,
        conditions: &WeatherConditions,
        state: &AppState,
        term_width: u16,
        term_height: u16,
    ) -> io::Result<()> {
        if !conditions.is_day {
            self.star_system.update(term_width, term_height);
            self.star_system.render(renderer)?;
            self.moon_system.update(term_width, term_height);
            self.moon_system.render(renderer)?;

            if state.should_show_fireflies() {
                self.firefly_system.update(term_width, term_height);
                self.firefly_system.render(renderer)?;
            }
        }

        if conditions.is_cloudy
            || (!conditions.is_raining && !conditions.is_thunderstorm && !conditions.is_snowing)
        {
            if conditions.is_cloudy {
                self.cloud_system.update(term_width, term_height);
                self.cloud_system.render(renderer)?;
            }

            if !conditions.is_raining
                && !conditions.is_thunderstorm
                && !conditions.is_snowing
                && conditions.is_day
            {
                self.bird_system.update(term_width, term_height);
                self.bird_system.render(renderer)?;
            }
        }

        if state.should_show_sun()
            && !conditions.is_raining
            && !conditions.is_thunderstorm
            && !conditions.is_snowing
        {
            let animation_y = if term_height > 20 { 3 } else { 2 };
            self.animation_controller
                .render_frame(renderer, &self.sunny_animation, animation_y)?;
        }

        Ok(())
    }

    pub fn render_chimney_smoke(
        &mut self,
        renderer: &mut TerminalRenderer,
        conditions: &WeatherConditions,
        term_width: u16,
        term_height: u16,
    ) -> io::Result<()> {
        if conditions.is_raining || conditions.is_thunderstorm {
            return Ok(());
        }

        let ground_height = WorldScene::GROUND_HEIGHT;
        let horizon_y = term_height.saturating_sub(ground_height);
        let house_width = House::WIDTH;
        let house_height = House::HEIGHT;
        let house_x = (term_width / 2).saturating_sub(house_width / 2);
        let house_y = horizon_y.saturating_sub(house_height);
        let chimney_x = house_x + House::CHIMNEY_X_OFFSET;
        let chimney_y = house_y;

        self.chimney_smoke.update(chimney_x, chimney_y);
        self.chimney_smoke.render(renderer)?;

        Ok(())
    }

    pub fn render_foreground(
        &mut self,
        renderer: &mut TerminalRenderer,
        conditions: &WeatherConditions,
        term_width: u16,
        term_height: u16,
    ) -> io::Result<()> {
        if conditions.is_thunderstorm {
            self.raindrop_system.update(term_width, term_height);
            self.raindrop_system.render(renderer)?;

            self.thunderstorm_system.update(term_width, term_height);
            self.thunderstorm_system.render(renderer)?;

            if self.thunderstorm_system.is_flashing() {
                renderer.flash_screen()?;
            }
        } else if conditions.is_raining {
            self.raindrop_system.update(term_width, term_height);
            self.raindrop_system.render(renderer)?;
        } else if conditions.is_snowing {
            self.snow_system.update(term_width, term_height);
            self.snow_system.render(renderer)?;
        }

        if self.show_leaves
            && !conditions.is_raining
            && !conditions.is_thunderstorm
            && !conditions.is_snowing
        {
            self.falling_leaves.update(term_width, term_height);
            self.falling_leaves.render(renderer)?;
        }

        Ok(())
    }

    pub fn update_sunny_animation(&mut self, conditions: &WeatherConditions) {
        if !conditions.is_raining
            && !conditions.is_thunderstorm
            && !conditions.is_snowing
            && self.last_frame_time.elapsed() >= FRAME_DELAY
        {
            self.animation_controller.next_frame(&self.sunny_animation);
            self.last_frame_time = Instant::now();
        }
    }
}
