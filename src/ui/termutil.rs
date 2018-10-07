use std::io;
use std::io::{Read, Write};

use nix::sys::termios;
use nix::unistd;
use nix::sys::signal;

pub fn term_setup() -> bool {
    println!("\x1B[?25h");
    println!("\x1B[?25l");

    for _ in 0..255 {
        println!();
    }

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

pub fn register_for_sigint(handler : extern fn(c_int)) {
    use nix::sys::signal::*;
    let mut sigint = signal::SigSet::empty();
    let mut flags = SaFlags::empty();

    let mut action = SigAction::new(
        SigHandler::Handler(handler),
        flags,
        sigint
    );

    unsafe {
        let _ = sigaction(
            signal::SIGINT,
            &action
        );
    }
}

static mut SIGINT : bool = false;

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
        let mut fd = [PollFd::new(0, EventFlags::POLLIN)];
        poll(&mut fd, 0);

        if fd[0].revents()? == EventFlags::POLLIN {
            let mut buf = [0u8; 1024];
            let _void = read(1, &mut buf);
        }else {
            return Some(());
        }
    }

    Some(())
}

pub fn term_unsetup() {
    println!("\x1B[?25h");
    let _ = clear_buffer();
}

pub fn query_terminal_size_and_reset() -> io::Result<(u16, u16)> {
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();

    write!(stdout, "\x1B[999;999H");
    write!(stdout, "\x1B[6n");
    stdout.flush()?;

    let mut r: char = '\0';
    let mut rbuf = [0u8; 1];

    stdin.read_exact(&mut rbuf)?;
    r = rbuf[0] as char;
    if r != '\x1B' {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Expected ansi esc.",
        ));
    }

    stdin.read_exact(&mut rbuf)?;
    r = rbuf[0] as char;
    if r != '[' {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected '['."));
    }

    let mut x = String::new();
    let mut y = String::new();
    for _ in 0..=9 {
        stdin.read_exact(&mut rbuf)?;
        r = rbuf[0] as char;
        if r == ';' {
            break;
        } else {
            y.push(r);
        }
    }

    for _ in 0..=9 {
        stdin.read_exact(&mut rbuf)?;
        r = rbuf[0] as char;
        if r == 'R' {
            break;
        } else {
            x.push(r);
        }
    }

    let w = x
        .parse::<u16>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Expected Numeric"))?;
    let h = y
        .parse::<u16>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Expected Numeric"))?;

    write!(stdout, "\x1B[0;0H");
    stdout.flush()?;

    Ok((w, h))
}
