use crate::stdio::UnixStdio;
use anyhow::{bail, Result};
use mio::{Events, Interest, Poll, Token};
use mio_serial::SerialStream;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

const SERIAL_TOKEN: Token = Token(0);
const STDIN_TOKEN: Token = Token(1);

const NUM_BREAKS_TRIGGER_KERNEL_PUSH: usize = 3;
const KERNEL_PUSH_SIGNAL: u8 = 3;
const KERNEL_SIZE_CHUNKS: u32 = 4;

const CTRL_CHARACTER: u8 = 0x01;
const CARTRIDGE_RETURN: u8 = b'\r';
const EXIT_CHAR: u8 = b'x';

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SessionState {
    /// While waiting for loader to send something
    WaitingForLoader,
    /// While sending the kernel to loader
    SendingKernel,
    /// Echo mode - the main mode right now
    EchoMode,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Action {
    /// Do nothing
    Proceed,
    /// Exit pusher
    Exit,
}

/// Represent an active serial session
pub struct SerialSession {
    serial_buffer: [u8; 1024],
    /// Stdin buffer
    stdin_buffer: [u8; 1024],
    /// Serial stream of the RPi
    serial_stream: SerialStream,
    /// Represents the output/input device
    stdio: UnixStdio,
    /// Poll for readiness event
    session_poll: Poll,
    /// Poll for sending the kernel image
    kernel_poll: Poll,
    /// Number of breaks the bootloader sent
    num_breaks: usize,
    /// The kernel image path
    kimage_path: PathBuf,
    /// Session state
    session_state: SessionState,
    /// Control character pressed
    ctrl_character_pressed: bool,
}

impl SerialSession {
    /// Create new serial (pusher) session.
    pub fn init(serial_dev_path: String, baudrate: u32, kernel_path: PathBuf) -> Result<Self> {
        let args = mio_serial::new(serial_dev_path, baudrate);
        let serial_stream = match SerialStream::open(&args) {
            Ok(device) => device,
            Err(err) => bail!("Error opening serial device: {}", err),
        };
        let stdout_device = match UnixStdio::init() {
            Ok(stdio) => stdio,
            Err(err) => bail!("Failed initializing stdio: {}", err),
        };
        Ok(Self {
            serial_buffer: [0u8; 1024],
            stdin_buffer: [0u8; 1024],
            serial_stream,
            stdio: stdout_device,
            session_poll: Poll::new()?,
            kernel_poll: Poll::new()?,
            num_breaks: 0,
            kimage_path: kernel_path,
            session_state: SessionState::WaitingForLoader,
            ctrl_character_pressed: true,
        })
    }

    /// Start the pusher session
    pub fn start_pusher(&mut self) -> Result<()> {
        let mut events = Events::with_capacity(1024);

        print!("[PUSHER] Polling now...\r\n");
        // register serial port and stdin for polling
        self.session_poll.registry().register(
            &mut self.serial_stream,
            SERIAL_TOKEN,
            Interest::READABLE,
        )?;
        self.session_poll
            .registry()
            .register(&mut self.stdio, STDIN_TOKEN, Interest::READABLE)?;

        loop {
            self.session_poll.poll(&mut events, None)?;
            for event in events.iter() {
                match event.token() {
                    SERIAL_TOKEN => {
                        let bytes_read = self.serial_stream.read(&mut self.serial_buffer)?;
                        self.process_serial(bytes_read);
                    }
                    STDIN_TOKEN if self.session_state == SessionState::EchoMode => {
                        match self.process_stdin()? {
                            Action::Exit => return Ok(()),
                            Action::Proceed => {}
                        }
                    }
                    STDIN_TOKEN => {
                        write!(
                            self.stdio,
                            "[PUSHER] STDIO is off while waiting for loader!\r\n"
                        )?;
                        self.stdio.flush().unwrap();
                        self.clear_stdin_buffer();
                    }
                    Token(_) => eprintln!("Unknown token."),
                }
            }
        }
    }

    /// Process serial output
    fn process_serial(&mut self, bytes_read: usize) {
        for character in &self.serial_buffer[..bytes_read] {
            if *character as char == '\n' {
                self.stdio.write_all("\r\n".as_bytes()).unwrap();
            } else {
                self.stdio.write_all(&[*character]).unwrap();
            }
            self.stdio.flush().unwrap();
        }
        self.num_breaks += self.serial_buffer[..bytes_read]
            .iter()
            .filter(|&byte| *byte == KERNEL_PUSH_SIGNAL)
            .count();
        self.clear_serial_buffer();

        if NUM_BREAKS_TRIGGER_KERNEL_PUSH == self.num_breaks {
            write!(self.stdio, "[PUSHER] Sending kernel!\r\n").unwrap();
            self.stdio.flush().unwrap();
            self.num_breaks = 0;
            self.session_state = SessionState::SendingKernel;
            if let Err(e) = self.send_kernel() {
                write!(self.stdio, "Error sending kernel: {e}").unwrap();
            }
        }
    }

    /// Process input type by the user to the terminal
    fn process_stdin(&mut self) -> Result<Action> {
        // read from stdin and write to serial, only in echo mode
        let bytes_read = self.stdio.read(&mut self.stdin_buffer)?;
        for &character in &self.stdin_buffer[..bytes_read] {
            match character {
                CARTRIDGE_RETURN => {
                    self.serial_stream.write_all(&[b'\n'])?;
                }
                CTRL_CHARACTER => {
                    self.ctrl_character_pressed = true;
                }
                EXIT_CHAR if self.ctrl_character_pressed => {
                    return Ok(Action::Exit);
                }
                _ => {
                    self.serial_stream.write_all(&[character])?;
                }
            }
            self.serial_stream.flush()?;
            // if the control character flag is on, but the current character is not the control
            // character (meaning it wasn't turned on in the current iteration) -
            // we handled whatever was to be handled, turn the flag off.
            if !self.ctrl_character_pressed && character != CTRL_CHARACTER {
                self.ctrl_character_pressed = false;
            }
        }
        self.clear_stdin_buffer();
        Ok(Action::Proceed)
    }
    /// Send kernel image over serial connection
    fn send_kernel(&mut self) -> Result<()> {
        let mut bytes_sent = 0;
        let mut kernel_events = Events::with_capacity(1024);

        // Patch: I am not sure if polling for writable event is necessary, as it isn't always
        // working.
        self.kernel_poll.registry().register(
            &mut self.serial_stream,
            SERIAL_TOKEN,
            Interest::READABLE | Interest::WRITABLE,
        )?;

        let kernel_size = fs::metadata(&self.kimage_path)?.len() as u32;
        write!(self.stdio, "[PUSHER] Kernel size: {kernel_size} bytes\r\n")?;
        self.stdio.flush()?;
        self.serial_stream.flush()?;
        assert!(std::u32::MAX > kernel_size);

        while KERNEL_SIZE_CHUNKS != bytes_sent {
            self.kernel_poll.poll(&mut kernel_events, None)?;
            for kevent in kernel_events.iter() {
                if kevent.token() == SERIAL_TOKEN && kevent.is_writable() {
                    let byte = ((kernel_size >> (8 * bytes_sent)) & 0xff) as u8;
                    self.serial_stream.write_all(&[byte])?;
                    self.serial_stream.flush()?;
                    bytes_sent += 1;
                }
            }
        }

        kernel_events.clear();
        let mut bytes_read = 0;

        // Read the response from the loader
        while 2 != bytes_read {
            self.kernel_poll.poll(&mut kernel_events, None)?;
            for kevent in kernel_events.iter() {
                if kevent.token() == SERIAL_TOKEN && kevent.is_readable() {
                    bytes_read += self
                        .serial_stream
                        .read(&mut self.serial_buffer[bytes_read..])?;
                }
            }
        }

        if self.serial_buffer[..bytes_read] != vec![b'O', b'K'] {
            bail!("didn't receive OK!");
        }

        write!(
            self.stdio,
            "[PUSHER] Got response: \"{}\", sending image now!\r\n",
            String::from_utf8_lossy(&self.serial_buffer[..bytes_read])
        )?;
        self.stdio.flush()?;

        let kernel_image = fs::read(&self.kimage_path)?;
        kernel_events.clear();
        bytes_sent = 0;

        while bytes_sent != kernel_size {
            self.serial_stream
                .write_all(&[kernel_image[bytes_sent as usize]])?;
            self.serial_stream.flush()?;
            bytes_sent += 1;
        }
        write!(self.stdio, "[PUSHER] Done! Booting now\r\n")?;

        self.stdio.flush()?;
        self.clear_serial_buffer();
        self.kernel_poll = Poll::new()?;
        self.session_state = SessionState::EchoMode;
        Ok(())
    }

    /// Clear serial buffer content by re-setting it
    fn clear_serial_buffer(&mut self) {
        self.serial_buffer = [0u8; 1024]
    }

    /// Clear stdin buffer content by re-setting it
    fn clear_stdin_buffer(&mut self) {
        self.stdin_buffer = [0u8; 1024]
    }
}
