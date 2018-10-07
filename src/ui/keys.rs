#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Key {
    Char(char),
    Null,
    EOF,
    Interupt,
    Unknown,

    //Arrows
    Up,
    Down,
    Left,
    Right,

    // F Keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    //Ctrl
    ESC,
    Enter,
    Backspace,
    Tab,

    //Block
    Paste,
    Delete,

    Pause,

    Home,
    End,

    PageUp,
    PageDown,

    Alt(char),
    Ctrl(char),
}

pub fn read() -> Key {
    use nix::unistd::read;
    use nix::Error::Sys;
    use nix::errno::Errno::EINTR;

    let mut key = [0u8;5];

    let read_result = read(0,&mut key);
    let ret = match read_result {
        Ok(0) => Key::EOF,
        Ok(1) => read1(&[key[0]]),
        Ok(2) => read2(&[key[0],key[0]]),
        Ok(3) => read3(&[key[0],key[1],key[2]]),
        Ok(4) => read4(&[key[0],key[1],key[2],key[3]]),
        Ok(5) => read5(&[key[0],key[1],key[2],key[3],key[4]]),
        Ok(_) => unreachable!(),
        Err(Sys(EINTR)) => Key::Interupt,
        Err(e) => Key::Unknown,
    };

    // DEBUG
    //eprintln!("{:?} {:?}", key, read_result);
    ret
}

const MAX_KEY_LEN : usize = 16;
pub fn advanced_keys(mut key : [u8;MAX_KEY_LEN], filled : usize) -> (Key, [u8;MAX_KEY_LEN], usize) {
    use nix::unistd::read;
    use nix::Error::Sys;
    use nix::errno::Errno::EINTR;

    debug_assert!(filled < key.len());

    //eprintln!("key {:?} filled {:?}", key, filled);

    if !is_data_on_stdin() {
        match find_possible_key(&key, filled) {
            Some(r) => return r,
            None => (),
        }
    }

    let read_result = read(0,&mut key[filled..MAX_KEY_LEN]);

    match read_result {
        Ok(0) => return (Key::EOF, [0u8;MAX_KEY_LEN], 0),
        Ok(n) => {
            let valid_bytes = filled + n;
            match find_possible_key(&key, valid_bytes) {
                Some(r) => return r,
                None =>  return (Key::Unknown, [0u8;MAX_KEY_LEN], 0),
            }
        }
        Err(Sys(EINTR)) => (Key::Interupt, key, filled),
        Err(e) => return (Key::Unknown, [0u8;MAX_KEY_LEN], 0),
    }
}

fn is_data_on_stdin() -> bool {
    use nix::poll::*;

    let mut fd = [PollFd::new(0, EventFlags::POLLIN)];
    poll(&mut fd, 0);

    match fd[0].revents() {
        Some(EventFlags::POLLIN) => true,
        _ => false,
    }
}

fn find_possible_key(key : &[u8; MAX_KEY_LEN], valid_bytes : usize) -> Option<(Key, [u8;MAX_KEY_LEN], usize)> {
    if valid_bytes == 0 {
        return  None;
    }

    for eaten_bytes in (1..=valid_bytes).rev() {
        let (buf, rest) = key.split_at(eaten_bytes);
        //eprintln!("{:?}  ::  {:?}  ({}/{})", buf, rest, eaten_bytes, valid_bytes);
        match process_single_key(&buf) {
            Key::Unknown => continue,
            key => {
                let mut new_rest = [0u8;MAX_KEY_LEN];
                for (i,r) in rest.iter().enumerate() {
                    new_rest[i] = r.clone();
                }

                return Some((key, new_rest, valid_bytes - eaten_bytes));
            }
        }
    }
    None
}

fn process_single_key(key : &[u8]) -> Key {

    match key.len() {
        0 => Key::EOF,
        1 => read1(&[key[0]]),
        2 => read2(&[key[0],key[0]]),
        3 => read3(&[key[0],key[1],key[2]]),
        4 => read4(&[key[0],key[1],key[2],key[3]]),
        5 => read5(&[key[0],key[1],key[2],key[3],key[4]]),
        _ => Key::Unknown,
    }
}

