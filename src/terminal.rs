use std::{
    env,
    fmt::Display,
    io::{self, IsTerminal, Write},
};

pub const BLUE: &str = "1;34";
pub const CYAN: &str = "36";
pub const DIM: &str = "2";
pub const MAGENTA: &str = "35";
pub const MATCH: &str = "1;33";

#[derive(Clone, Copy)]
pub struct Theme {
    enabled: bool,
}

impl Theme {
    pub fn stdout() -> Self {
        Self::detect(io::stdout().is_terminal(), Stream::Stdout)
    }

    pub fn stderr() -> Self {
        Self::detect(io::stderr().is_terminal(), Stream::Stderr)
    }

    fn detect(is_terminal: bool, stream: Stream) -> Self {
        if env::var_os("NO_COLOR").is_some() {
            return Self { enabled: false };
        }
        let forced = env::var_os("CLICOLOR_FORCE").is_some_and(|value| value != "0");
        if !forced
            && (env::var_os("TERM").is_some_and(|value| value == "dumb")
                || env::var_os("CLICOLOR").is_some_and(|value| value == "0"))
        {
            return Self { enabled: false };
        }
        let enabled = forced || (is_terminal && enable_terminal_color(stream));
        Self { enabled }
    }

    #[cfg(test)]
    pub const fn colored() -> Self {
        Self { enabled: true }
    }

    pub fn write(&self, out: &mut impl Write, style: &str, value: impl Display) -> io::Result<()> {
        if self.enabled {
            write!(out, "\x1b[{style}m{value}\x1b[0m")
        } else {
            write!(out, "{value}")
        }
    }
}

#[derive(Clone, Copy)]
enum Stream {
    Stdout,
    Stderr,
}

#[cfg(not(windows))]
fn enable_terminal_color(_: Stream) -> bool {
    true
}

#[cfg(windows)]
fn enable_terminal_color(stream: Stream) -> bool {
    use windows_sys::Win32::{
        Foundation::INVALID_HANDLE_VALUE,
        System::Console::{
            ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle, STD_ERROR_HANDLE,
            STD_OUTPUT_HANDLE, SetConsoleMode,
        },
    };

    let kind = match stream {
        Stream::Stdout => STD_OUTPUT_HANDLE,
        Stream::Stderr => STD_ERROR_HANDLE,
    };
    let handle = unsafe { GetStdHandle(kind) };
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return false;
    }
    let mut mode = 0;
    unsafe {
        GetConsoleMode(handle, &mut mode) != 0
            && SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0
    }
}
