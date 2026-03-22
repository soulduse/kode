use std::ffi::CString;
use std::io;
use std::os::fd::AsRawFd;
use std::path::Path;

use nix::pty::{openpty, OpenptyResult};
use nix::unistd::{self, ForkResult};

/// Pseudoterminal (PTY) managing a child shell process.
pub struct Pty {
    master_fd: i32,
    child_pid: nix::unistd::Pid,
}

impl Pty {
    /// Spawn a new PTY with a shell process.
    pub fn spawn(shell: &str, cwd: &Path, rows: u16, cols: u16) -> io::Result<Self> {
        let OpenptyResult { master, slave } = openpty(None, None)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let master_fd = master.as_raw_fd();
        let slave_fd = slave.as_raw_fd();

        // Set initial window size
        set_window_size(master_fd, rows, cols)?;

        // Fork
        match unsafe { unistd::fork() } {
            Ok(ForkResult::Child) => {
                drop(master);

                // Create new session
                let _ = unistd::setsid();

                // Set slave as controlling terminal
                unsafe {
                    libc::ioctl(slave_fd, libc::TIOCSCTTY as _, 0);
                }

                // Redirect stdio to slave PTY
                unsafe {
                    libc::dup2(slave_fd, 0); // stdin
                    libc::dup2(slave_fd, 1); // stdout
                    libc::dup2(slave_fd, 2); // stderr
                }
                drop(slave);

                // Change directory
                let cwd_cstr = CString::new(cwd.to_str().unwrap_or("/")).unwrap();
                unsafe {
                    libc::chdir(cwd_cstr.as_ptr());
                }

                // Exec shell
                let shell_cstr = CString::new(shell).unwrap();
                let args = [shell_cstr.clone()];
                let _ = unistd::execvp(&shell_cstr, &args);
                std::process::exit(1);
            }
            Ok(ForkResult::Parent { child }) => {
                drop(slave);

                // Set master to non-blocking
                unsafe {
                    let flags = libc::fcntl(master_fd, libc::F_GETFL);
                    libc::fcntl(master_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                }

                // Leak the OwnedFd so it doesn't get closed
                std::mem::forget(master);

                Ok(Self {
                    master_fd,
                    child_pid: child,
                })
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }

    /// Read from the PTY master.
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe { libc::read(self.master_fd, buf.as_mut_ptr() as *mut _, buf.len()) };
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }

    /// Write to the PTY master.
    pub fn write_all(&self, data: &[u8]) -> io::Result<()> {
        let mut offset = 0;
        while offset < data.len() {
            let ret = unsafe {
                libc::write(
                    self.master_fd,
                    data[offset..].as_ptr() as *const _,
                    data.len() - offset,
                )
            };
            if ret < 0 {
                return Err(io::Error::last_os_error());
            }
            offset += ret as usize;
        }
        Ok(())
    }

    /// Resize the PTY window.
    pub fn resize(&self, rows: u16, cols: u16) {
        let _ = set_window_size(self.master_fd, rows, cols);
    }

    pub fn child_pid(&self) -> nix::unistd::Pid {
        self.child_pid
    }

    pub fn master_fd(&self) -> i32 {
        self.master_fd
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        // Send SIGHUP to child
        let _ = nix::sys::signal::kill(self.child_pid, nix::sys::signal::Signal::SIGHUP);
        // Close master fd
        unsafe {
            libc::close(self.master_fd);
        }
    }
}

fn set_window_size(fd: i32, rows: u16, cols: u16) -> io::Result<()> {
    let ws = libc::winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let ret = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, &ws) };
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_and_drop() {
        let pty = Pty::spawn("/bin/echo", Path::new("/"), 24, 80);
        assert!(pty.is_ok());
        let pty = pty.unwrap();
        assert!(pty.master_fd() > 0);
    }
}
