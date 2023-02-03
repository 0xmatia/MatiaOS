use mio::unix::SourceFd;
use mio::{event, Interest, Registry, Token};
use std::io::{self, Read, Stdin, Stdout, Write};
use std::os::unix::io::RawFd;
use termios::*;

const STDIN_FILENO: RawFd = 0;

/// Represents the input
pub struct UnixStdio {
    stdin: Stdin,
    stdout: Stdout,
    old_flags: Termios,
}

impl UnixStdio {
    pub fn init() -> io::Result<Self> {
        let flags = termios_setup(STDIN_FILENO)?;
        Ok(UnixStdio {
            stdin: io::stdin(),
            stdout: io::stdout(),
            old_flags: flags,
        })
    }
}

/// Setup termios options for stdin and stdout
///
/// # Return
///
/// Return the old flags in order to restore once done
fn termios_setup(fd: RawFd) -> io::Result<Termios> {
    let old_flags = Termios::from_fd(fd)?;
    let mut new_flags = old_flags;
    new_flags.c_cflag = CS8 | CLOCAL | CREAD;
    new_flags.c_iflag = 0;
    new_flags.c_oflag = 0;
    new_flags.c_lflag = 0;
    new_flags.c_cc[VMIN] = 1;
    new_flags.c_cc[VTIME] = 0;
    tcflush(fd, TCIFLUSH)?;
    tcsetattr(fd, TCSANOW, &new_flags)?;

    Ok(old_flags)
}

/// Restore old terminal settings
impl Drop for UnixStdio {
    fn drop(&mut self) {
        tcsetattr(STDIN_FILENO, TCSANOW, &self.old_flags).ok();
    }
}

/// Implement event::source in order to be able to register this struct
impl event::Source for UnixStdio {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&STDIN_FILENO).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&STDIN_FILENO).reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        SourceFd(&STDIN_FILENO).deregister(registry)
    }
}

impl Read for UnixStdio {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdin.read(buf)
    }
}

impl Write for UnixStdio {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}
