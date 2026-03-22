pub mod emulator;
pub mod grid;
pub mod input;
pub mod pty;

use std::io;

use kode_core::error::{KodeError, KodeResult};

use crate::emulator::TerminalEmulator;
use crate::pty::Pty;

/// A terminal instance combining PTY and terminal emulator.
pub struct Terminal {
    pub id: usize,
    pub pty: Pty,
    pub emulator: TerminalEmulator,
    pub cwd: std::path::PathBuf,
}

impl Terminal {
    /// Spawn a new terminal with the user's default shell.
    pub fn spawn(id: usize, rows: u16, cols: u16) -> KodeResult<Self> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
        let cwd = std::env::current_dir().unwrap_or_else(|_| "/".into());

        let pty = Pty::spawn(&shell, &cwd, rows, cols)
            .map_err(|e| KodeError::Other(format!("PTY spawn failed: {}", e)))?;

        let emulator = TerminalEmulator::new(rows, cols);

        Ok(Self {
            id,
            pty,
            emulator,
            cwd,
        })
    }

    /// Read available data from PTY and feed into emulator.
    pub fn process_output(&mut self) -> io::Result<bool> {
        let mut buf = [0u8; 4096];
        match self.pty.read(&mut buf) {
            Ok(0) => Ok(false), // EOF
            Ok(n) => {
                self.emulator.process_bytes(&buf[..n]);
                Ok(true)
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(true),
            Err(e) => Err(e),
        }
    }

    /// Write input data to PTY.
    pub fn write_input(&mut self, data: &[u8]) -> io::Result<()> {
        self.pty.write_all(data)
    }

    /// Resize the terminal.
    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.pty.resize(rows, cols);
        self.emulator.resize(rows, cols);
    }
}
