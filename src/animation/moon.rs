use crate::render::TerminalRenderer;
use crossterm::style::Color;
use std::io;

pub struct MoonSystem {
    phase: f64, // 0.0 = New, 0.25 = First Quarter, 0.5 = Full, 0.75 = Last Quarter
    x: u16,
    y: u16,
}

impl MoonSystem {
    pub fn new(terminal_width: u16, terminal_height: u16, phase: Option<f64>) -> Self {
        Self {
            phase: phase.unwrap_or(0.5),
            x: (terminal_width / 4) + 10,
            y: (terminal_height / 4) + 2,
        }
    }

    pub fn set_phase(&mut self, phase: f64) {
        self.phase = phase;
    }

    pub fn update(&mut self, terminal_width: u16, terminal_height: u16) {
        self.x = (terminal_width / 4 * 3).min(terminal_width.saturating_sub(15));
        self.y = (terminal_height / 4).max(2);
    }

    pub fn render(&self, renderer: &mut TerminalRenderer) -> io::Result<()> {
        let step = (self.phase * 8.0).round() as usize % 8;

        let art = match step {
            0 => vec![
                // NEW MOON (Invisible)
                "                 ",
                "                 ",
                "                 ",
                "                 ",
                "                 ",
                "                 ",
            ],
            1 => vec![
                // WAXING CRESCENT (Thin, mostly edge)
                "             .    ",
                "            . `.  ",
                "               :  ",
                "               :  ",
                "            . .'  ",
                "             `    ",
            ],
            2 => vec![
                // FIRST QUARTER (Right Half - Solid)
                "            _     ",
                "           |~ `.  ",
                "           |~~~~: ",
                "           |~~~~: ",
                "           |~ .'  ",
                "           |-'    ",
            ],
            3 => vec![
                // WAXING GIBBOUS (Mostly full, textured)
                "         ..._     ",
                "       .'~~~~`.   ",
                "      |~~~~o~~~:  ",
                "      |~.~~~~o~:  ",
                "       `.~~~~~'   ",
                "         `...-'   ",
            ],
            4 => vec![
                // FULL MOON (Full Circle, textured with craters)
                "       _..._      ",
                "     .'~o~~~`.    ",
                "    :~~~~~o~~~:   ",
                "    :~~o~~~~.~:   ",
                "    `.~~~~~o~.'   ",
                "      `-...-'     ",
            ],
            5 => vec![
                // WANING GIBBOUS
                "       _...       ",
                "     .'~~~~`.     ",
                "    :~~~o~~~~|    ",
                "    :~o~~~~.~|    ",
                "    `.~~~~~.'     ",
                "      `-...-'     ",
            ],
            6 => vec![
                // LAST QUARTER
                "        _         ",
                "      .' ~|       ",
                "     :~~~~|       ",
                "     :~~~~|       ",
                "      `.~ |       ",
                "        `-|       ",
            ],
            7 => vec![
                // WANING CRESCENT
                "        .         ",
                "      .' .        ",
                "     :            ",
                "     :            ",
                "      '. .        ",
                "        `         ",
            ],
            _ => vec![],
        };

        for (i, line) in art.iter().enumerate() {
            let y = self.y + i as u16;
            for (j, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue; // Transparent (Sky)
                }

                let x = self.x + j as u16;

                if ch == '~' {
                    // Opaque Moon Body (hides stars) - Render as space but overwrite what's there
                    renderer.render_char(x, y, ' ', Color::White)?;
                } else {
                    // Texture/Outline
                    renderer.render_char(x, y, ch, Color::White)?;
                }
            }
        }
        Ok(())
    }
}