fn read1(buf : &[u8;1]) -> Key {
    match buf {
        b"\x00" => Key::Null,
        b"\x09" => Key::Tab,
        b"\x1B" => Key::ESC,
        b"\x0A" => Key::Enter,
        b"\x7F" => Key::Backspace,

        buf if buf[0] < b' ' =>
            Key::Ctrl((buf[0] + b'@') as char),

        //
        b => read_mod_or_utf8(b),
    }
}

fn read2(buf : &[u8;2]) -> Key {
    match buf {

        //
        b => read_mod_or_utf8(b),
    }}

fn read3(buf : &[u8;3]) -> Key {
    match buf {
        // Konsole
        b"\x1B\x5B\x48" => Key::Home,
        b"\x1B\x5B\x46" => Key::End,

        b"\x1B\x5B\x41" => Key::Up,
        b"\x1B\x5B\x42" => Key::Down,
        b"\x1B\x5B\x44" => Key::Left,
        b"\x1B\x5B\x43" => Key::Right,

        b"\x1B\x5B\x50" => Key::Pause,

        // Konsole
        b"\x1B\x4F\x50" => Key::F1,
        b"\x1B\x4F\x51" => Key::F2,
        b"\x1B\x4F\x52" => Key::F3,
        b"\x1B\x4F\x53" => Key::F4,

        //
        b => read_mod_or_utf8(b),
    }}

fn read4(buf : &[u8;4]) -> Key {
    match buf {
        // Linux Terminal
        b"\x1B\x5B\x5B\x41" => Key::F1,
        b"\x1B\x5B\x5B\x42" => Key::F2,
        b"\x1B\x5B\x5B\x43" => Key::F3,
        b"\x1B\x5B\x5B\x44" => Key::F4,
        b"\x1B\x5B\x5B\x45" => Key::F5,

        b"\x1B\x5B\x32\x7E" => Key::Paste,
        b"\x1B\x5B\x35\x7E" => Key::PageUp,
        b"\x1B\x5B\x36\x7E" => Key::PageDown,
        b"\x1B\x5B\x33\x7E" => Key::Delete,

        // Linux Terminal
        b"\x1B\x5B\x31\x7E" => Key::Home,
        b"\x1B\x5B\x34\x7E" => Key::End,

        //
        b => read_mod_or_utf8(b),
    }
}

fn read5(buf : &[u8;5]) -> Key {
    match buf {
        b"\x1B\x5B\x31\x35\x7E" => Key::F5,
        b"\x1B\x5B\x31\x37\x7E" => Key::F6,
        b"\x1B\x5B\x31\x38\x7E" => Key::F7,
        b"\x1B\x5B\x31\x39\x7E" => Key::F8,

        b"\x1B\x5B\x32\x30\x7E" => Key::F9,
        b"\x1B\x5B\x32\x31\x7E" => Key::F10,
        b"\x1B\x5B\x32\x33\x7E" => Key::F11,
        b"\x1B\x5B\x32\x34\x7E" => Key::F12,

        //
        b => read_mod_or_utf8(b),
    }
}

fn read_mod_or_utf8(buf : &[u8]) -> Key {
    if buf.len() == 0 {
        return Key::Unknown;
    }

    // Alt-Keys start with '\x1B'
    if buf.len() >= 2 && buf[0] == b'\x1B' && buf[1] != b'[' {
        return match utf8_or_bust(&buf[1..]) {
            Key::Char(c) =>
                Key::Alt(c),
            _ =>
                Key::Unknown,
        };
    }

    utf8_or_bust(buf)
}

fn utf8_or_bust(buf : &[u8]) -> Key {
    use std::str::from_utf8;
    let mut chars = match from_utf8(&buf) {
        Ok(s) => s.chars(),
        Err(e) => return Key::Unknown,
    };

    let (a,b) = (chars.next(), chars.next());

    match (a,b) {
        (Some(cr), None) => Key::Char(cr),
        _ => return Key::Unknown,
    }
}
