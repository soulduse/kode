use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::cell::Flags as CellFlags;
use alacritty_terminal::vte::ansi::NamedColor;

use kode_core::color::Color;

use crate::emulator::TerminalEmulator;

/// A renderable terminal cell.
#[derive(Debug, Clone)]
pub struct TerminalCell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::rgb(0.8, 0.8, 0.8),
            bg: Color::rgb(0.0, 0.0, 0.0),
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }
}

/// Extract visible cells from the terminal emulator.
pub fn extract_visible_cells(emu: &TerminalEmulator) -> Vec<Vec<TerminalCell>> {
    let rows = emu.rows();
    let cols = emu.cols();
    let mut result = Vec::with_capacity(rows);

    for row in 0..rows {
        let mut line = Vec::with_capacity(cols);
        for col in 0..cols {
            let cell = emu.cell(row as i32, col);
            let flags = cell.flags;

            line.push(TerminalCell {
                ch: cell.c,
                fg: ansi_color_to_color(&cell.fg),
                bg: ansi_color_to_color(&cell.bg),
                bold: flags.contains(CellFlags::BOLD),
                italic: flags.contains(CellFlags::ITALIC),
                underline: flags.contains(CellFlags::UNDERLINE)
                    || flags.contains(CellFlags::DOUBLE_UNDERLINE),
                strikethrough: flags.contains(CellFlags::STRIKEOUT),
            });
        }
        result.push(line);
    }

    result
}

fn ansi_color_to_color(color: &alacritty_terminal::vte::ansi::Color) -> Color {
    use alacritty_terminal::vte::ansi::Color as AnsiColor;
    match color {
        AnsiColor::Spec(rgb) => Color::new(
            rgb.r as f32 / 255.0,
            rgb.g as f32 / 255.0,
            rgb.b as f32 / 255.0,
            1.0,
        ),
        AnsiColor::Named(named) => named_to_color(*named),
        AnsiColor::Indexed(idx) => indexed_to_color(*idx),
    }
}

fn named_to_color(named: NamedColor) -> Color {
    match named {
        NamedColor::Black => Color::from_hex("#282828").unwrap(),
        NamedColor::Red => Color::from_hex("#cc241d").unwrap(),
        NamedColor::Green => Color::from_hex("#98971a").unwrap(),
        NamedColor::Yellow => Color::from_hex("#d79921").unwrap(),
        NamedColor::Blue => Color::from_hex("#458588").unwrap(),
        NamedColor::Magenta => Color::from_hex("#b16286").unwrap(),
        NamedColor::Cyan => Color::from_hex("#689d6a").unwrap(),
        NamedColor::White => Color::from_hex("#a89984").unwrap(),
        NamedColor::BrightBlack => Color::from_hex("#928374").unwrap(),
        NamedColor::BrightRed => Color::from_hex("#fb4934").unwrap(),
        NamedColor::BrightGreen => Color::from_hex("#b8bb26").unwrap(),
        NamedColor::BrightYellow => Color::from_hex("#fabd2f").unwrap(),
        NamedColor::BrightBlue => Color::from_hex("#83a598").unwrap(),
        NamedColor::BrightMagenta => Color::from_hex("#d3869b").unwrap(),
        NamedColor::BrightCyan => Color::from_hex("#8ec07c").unwrap(),
        NamedColor::BrightWhite => Color::from_hex("#ebdbb2").unwrap(),
        NamedColor::Foreground => Color::from_hex("#ebdbb2").unwrap(),
        NamedColor::Background => Color::from_hex("#1d2021").unwrap(),
        _ => Color::rgb(0.8, 0.8, 0.8),
    }
}

fn indexed_to_color(idx: u8) -> Color {
    // Standard 16 colors map to named
    if idx < 16 {
        let named = match idx {
            0 => NamedColor::Black,
            1 => NamedColor::Red,
            2 => NamedColor::Green,
            3 => NamedColor::Yellow,
            4 => NamedColor::Blue,
            5 => NamedColor::Magenta,
            6 => NamedColor::Cyan,
            7 => NamedColor::White,
            8 => NamedColor::BrightBlack,
            9 => NamedColor::BrightRed,
            10 => NamedColor::BrightGreen,
            11 => NamedColor::BrightYellow,
            12 => NamedColor::BrightBlue,
            13 => NamedColor::BrightMagenta,
            14 => NamedColor::BrightCyan,
            15 => NamedColor::BrightWhite,
            _ => unreachable!(),
        };
        return named_to_color(named);
    }

    // 216 color cube (indices 16-231)
    if idx < 232 {
        let idx = idx - 16;
        let r = (idx / 36) * 51;
        let g = ((idx % 36) / 6) * 51;
        let b = (idx % 6) * 51;
        return Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);
    }

    // Grayscale (indices 232-255)
    let gray = (idx - 232) * 10 + 8;
    Color::new(
        gray as f32 / 255.0,
        gray as f32 / 255.0,
        gray as f32 / 255.0,
        1.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_empty_grid() {
        let emu = TerminalEmulator::new(24, 80);
        let cells = extract_visible_cells(&emu);
        assert_eq!(cells.len(), 24);
        assert_eq!(cells[0].len(), 80);
        assert_eq!(cells[0][0].ch, ' ');
    }

    #[test]
    fn extract_after_write() {
        let mut emu = TerminalEmulator::new(24, 80);
        emu.process_bytes(b"ABC");
        let cells = extract_visible_cells(&emu);
        assert_eq!(cells[0][0].ch, 'A');
        assert_eq!(cells[0][1].ch, 'B');
        assert_eq!(cells[0][2].ch, 'C');
    }

    #[test]
    fn named_color_mapping() {
        let c = named_to_color(NamedColor::Red);
        assert!(c.r > 0.5);
    }

    #[test]
    fn indexed_color_grayscale() {
        let c = indexed_to_color(240);
        assert!(c.r > 0.0);
        assert!((c.r - c.g).abs() < f32::EPSILON);
    }
}
