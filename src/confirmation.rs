use std::io::{self, IsTerminal, Read};

pub fn read_key() -> io::Result<bool> {
    if !io::stdin().is_terminal() {
        return read_byte();
    }
    read_terminal_key()
}

fn read_byte() -> io::Result<bool> {
    let mut byte = [0];
    match io::stdin().lock().read(&mut byte)? {
        0 => Ok(false),
        _ => Ok(byte[0] == b'y'),
    }
}

#[cfg(unix)]
fn read_terminal_key() -> io::Result<bool> {
    use std::os::fd::AsRawFd;

    struct RestoreMode {
        fd: libc::c_int,
        mode: libc::termios,
    }
    impl Drop for RestoreMode {
        fn drop(&mut self) {
            // Best effort during unwinding or an input error.
            unsafe { libc::tcsetattr(self.fd, libc::TCSANOW, &self.mode) };
        }
    }

    let stdin = io::stdin();
    let fd = stdin.as_raw_fd();
    let mut original = std::mem::MaybeUninit::<libc::termios>::uninit();
    if unsafe { libc::tcgetattr(fd, original.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let original = unsafe { original.assume_init() };
    let guard = RestoreMode { fd, mode: original };
    let mut immediate = original;
    immediate.c_lflag &= !(libc::ICANON | libc::ECHO);
    immediate.c_cc[libc::VMIN] = 1;
    immediate.c_cc[libc::VTIME] = 0;
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &immediate) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let result = read_byte();
    drop(guard);
    result
}

#[cfg(windows)]
fn read_terminal_key() -> io::Result<bool> {
    use windows_sys::Win32::{
        Foundation::INVALID_HANDLE_VALUE,
        System::Console::{
            ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, GetConsoleMode, GetStdHandle, STD_INPUT_HANDLE,
            SetConsoleMode,
        },
    };

    struct RestoreMode {
        handle: windows_sys::Win32::Foundation::HANDLE,
        mode: u32,
    }
    impl Drop for RestoreMode {
        fn drop(&mut self) {
            unsafe { SetConsoleMode(self.handle, self.mode) };
        }
    }

    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let mut original = 0;
    if unsafe { GetConsoleMode(handle, &mut original) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let guard = RestoreMode {
        handle,
        mode: original,
    };
    if unsafe { SetConsoleMode(handle, original & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT)) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let result = read_byte();
    drop(guard);
    result
}
