use kode_core::color::{Color, ThemeColors};
use ratatui::style::{self, Modifier, Style};

/// Convert a kode Color (f32 RGBA) to a ratatui Color (u8 RGB).
pub fn to_ratatui_color(color: &Color) -> style::Color {
    style::Color::Rgb(
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8,
    )
}

/// Theme styles for TUI rendering.
pub struct ThemeStyles {
    pub background: Style,
    pub foreground: Style,
    pub cursor: Style,
    pub selection: Style,
    pub line_highlight: Style,
    pub gutter: Style,
    pub gutter_active: Style,
    pub keyword: Style,
    pub string: Style,
    pub comment: Style,
    pub function: Style,
    pub type_name: Style,
    pub number: Style,
    pub error: Style,
    pub warning: Style,
    pub info: Style,
    pub mode_normal: Style,
    pub mode_insert: Style,
    pub mode_visual: Style,
    pub mode_command: Style,
    pub border_focused: Style,
    pub border_unfocused: Style,
    pub tab_active: Style,
    pub tab_inactive: Style,
}

/// Create TUI theme styles from theme colors.
pub fn theme_styles(theme: &ThemeColors) -> ThemeStyles {
    ThemeStyles {
        background: Style::default().bg(to_ratatui_color(&theme.background)),
        foreground: Style::default().fg(to_ratatui_color(&theme.foreground)),
        cursor: Style::default()
            .fg(to_ratatui_color(&theme.background))
            .bg(to_ratatui_color(&theme.cursor)),
        selection: Style::default().bg(to_ratatui_color(&theme.selection)),
        line_highlight: Style::default().bg(to_ratatui_color(&theme.line_highlight)),
        gutter: Style::default().fg(to_ratatui_color(&theme.gutter)),
        gutter_active: Style::default().fg(to_ratatui_color(&theme.gutter_active)),
        keyword: Style::default().fg(to_ratatui_color(&theme.keyword)),
        string: Style::default().fg(to_ratatui_color(&theme.string)),
        comment: Style::default().fg(to_ratatui_color(&theme.comment)),
        function: Style::default().fg(to_ratatui_color(&theme.function)),
        type_name: Style::default().fg(to_ratatui_color(&theme.type_name)),
        number: Style::default().fg(to_ratatui_color(&theme.number)),
        error: Style::default()
            .fg(to_ratatui_color(&theme.error))
            .add_modifier(Modifier::BOLD),
        warning: Style::default().fg(to_ratatui_color(&theme.warning)),
        info: Style::default().fg(to_ratatui_color(&theme.info)),
        mode_normal: Style::default()
            .fg(style::Color::Black)
            .bg(style::Color::Green)
            .add_modifier(Modifier::BOLD),
        mode_insert: Style::default()
            .fg(style::Color::Black)
            .bg(style::Color::Blue)
            .add_modifier(Modifier::BOLD),
        mode_visual: Style::default()
            .fg(style::Color::Black)
            .bg(style::Color::Yellow)
            .add_modifier(Modifier::BOLD),
        mode_command: Style::default()
            .fg(style::Color::Black)
            .bg(style::Color::Magenta)
            .add_modifier(Modifier::BOLD),
        border_focused: Style::default().fg(style::Color::Cyan),
        border_unfocused: Style::default().fg(style::Color::DarkGray),
        tab_active: Style::default()
            .fg(style::Color::White)
            .add_modifier(Modifier::BOLD),
        tab_inactive: Style::default().fg(style::Color::DarkGray),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_conversion() {
        let c = Color::rgb(1.0, 0.5, 0.0);
        let rc = to_ratatui_color(&c);
        assert_eq!(rc, style::Color::Rgb(255, 127, 0));
    }

    #[test]
    fn theme_creates_styles() {
        let theme = ThemeColors::default();
        let styles = theme_styles(&theme);
        // Just verify it doesn't panic and produces a non-default style
        assert_ne!(styles.keyword, Style::default());
    }
}
