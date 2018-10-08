#![deny(warnings)]
//^ Warnings as Errors.
#![allow(dead_code)]

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate chrono;
extern crate nix;
extern crate reqwest;
extern crate select;
extern crate unicode_segmentation;

use chrono::Date;
use chrono::Datelike;
use chrono::Local;

mod data_source;
mod ui;
mod util;
use util::*;

use std::thread;

use std::collections::HashMap;

use std::sync::mpsc;

type State = u64;

const DEFAULT_SIZE: (isize, isize) = (80, 40);

use ui::Color;
struct Theme {
    background: Color,

    textback1: Color,
    textback2: Color,

    text: Color,
    heading: Color,
}

struct AppState {
    course: String,

    theme: Theme,
    day: Date<Local>,

    canteen: HashMap<Date<Local>, Vec<String>>,
    timetable: HashMap<Date<Local>, Vec<String>>,

    loading: (usize, usize),

    errors: Vec<String>,

    display_mode: usize,
}

enum Message {
    CanteenData(HashMap<Date<Local>, Vec<String>>),
    TimetableData(HashMap<Date<Local>, Vec<String>>),
    Error(String),
    Key(ui::keys::Key),
    Resize(isize, isize),
}

use std::sync::Mutex;
static mut SIG_CHANNEL: Option<Mutex<mpsc::SyncSender<Message>>> = None;
extern "C" fn sigint(_: i32) {
    println!("Bye!");
    unsafe {
        if let Some(ref mutex) = SIG_CHANNEL {
            let inner = mutex.lock().unwrap();
            let _ = inner.try_send(Message::Key(ui::keys::Key::ESC));
        }
    }
}
extern "C" fn sig_resize(_: i32) {
    unsafe {
        use ui::termutil::terminal_size;
        if let Some((w, h)) = terminal_size() {
            if let Some(ref mutex) = SIG_CHANNEL {
                let inner = mutex.lock().unwrap();
                let _ = inner.try_send(Message::Resize(w, h));
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonState {
    timetable: HashMap<String, Vec<String>>,
    canteen: HashMap<String, Vec<String>>,
}

fn mk_json() {
    let state = JsonState {
        timetable: data_source::timetable::get(data_source::timetable::Query::ThisWeek, "AI3")
            .unwrap_or(Default::default())
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
        canteen: data_source::canteen_plan::get(data_source::canteen_plan::Query::ThisWeek)
            .unwrap_or(Default::default())
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    };

    let out = serde_json::to_string_pretty(&state).expect("Could not print JSON, somehow.");

    println!("{}", out);
}

fn main() -> Result<(), String> {
    //if let Some("") = std::env::args.iter().next() {
    //}
    //
    //

    //mk_json();
    //return Ok(());

    ui::termutil::term_setup();

    let (outgoing, incoming) = mpsc::sync_channel::<Message>(256);
    unsafe {
        SIG_CHANNEL = Some(Mutex::new(outgoing.clone()));
    }
    ui::termutil::register_for_sigint(sigint);
    ui::termutil::register_for_resize(sig_resize);

    let mut state = AppState {
        course: "AI3".to_string(),

        theme: select_colorscheme(),
        day: {
            let mut today = chrono::Local::today();

            if today.weekday() == chrono::Weekday::Sat {
                today = today.succ();
            }

            if today.weekday() == chrono::Weekday::Sun {
                today = today.succ();
            }

            today
        },

        canteen: Default::default(),
        timetable: Default::default(),

        loading: (0, 0),

        errors: vec![],

        display_mode: 0,
    };

    setup_datasources(&state, &outgoing);
    setup_keyboard_datasource(&outgoing);

    let mut size: (isize, isize) = ui::termutil::terminal_size().unwrap_or(DEFAULT_SIZE);

    loop {
        // render;
        if state.display_mode % 3 == 0 {
            render(size.clone(), &state);
        } else if state.display_mode % 3 == 2 {
            table_render(size.clone(), &state, &state.timetable);
        } else {
            table_render(size.clone(), &state, &state.canteen);
        };

        // process

        let msg = match incoming.recv() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error :  {}", e.to_string());
                break;
            }
        };

        match msg {
            Message::Key(key) => {
                use ui::keys::Key;

                match key {
                    Key::Char('m') | Key::Char('M') => state.display_mode += 1,

                    Key::Right | Key::Char('l') | Key::Char('L') => state.day = state.day.succ(),

                    Key::Left | Key::Char('h') | Key::Char('H') => state.day = state.day.pred(),

                    Key::Ctrl('L') => size = ui::termutil::terminal_size().unwrap_or(DEFAULT_SIZE),

                    Key::Ctrl(_) | Key::ESC | Key::Char('q') | Key::Char('Q') => break,
                    _ => (),
                }
            }
            Message::Error(e) => handle_error(&mut state, e),

            Message::CanteenData(data) => {
                state.canteen.extend(data);
            }
            Message::TimetableData(data) => {
                state.timetable.extend(data);
            }

            Message::Resize(w, h) => {
                size = (w, h);
            }
        }
    }
    ui::termutil::term_unsetup();
    Ok(())
}

fn handle_error(state: &mut AppState, err: String) {
    state.errors.push(err);
}

fn setup_keyboard_datasource(outgoing: &mpsc::SyncSender<Message>) {
    let outgoing_cp = outgoing.clone();

    thread::spawn(move || {
        let mut key_buffer = [0u8; 16];
        let mut key_buffer_filled = 0usize;

        loop {
            let (k, r, f) = ui::keys::advanced_keys(key_buffer, key_buffer_filled);
            key_buffer = r;
            key_buffer_filled = f;

            outgoing_cp.send(Message::Key(k)).unwrap();
        }
    });
}

fn setup_datasources(state: &AppState, outgoing: &mpsc::SyncSender<Message>) {
    message_adapter(
        data_source::timetable::get_async(data_source::timetable::Query::ThisWeek, &state.course),
        &outgoing,
        |r| match r {
            Ok(content) => Message::TimetableData(content),
            Err(s) => Message::Error(s),
        },
    );

    message_adapter(
        data_source::timetable::get_async(data_source::timetable::Query::NextWeek, &state.course),
        &outgoing,
        |r| match r {
            Ok(content) => Message::TimetableData(content),
            Err(s) => Message::Error(s),
        },
    );

    message_adapter(
        data_source::canteen_plan::get_async(data_source::canteen_plan::Query::ThisWeek),
        &outgoing,
        |r| match r {
            Ok(content) => Message::CanteenData(content),
            Err(s) => Message::Error(s),
        },
    );

    message_adapter(
        data_source::canteen_plan::get_async(data_source::canteen_plan::Query::NextWeek),
        &outgoing,
        |r| match r {
            Ok(content) => Message::CanteenData(content),
            Err(s) => Message::Error(s),
        },
    );
}

fn show_error(_theme: &Theme, _err: &String) {
    //use ui::termutil::*;
    //use ui::*;

    unimplemented!();
}

fn select_colorscheme() -> Theme {
    let truecolor = std::env::var("COLORTERM")
        .map(|s| s.to_lowercase().contains("truecolor"))
        .unwrap_or(false);

    if truecolor {
        Theme {
            background: solarized::CYAN,

            textback1: solarized::BASE3,
            textback2: solarized::BASE2,

            text: solarized::BASE00,
            heading: solarized::BASE01,
        }
    } else {
        Theme {
            background: Color::Cyan,

            textback1: Color::Blue,
            textback2: Color::Magenta,

            text: Color::White,
            heading: Color::White,
        }
    }
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
fn render(size: (isize, isize), state: &AppState) {
    use ui::*;

    let theme = &state.theme;
    let today = &state.day;
    let canteen = &state.canteen;
    let timetable = &state.timetable;

    let mut i = 0;
    let mut table_widget = GridV::new();
    for d in timetable.get(&today).unwrap_or(&Default::default()) {
        i += 1;
        table_widget.push(Margin(
            (1, 0),
            Backgound(
                if i % 2 == 1 {
                    theme.textback1
                } else {
                    theme.textback2
                },
                Spacer::new(Margin((2, 0), VText::colored(theme.text, d))),
            ),
        ));
    }

    let mut canteen_widget = GridV::new();
    let mut i = 0;
    for d in canteen.get(&today).unwrap_or(&Default::default()) {
        i += 1;
        canteen_widget.push(Margin(
            (1, 0),
            Backgound(
                if i % 2 == 1 {
                    theme.textback1
                } else {
                    theme.textback2
                },
                Center::new(Margin((2, 0), VText::colored(theme.text, d))),
            ),
        ));
    }

    let loading = if canteen.len() == 0 || timetable.len() == 0 {
        "\n\nLädt..."
    } else {
        ""
    };

    let info_str = format!(
        "\
    Hochschul-App \n\tv{}\n\n\
    {:10} {:02}.{:02}.{}{}
    ",
        VERSION,
        german_weekday(today.weekday()),
        today.day(),
        today.month(),
        today.year(),
        loading
    );

    let heading = VBox(
        NONE_BOX,
        Color::None,
        Backgound(
            theme.textback1,
            Margin((2, 1), VText::colored(theme.heading, &info_str)),
        ),
    );

    let help = Margin(
        (4, 2),
        VText::colored(
            theme.heading,
            "\
    HELP

    q => Quit
    m => Modus
    ▶ => Next
    ◀ => Prev
    ",
        ),
    );

    let grid_root = GridH::new()
        .add(Center::new(Margin(
            (2, 1),
            GridV::new().add(heading).add(help),
        ))).add(Center::new(Margin((2, 1), table_widget)))
        .add(Center::new(Margin((2, 1), canteen_widget)));

    let mut root = Backgound(theme.background, Center::new(grid_root));

    let (w, h) = size;
    root.try_set_size(w, h);
    root.render_to_stdout();
}

fn table_render(
    size: (isize, isize),
    state: &AppState,
    content: &HashMap<Date<Local>, Vec<String>>,
) {
    use ui::*;

    let theme = &state.theme;
    let mut today = state.day.clone();

    let mut i = 0;
    let mut grid_root = GridH::new();

    for _ in 0..7 {
        let info_str = format!(
            "{:10}\n{:02}.{:02}.{}",
            german_weekday(today.weekday()),
            today.day(),
            today.month(),
            today.year()
        );

        let mut table_widget =
            GridV::new().add(Center::new(VText::colored(theme.heading, &info_str)));

        for d in content.get(&today).unwrap_or(&Default::default()) {
            i += 1;
            table_widget.push(Backgound(
                if i % 2 == 1 {
                    theme.textback1
                } else {
                    theme.textback2
                },
                Spacer::new(VText::colored(theme.text, d)),
            ));
        }

        if !content.get(&today).is_none() {
            grid_root.push(table_widget);
        }

        today = today.succ();
    }

    let mut root = Backgound(theme.background, Center::new(grid_root));

    let (w, h) = size; //query_terminal_size_and_reset().unwrap_or((100, 100));
    root.try_set_size(w as isize, h as isize - 1);
    root.render_to_stdout();
}

fn german_weekday(day: chrono::Weekday) -> &'static str {
    use chrono::Weekday;
    match day {
        Weekday::Mon => "Montag",
        Weekday::Tue => "Dienstag",
        Weekday::Wed => "Mittwoch",
        Weekday::Thu => "Donnerstag",
        Weekday::Fri => "Freitag",
        Weekday::Sat => "Samstag",
        Weekday::Sun => "Sonntag",
    }
}

mod solarized {
    use ui::vterm::Color;

    pub const BASE3: Color = Color::Custom(0xfd, 0xf6, 0xe3);
    pub const BASE2: Color = Color::Custom(0xee, 0xe8, 0xd5);

    pub const BASE00: Color = Color::Custom(0x65, 0x7b, 0x83);
    pub const BASE01: Color = Color::Custom(0x58, 0x6e, 0x75);

    pub const YELLOW: Color = Color::Custom(0xb5, 0x89, 0x00);
    pub const ORANGE: Color = Color::Custom(0xcb, 0x4b, 0x16);
    pub const RED: Color = Color::Custom(0xdc, 0x32, 0x2f);
    pub const MAGENTA: Color = Color::Custom(0xd3, 0x36, 0x82);
    pub const VIOLET: Color = Color::Custom(0x6c, 0x71, 0xc4);
    pub const BLUE: Color = Color::Custom(0x26, 0x8b, 0xd2);
    pub const CYAN: Color = Color::Custom(0x2a, 0xa1, 0x98);
    pub const GREEN: Color = Color::Custom(0x85, 0x99, 0x00);

}
