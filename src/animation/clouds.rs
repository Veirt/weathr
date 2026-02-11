use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::prelude::*;
use std::io;
use std::sync::OnceLock;

static CLOUD_SHAPES: OnceLock<Vec<Vec<String>>> = OnceLock::new();

struct Cloud {
    x: f32,
    y: f32,
    speed: f32,
    shape: Vec<String>,
    color: Color,
}

pub struct CloudSystem {
    clouds: Vec<Cloud>,
    terminal_width: u16,
    terminal_height: u16,
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
}

impl CloudSystem {
    pub fn new(terminal_width: u16, terminal_height: u16) -> Self {
        let mut rng = rand::rng();
        // Add a few initial clouds
        let count = std::cmp::max(1, terminal_width / 20);

        let max_capacity = (terminal_width / 20) as usize;
        let mut clouds = Vec::with_capacity(max_capacity);

        for _ in 0..count {
            clouds.push(Self::create_random_cloud(
                terminal_width,
                terminal_height,
                true,
                Color::White,
                &mut rng,
            ));
        }

        Self {
            clouds,
            terminal_width,
            terminal_height,
        }
    }

    fn create_random_cloud(
        width: u16,
        height: u16,
        random_x: bool,
        color: Color,
        rng: &mut impl Rng,
    ) -> Cloud {
        let shapes = CLOUD_SHAPES.get_or_init(Self::create_cloud_shapes);

        let shape_idx = (rng.random::<u32>() as usize) % shapes.len();
        let shape = shapes[shape_idx].clone();

        let y_range = height / 3;
        let y = (rng.random::<u16>() % std::cmp::max(1, y_range)) as f32;

        let x = if random_x {
            (rng.random::<u16>() % width) as f32
        } else {
            -(shape[0].len() as f32)
        };

        let speed = 0.05 + (rng.random::<f32>() * 0.1);

        Cloud {
            x,
            y,
            speed,
            shape,
            color,
        }
    }

    fn create_cloud_shapes() -> Vec<Vec<String>> {
        let shapes = [
            vec![
                "   .--.   ".to_string(),
                " .-(    ). ".to_string(),
                "(___.__)_)".to_string(),
            ],
            vec![
                "      _  _   ".to_string(),
                "    ( `   )_ ".to_string(),
                "   (    )    `)".to_string(),
                "    \\_  (___  )".to_string(),
            ],
            vec![
                "     .--.    ".to_string(),
                "  .-(    ).  ".to_string(),
                " (___.__)__) ".to_string(),
            ],
            vec![
                "   _  _   ".to_string(),
                "  ( `   )_ ".to_string(),
                " (    )   `)".to_string(),
                "  `--'     ".to_string(),
            ],
        ];

        shapes.to_vec()
    }

    pub fn update(
        &mut self,
        terminal_width: u16,
        terminal_height: u16,
        is_clear: bool,
        cloud_color: Color,
        rng: &mut impl Rng,
    ) {
        self.terminal_width = terminal_width;
        self.terminal_height = terminal_height;

        for cloud in &mut self.clouds {
            cloud.x += cloud.speed;
        }

        self.clouds.retain(|c| c.x < terminal_width as f32);

        let max_clouds = if is_clear {
            (terminal_width / 40) as usize
        } else {
            (terminal_width / 20) as usize
        };

        let spawn_chance = if is_clear { 0.002 } else { 0.005 };

        if self.clouds.len() < max_clouds && rng.random::<f32>() < spawn_chance {
            self.clouds.push(Self::create_random_cloud(
                terminal_width,
                terminal_height,
                false,
                cloud_color,
                rng,
            ));
        }
    }

    pub fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        for cloud in &self.clouds {
            for (i, line) in cloud.shape.iter().enumerate() {
                let y = cloud.y as i16 + i as i16;
                let x = cloud.x as i16;

                if y >= 0 && y < self.terminal_height as i16 {
                    renderer.render_line_colored(
                        std::cmp::max(0, x) as u16,
                        y as u16,
                        line,
                        cloud.color,
                    )?;
                }
            }
        }
        Ok(())
    }
}
