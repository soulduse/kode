use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::cell::Cell;
use alacritty_terminal::term::test::TermSize;
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi;

/// No-op event listener for the terminal.
#[derive(Copy, Clone)]
struct Listener;

impl EventListener for Listener {
    fn send_event(&self, _event: Event) {}
}

/// Terminal emulator wrapping alacritty_terminal::Term.
pub struct TerminalEmulator {
    term: Term<Listener>,
    parser: ansi::Processor,
}

impl TerminalEmulator {
    pub fn new(rows: u16, cols: u16) -> Self {
        let config = Config {
            scrolling_history: 10_000,
            ..Default::default()
        };
        let size = TermSize::new(cols as usize, rows as usize);
        let term = Term::new(config, &size, Listener);
        let parser = ansi::Processor::new();

        Self { term, parser }
    }

    /// Feed raw bytes from PTY output into the terminal state machine.
    pub fn process_bytes(&mut self, data: &[u8]) {
        self.parser.advance(&mut self.term, data);
    }

    /// Get a reference to the underlying Term.
    pub fn term(&self) -> &Term<Listener> {
        &self.term
    }

    /// Get the number of visible rows.
    pub fn rows(&self) -> usize {
        self.term.grid().screen_lines()
    }

    /// Get the number of columns.
    pub fn cols(&self) -> usize {
        self.term.grid().columns()
    }

    /// Get a cell at the given position.
    pub fn cell(&self, line: i32, col: usize) -> &Cell {
        &self.term.grid()[Line(line)][Column(col)]
    }

    /// Resize the terminal grid.
    pub fn resize(&mut self, rows: u16, cols: u16) {
        let size = TermSize::new(cols as usize, rows as usize);
        self.term.resize(size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_emulator() {
        let emu = TerminalEmulator::new(24, 80);
        assert_eq!(emu.rows(), 24);
        assert_eq!(emu.cols(), 80);
    }

    #[test]
    fn process_simple_text() {
        let mut emu = TerminalEmulator::new(24, 80);
        emu.process_bytes(b"Hello, World!");
        let cell = emu.cell(0, 0);
        assert_eq!(cell.c, 'H');
        let cell = emu.cell(0, 7);
        assert_eq!(cell.c, 'W');
    }

    #[test]
    fn resize_emulator() {
        let mut emu = TerminalEmulator::new(24, 80);
        emu.resize(40, 120);
        assert_eq!(emu.rows(), 40);
        assert_eq!(emu.cols(), 120);
    }
}
