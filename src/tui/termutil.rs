


use nix::sys::signal;
use nix::sys::termios;

pub fn term_setup() -> bool {
    println!("\x1B[?25h");
    println!("\x1B[?25l");

    let mut term = match termios::tcgetattr(0) {
        Ok(o) => o,
        Err(_) => return false,
    };

    term.local_flags &= !termios::LocalFlags::ICANON;
    term.local_flags &= !termios::LocalFlags::ECHO;

    let ret = match termios::tcsetattr(0, termios::SetArg::TCSANOW, &term) {
        Ok(_) => true,
        Err(_) => false,
    };

    let _ = clear_buffer();

    register_for_sigint(set_sigint);

    ret
}

// TODO: dedup code
pub fn register_for_sigint(handler: extern "C" fn(c_int)) {
    use nix::sys::signal::*;
    let sigint = signal::SigSet::empty();
    let flags = SaFlags::empty();

    let action = SigAction::new(SigHandler::Handler(handler), flags, sigint);
    unsafe {
        let _ = sigaction(signal::SIGINT, &action);
    }
}

pub fn register_for_resize(handler: extern "C" fn(c_int)) {
    use nix::sys::signal::*;
    let sigint = signal::SigSet::empty();
    let flags = SaFlags::empty();

    let action = SigAction::new(SigHandler::Handler(handler), flags, sigint);

    unsafe {
        let _ = sigaction(signal::SIGWINCH, &action);
    }
}

pub use self::ioctl::terminal_size;
mod ioctl {

    use nix::libc::{self, c_ulong, c_ushort};
    use std::mem;

    #[repr(C)]
    struct termsize {
        height: c_ushort,
        width: c_ushort,
        _xpixel: c_ushort,
        _ypixel: c_ushort,
    }

    #[cfg(any(target_os = "macos", target_os = "freebsd"))]
    const TIOCGWINSZ: c_ulong = 0x40087468;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    const TIOCGWINSZ: c_ulong = 0x00005413;

    pub fn terminal_size() -> Option<(isize, isize)> {
        unsafe {
            let mut size: termsize = mem::zeroed();
            let res = libc::ioctl(0, TIOCGWINSZ, &mut size);
            if res >= 0 {
                Some((size.width as isize, size.height as isize))
            } else {
                None
            }
        }
    }
}

static mut SIGINT: bool = false;

use nix::libc::c_int;
extern "C" fn set_sigint(_: c_int) {
    println!("SIGINT");
    unsafe {
        SIGINT = true;
    }
}

pub fn was_sigint() -> bool {
    unsafe {
        let ret = SIGINT.clone();
        SIGINT = false;
        ret
    }
}

fn clear_buffer() -> Option<()> {
    use nix::poll::*;
    use nix::unistd::read;

    loop {
        let mut fd = [PollFd::new(0, PollFlags::POLLIN)];
        if let Err(_err) = poll(&mut fd, 0) {
            return None;
        }

        if fd[0].revents()? == PollFlags::POLLIN {
            let mut buf = [0u8; 1024];
            let _void = read(1, &mut buf);
        } else {
            return Some(());
        }
    }
}

pub fn term_unsetup() {
    println!("\x1B[?25h");
    let _ = clear_buffer();
}
