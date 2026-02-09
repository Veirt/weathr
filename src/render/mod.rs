mod capabilities;

use capabilities::TerminalCapabilities;
use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, BufWriter, Stdout, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Cell {
    character: char,
    color: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            character: ' ',
            color: Color::Reset,
        }
    }
}

pub struct TerminalRenderer {
    stdout: BufWriter<Stdout>,
    width: u16,
    height: u16,
    buffer: Vec<Cell>,
    last_buffer: Vec<Cell>,
    capabilities: TerminalCapabilities,
}

impl TerminalRenderer {
    pub fn new() -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        let stdout = BufWriter::new(io::stdout());
        let buffer_size = (width as usize) * (height as usize);
        let capabilities = TerminalCapabilities::detect();

        Ok(Self {
            stdout,
            width,
            height,
            buffer: vec![Cell::default(); buffer_size],
            last_buffer: vec![Cell::default(); buffer_size],
            capabilities,
        })
    }

    pub fn init(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(self.stdout, EnterAlternateScreen, cursor::Hide)?;
        Ok(())
    }

    pub fn cleanup(&mut self) -> io::Result<()> {
        execute!(self.stdout, LeaveAlternateScreen, cursor::Show, ResetColor)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn manual_resize(&mut self, width: u16, height: u16) -> io::Result<()> {
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            let buffer_size = (width as usize) * (height as usize);
            self.buffer = vec![Cell::default(); buffer_size];
            self.last_buffer = vec![Cell::default(); buffer_size];
            execute!(self.stdout, Clear(ClearType::All))?;
        }
        Ok(())
    }

    pub fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.buffer.fill(Cell::default());
        Ok(())
    }

    pub fn render_centered_colored(
        &mut self,
        lines: &[String],
        start_row: u16,
        color: Color,
    ) -> io::Result<()> {
        let max_width = lines.iter().map(|l| l.len()).max().unwrap_or(0);
        let start_col = if self.width as usize > max_width {
            (self.width as usize - max_width) / 2
        } else {
            0
        };
        let adjusted_color = self.capabilities.adjust_color(color);

        for (idx, line) in lines.iter().enumerate() {
            let row = start_row + idx as u16;
            if row < self.height {
                for (char_idx, ch) in line.chars().enumerate() {
                    let col = start_col as u16 + char_idx as u16;
                    if col < self.width {
                        let buffer_idx = (row as usize) * (self.width as usize) + (col as usize);
                        if buffer_idx < self.buffer.len() {
                            self.buffer[buffer_idx] = Cell {
                                character: ch,
                                color: adjusted_color,
                            };
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn render_line_colored(
        &mut self,
        x: u16,
        y: u16,
        text: &str,
        color: Color,
    ) -> io::Result<()> {
        if y >= self.height {
            return Ok(());
        }
        let adjusted_color = self.capabilities.adjust_color(color);

        for (idx, ch) in text.chars().enumerate() {
            let col = x + idx as u16;
            if col < self.width {
                let buffer_idx = (y as usize) * (self.width as usize) + (col as usize);
                if buffer_idx < self.buffer.len() {
                    self.buffer[buffer_idx] = Cell {
                        character: ch,
                        color: adjusted_color,
                    };
                }
            }
        }
        Ok(())
    }

    pub fn render_char(&mut self, x: u16, y: u16, ch: char, color: Color) -> io::Result<()> {
        if x < self.width && y < self.height {
            let buffer_idx = (y as usize) * (self.width as usize) + (x as usize);
            if buffer_idx < self.buffer.len() {
                self.buffer[buffer_idx] = Cell {
                    character: ch,
                    color: self.capabilities.adjust_color(color),
                };
            }
        }
        Ok(())
    }

    pub fn flash_screen(&mut self) -> io::Result<()> {
        let flash_color = self.capabilities.adjust_color(Color::White);
        for cell in &mut self.buffer {
            cell.color = flash_color;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let mut current_color = Color::Reset;
        let mut last_pos: Option<(u16, u16)> = None;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);

                if idx >= self.buffer.len() || idx >= self.last_buffer.len() {
                    continue;
                }

                let cell = self.buffer[idx];
                let last_cell = self.last_buffer[idx];

                if cell != last_cell {
                    let expected_pos = last_pos.map(|(lx, ly)| (lx + 1, ly));
                    if expected_pos != Some((x, y)) {
                        queue!(self.stdout, cursor::MoveTo(x, y))?;
                    }

                    if cell.color != current_color {
                        queue!(self.stdout, SetForegroundColor(cell.color))?;
                        current_color = cell.color;
                    }

                    queue!(self.stdout, Print(cell.character))?;
                    last_pos = Some((x, y));
                }
            }
        }

        if current_color != Color::Reset {
            queue!(self.stdout, ResetColor)?;
        }

        self.stdout.flush()?;
        self.last_buffer.copy_from_slice(&self.buffer);
        Ok(())
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
