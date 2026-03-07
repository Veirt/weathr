use crate::animation::{
    AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize, Wind,
};
use crate::render::TerminalRenderer;
use crossterm::style::Color;

use rand::{Rng, RngExt};
use std::io;
use std::sync::OnceLock;

const CLOUD_SHAPE_SRCS: [&str; 4] = [
    include_str!("assets/cloud_0.txt"),
    include_str!("assets/cloud_1.txt"),
    include_str!("assets/cloud_2.txt"),
    include_str!("assets/cloud_3.txt"),
];

static CLOUD_SHAPES: OnceLock<Vec<Vec<String>>> = OnceLock::new();

fn cloud_shapes() -> &'static Vec<Vec<String>> {
    CLOUD_SHAPES.get_or_init(|| {
        CLOUD_SHAPE_SRCS
            .iter()
            .map(|src| src.lines().map(|l| l.to_string()).collect())
            .collect()
    })
}

struct Cloud {
    x: f32,
    y: f32,
    speed: f32,
    wind_x: f32,
    shape: Vec<String>,
    color: Color,
}

pub struct CloudSystem {
    clouds: Vec<Cloud>,
    terminal_width: u16,
    terminal_height: u16,
    base_wind_x: f32,
}

impl CloudSystem {
    pub fn set_cloud_color(&mut self, is_clear: bool) {
        let color = if is_clear {
            Color::White
        } else {
            Color::DarkGrey
        };

        for cloud in &mut self.clouds {
            cloud.color = color;
        }
    }

    pub fn set_wind(&mut self, speed_kmh: f32, direction_deg: f32) {
        let direction_rad = direction_deg.to_radians();
        self.base_wind_x = (speed_kmh / 50.0) * (-direction_rad.sin());
        let mut rng = rand::rng();
        for cloud in &mut self.clouds {
            cloud.wind_x = self.base_wind_x * (0.8 + rng.random::<f32>() * 0.4);
        }
    }
}

impl CloudSystem {
    pub fn new(terminal_width: u16, terminal_height: u16) -> Self {
        let mut rng = rand::rng();
        let base_wind_x = 0.15;

        // Add few initial clouds
        let count = std::cmp::max(1, terminal_width / 30) as usize;
        let segment = terminal_width as f32 / count as f32;

        let mut clouds = Vec::with_capacity(count);

        for i in 0..count {
            let x_min = (i as f32 * segment) as u16;
            let x_max = ((i as f32 + 1.0) * segment) as u16;
            let x = rng.random_range(x_min..=x_max) as f32;
            clouds.push(Self::create_random_cloud(
                x,
                terminal_height,
                Color::White,
                base_wind_x,
                &mut rng,
            ));
        }

        Self {
            clouds,
            terminal_width,
            terminal_height,
            base_wind_x,
        }
    }

    fn create_random_cloud(
        x: f32,
        height: u16,
        color: Color,
        base_wind_x: f32,
        rng: &mut (impl Rng + ?Sized),
    ) -> Cloud {
        let shapes = cloud_shapes();

        let shape_idx = rng.random_range(0..shapes.len());
        let shape = shapes[shape_idx].clone();

        let y_range = (height / 3).max(1);
        let y = rng.random_range(0..y_range) as f32;
        let speed = 0.02 + (rng.random::<f32>() * 0.03);
        let wind_x = base_wind_x * (0.8 + rng.random::<f32>() * 0.4);

        Cloud {
            x,
            y,
            speed,
            wind_x,
            shape,
            color,
        }
    }

    pub fn update(
        &mut self,
        terminal_width: u16,
        terminal_height: u16,
        is_clear: bool,
        cloud_color: Color,
        rng: &mut (impl Rng + ?Sized),
    ) {
        self.terminal_width = terminal_width;
        self.terminal_height = terminal_height;

        for cloud in &mut self.clouds {
            cloud.x += cloud.speed + cloud.wind_x;
        }

        let width_f = terminal_width as f32;
        self.clouds.retain(|cloud| {
            let cloud_width = cloud.shape.iter().map(|line| line.len()).max().unwrap_or(0) as f32;
            let drift_x = cloud.speed + cloud.wind_x;

            if drift_x >= 0.0 {
                cloud.x < width_f
            } else {
                cloud.x + cloud_width > 0.0
            }
        });

        let max_clouds = if is_clear {
            (terminal_width / 30) as usize
        } else {
            (terminal_width / 20) as usize
        };

        let spawn_chance = if is_clear { 0.002 } else { 0.005 };

        if self.clouds.len() < max_clouds && rng.random::<f32>() < spawn_chance {
            let mut cloud =
                Self::create_random_cloud(0.0, terminal_height, cloud_color, self.base_wind_x, rng);
            let cloud_width = cloud.shape.iter().map(|line| line.len()).max().unwrap_or(0) as f32;

            let drift_x = cloud.speed + cloud.wind_x;
            let spawn_from_left = drift_x >= 0.0;
            let min_gap = (terminal_width as f32 / 8.0).max(15.0);
            let too_close = if spawn_from_left {
                self.clouds.iter().any(|c| c.x < min_gap)
            } else {
                self.clouds.iter().any(|c| c.x > (width_f - min_gap))
            };

            if !too_close {
                cloud.x = if spawn_from_left {
                    -cloud_width
                } else {
                    width_f
                };
                self.clouds.push(cloud);
            }
        }
    }

    pub fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        for cloud in &self.clouds {
            for (i, line) in cloud.shape.iter().enumerate() {
                let y = cloud.y as i16 + i as i16;
                let x = cloud.x as i16;

                if y < 0 || y >= self.terminal_height as i16 {
                    continue;
                }

                let clip = ((-x).max(0)) as usize;
                let visible = &line[clip.min(line.len())..];

                if !visible.is_empty() {
                    renderer.render_line_colored(
                        x.max(0) as u16,
                        y as u16,
                        visible,
                        cloud.color,
                    )?;
                }
            }
        }
        Ok(())
    }
}

impl AnimationSystem for CloudSystem {
    fn id(&self) -> &'static str {
        "clouds"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::Background
    }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        let is_clear = ctx
            .state
            .current_weather
            .as_ref()
            .is_some_and(|w| matches!(w.condition, crate::weather::WeatherCondition::Clear));

        ctx.conditions.is_cloudy || is_clear
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.terminal_width = size.width;
        self.terminal_height = size.height;
    }

    fn on_wind(&mut self, wind: Wind) {
        self.set_wind(wind.speed_kmh, wind.direction_deg);
    }

    fn update(&mut self, ctx: &FrameContext<'_>, rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        let (is_clear, cloud_color) = if let Some(weather) = &ctx.state.current_weather {
            match weather.condition {
                crate::weather::WeatherCondition::Clear => (true, Color::White),
                crate::weather::WeatherCondition::PartlyCloudy => (false, Color::Grey),
                _ => (false, Color::DarkGrey),
            }
        } else {
            (false, Color::DarkGrey)
        };

        self.set_cloud_color(is_clear);
        self.update(ctx.size.width, ctx.size.height, is_clear, cloud_color, rng);
    }

    fn render(
        &mut self,
        renderer: &mut TerminalRenderer,
        _ctx: &FrameContext<'_>,
    ) -> io::Result<()> {
        CloudSystem::render(self, renderer)
    }
}
