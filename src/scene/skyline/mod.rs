pub mod cities;
pub mod generic;

use crate::render::TerminalRenderer;
use crossterm::style::Color;
use std::io;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum SkylineId {
    London,
    NewYork,
    Paris,
    Tokyo,
    Sydney,
    Dubai,
    SanFrancisco,
    Rome,
    GenericUrban,
    GenericRural,
}

pub struct Skyline {
    ascii: Vec<&'static str>,
    width: u16,
    height: u16,
}

impl Skyline {
    pub fn new(id: SkylineId) -> Self {
        let ascii = match id {
            SkylineId::London => cities::london(),
            SkylineId::NewYork => cities::new_york(),
            SkylineId::Paris => cities::paris(),
            SkylineId::Tokyo => cities::tokyo(),
            SkylineId::Sydney => cities::sydney(),
            SkylineId::Dubai => cities::dubai(),
            SkylineId::SanFrancisco => cities::san_francisco(),
            SkylineId::Rome => cities::rome(),
            SkylineId::GenericUrban => generic::urban(),
            SkylineId::GenericRural => generic::rural(),
        };

        let width = ascii.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
        let height = ascii.len() as u16;

        Self {
            ascii,
            width,
            height,
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn render(
        &self,
        renderer: &mut TerminalRenderer,
        x: u16,
        y: u16,
        is_day: bool,
    ) -> io::Result<()> {
        let color = if is_day { Color::White } else { Color::Grey };

        for (i, line) in self.ascii.iter().enumerate() {
            let row = y + i as u16;
            for (j, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    let col = x + j as u16;
                    renderer.render_char(col, row, ch, color)?;
                }
            }
        }
        Ok(())
    }
}

pub fn resolve_skyline(city: Option<&str>) -> Option<SkylineId> {
    let city = city?.to_lowercase();
    match city.as_str() {
        "london" => Some(SkylineId::London),
        "new york" | "new york city" | "nyc" | "manhattan" => Some(SkylineId::NewYork),
        "paris" => Some(SkylineId::Paris),
        "tokyo" => Some(SkylineId::Tokyo),
        "sydney" => Some(SkylineId::Sydney),
        "dubai" => Some(SkylineId::Dubai),
        "san francisco" | "sf" => Some(SkylineId::SanFrancisco),
        "rome" | "roma" => Some(SkylineId::Rome),
        _ => None,
    }
}
