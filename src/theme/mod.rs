pub mod catalogue;

use std::collections::HashMap;
use std::fmt;

use crossterm::style::Color;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub sky_day: Color,
    pub sky_night: Color,
    pub ground_day: Color,
    pub ground_night: Color,
    pub accent_primary: Color,
    pub accent_secondary: Color,
    pub atmosphere: Option<Color>,
}

pub struct Theme {
    pub id: &'static str,
    #[allow(dead_code)]
    pub display_name: &'static str,
    pub scene_id: &'static str,
    pub overlay_id: Option<&'static str>,
    pub palette: Palette,
}

#[derive(Debug)]
pub enum ThemeError {
    NotFound(String),
    #[allow(dead_code)]
    SceneNotRegistered {
        theme: &'static str,
        scene: &'static str,
    },
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeError::NotFound(id) => write!(f, "theme '{}' is not registered", id),
            ThemeError::SceneNotRegistered { theme, scene } => {
                write!(
                    f,
                    "theme '{}' references unregistered scene '{}'",
                    theme, scene
                )
            }
        }
    }
}

pub struct ThemeRegistry {
    themes: HashMap<&'static str, Theme>,
    active: &'static str,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            themes: HashMap::new(),
            active: "default",
        };
        catalogue::register_all(&mut registry);
        registry
    }

    pub fn register(&mut self, theme: Theme) {
        self.themes.insert(theme.id, theme);
    }

    pub fn set_active(&mut self, id: &str) -> Result<(), ThemeError> {
        match self.themes.get_key_value(id) {
            Some((&static_id, _)) => {
                self.active = static_id;
                Ok(())
            }
            None => Err(ThemeError::NotFound(id.to_owned())),
        }
    }

    pub fn active(&self) -> &Theme {
        self.themes
            .get(self.active)
            .expect("active theme id must always reference a registered theme")
    }

    pub fn get(&self, id: &str) -> Option<&Theme> {
        self.themes.get(id)
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
