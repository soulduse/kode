use serde::{Deserialize, Serialize};

/// RGBA color with f32 components (0.0 - 1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::new(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    1.0,
                ))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::new(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                ))
            }
            _ => None,
        }
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

/// Syntax highlighting color groups.
pub struct ThemeColors {
    pub background: Color,
    pub foreground: Color,
    pub cursor: Color,
    pub selection: Color,
    pub line_highlight: Color,
    pub gutter: Color,
    pub gutter_active: Color,
    pub comment: Color,
    pub keyword: Color,
    pub string: Color,
    pub number: Color,
    pub function: Color,
    pub type_name: Color,
    pub variable: Color,
    pub operator: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            background: Color::from_hex("#1e1e2e").unwrap(),
            foreground: Color::from_hex("#cdd6f4").unwrap(),
            cursor: Color::from_hex("#f5e0dc").unwrap(),
            selection: Color::from_hex("#45475a").unwrap(),
            line_highlight: Color::from_hex("#313244").unwrap(),
            gutter: Color::from_hex("#6c7086").unwrap(),
            gutter_active: Color::from_hex("#cdd6f4").unwrap(),
            comment: Color::from_hex("#6c7086").unwrap(),
            keyword: Color::from_hex("#cba6f7").unwrap(),
            string: Color::from_hex("#a6e3a1").unwrap(),
            number: Color::from_hex("#fab387").unwrap(),
            function: Color::from_hex("#89b4fa").unwrap(),
            type_name: Color::from_hex("#f9e2af").unwrap(),
            variable: Color::from_hex("#cdd6f4").unwrap(),
            operator: Color::from_hex("#89dceb").unwrap(),
            error: Color::from_hex("#f38ba8").unwrap(),
            warning: Color::from_hex("#f9e2af").unwrap(),
            info: Color::from_hex("#89b4fa").unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_hex_6() {
        let c = Color::from_hex("#ff0000").unwrap();
        assert!((c.r - 1.0).abs() < f32::EPSILON);
        assert!(c.g.abs() < f32::EPSILON);
    }

    #[test]
    fn from_hex_8() {
        let c = Color::from_hex("#ff000080").unwrap();
        assert!((c.a - 128.0 / 255.0).abs() < 0.01);
    }

    #[test]
    fn to_array() {
        let c = Color::rgb(1.0, 0.5, 0.0);
        let arr = c.to_array();
        assert_eq!(arr, [1.0, 0.5, 0.0, 1.0]);
    }
}
