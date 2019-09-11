#![allow(dead_code)]

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate chrono;
extern crate nix;

extern crate hs_crawler;

extern crate unicode_segmentation;

extern crate dirs;

extern crate clap;

mod ui;
use ui::theme::*;

use ui::cache;

mod tui;
use tui::keys::Key;

mod util;
use util::*;

use chrono::prelude::*;

use std::thread;

use std::collections::HashMap;

use std::sync::mpsc;

use clap::{Arg, App};

const DEFAULT_SIZE: (isize, isize) = (80, 40);

pub struct AppData {
    pub canteen: HashMap<Date<Local>, Vec<String>>,
    pub timetable: HashMap<Date<Local>, Vec<String>>,
}

pub struct AppState {
    course: String,

    theme: Theme,
    day: Date<Local>,

    data : AppData,

    loading: (usize, usize),

    errors: Vec<String>,

    display_mode: usize,
}

pub enum Message {
    CanteenData(HashMap<Date<Local>, Vec<String>>),
    TimetableData(HashMap<Date<Local>, Vec<String>>),
    Error(String),
    Key(Key),
    Resize(isize, isize),
}

fn main() -> Result<(), String> {

    let matches = App::new("HS APP")
        .version(VERSION)
        .author("Robin W. <robin@shellf.art>")
        .about("Shows timetable and food plan!")
        .arg(Arg::with_name("course")
            .short("c")
            .long("course")
            .takes_value(true)
            .default_value("AI4")
            .help("Sets course to fetch timetable from.")
        ).arg(
            Arg::with_name("simplecolor")
                .short("s")
                .long("simple-color")
                .help("Use only simple Terminal colors.")
                .conflicts_with("json")
        ).arg(Arg::with_name("json")
                .short("j")
                .long("json")
                .help("Dump data as JSON and exit.")
        ).get_matches();

    let course = matches.value_of("course").unwrap().to_uppercase();

    if matches.is_present("simplecolor") {
        use std::env;
        env::set_var("COLORTERM", "");
    }

    if matches.is_present("json") {
        return Ok(ui::json::print_as_json(&course));
    }

    return ui_app(&course);
}

fn ui_app(course : &str) -> Result<(), String> {
    use std::fmt::Write;
    let mut log = String::new();


    tui::termutil::term_setup();

    let (outgoing, incoming) = mpsc::sync_channel::<Message>(256);

    sighandler::set_back_channel(&outgoing);

    tui::termutil::register_for_sigint(sighandler::sigint);
    tui::termutil::register_for_resize(sighandler::sig_resize);

    let mut state = AppState {
        course: course.to_string(),

        theme: select_colorscheme(),
        day: {
            let mut today = chrono::Local::today();

            if chrono::Local::now().hour() > 18 {
                today = today.succ();
            }

            if today.weekday() == chrono::Weekday::Sat {
                today = today.succ();
            }

            if today.weekday() == chrono::Weekday::Sun {
                today = today.succ();
            }

            today
        },

        data : AppData {
            canteen: Default::default(),
            timetable: Default::default(),
        },

        loading: (0, 0),

        errors: vec![],

        display_mode: 0,
    };

    match cache::read_cache(course) {
        Ok(Some(data)) => state.data = data,
        Ok(None) => (),
        Err(e) => writeln!(log, "Error reading cache: {}", e).unwrap(),
    }


    setup_datasources(&state, &outgoing);
    setup_keyboard_datasource(&outgoing);

    let mut size: (isize, isize) = tui::termutil::terminal_size().unwrap_or(DEFAULT_SIZE);

    loop {
        // render;

        if ! state.errors.is_empty() {
            render_errors(size.clone(), &state);
        } else if state.display_mode % 3 == 0 {
            render(size.clone(), &state);
        } else if state.display_mode % 3 == 2 {
            table_render(size.clone(), &state, &state.data.timetable);
        } else {
            table_render(size.clone(), &state, &state.data.canteen);
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
                match key {
                    Key::Char('m') | Key::Char('M') => state.display_mode += 1,

                    Key::Right | Key::Char('l') | Key::Char('L') => state.day = state.day.succ(),

                    Key::Left | Key::Char('h') | Key::Char('H') => state.day = state.day.pred(),

                    Key::Ctrl('L') => size = tui::termutil::terminal_size().unwrap_or(DEFAULT_SIZE),

                    Key::Ctrl(_) | Key::ESC | Key::Char('q') | Key::Char('Q') => break,

                    Key::Enter => {
                        let _deleted = state.errors.pop();
                    },
                    _ => (),
                }
            }
            Message::Error(e) => {
                if log.len() < 8192 {
                    writeln!(log, "Error: {}", e).unwrap();
                }
                handle_error(&mut state, e)
            },

            Message::CanteenData(data) => {
                state.data.canteen.extend(data);
            }
            Message::TimetableData(data) => {
                state.data.timetable.extend(data);
            }

            Message::Resize(w, h) => {
                size = (w, h);
            }
        }
    }

    tui::termutil::term_unsetup();

    match cache::write_cache(&state.data, course) {
        Ok(()) => (),
        Err(e) => writeln!(log, "Error writing cache: {}", e).unwrap(),
    }

    eprintln!("{}", log);

    Ok(())
}

mod sighandler {
    use super::Message;

    use tui::keys::Key;

    use std::sync::Mutex;
    use std::sync::mpsc::SyncSender;

    static mut SIG_CHANNEL: Option<Mutex<SyncSender<Message>>> = None;

    pub fn set_back_channel(sender : &SyncSender<Message>) {
        unsafe {
            SIG_CHANNEL = Some(Mutex::new(sender.clone()));
        }
    }

    pub extern "C" fn sigint(_: i32) {
        println!("Bye!");
        unsafe {
            if let Some(ref mutex) = SIG_CHANNEL {
                let inner = mutex.lock().unwrap();
                let _ = inner.try_send(Message::Key(Key::ESC));
            }
        }
    }
    pub extern "C" fn sig_resize(_: i32) {
        unsafe {
            use tui::termutil::terminal_size;
            if let Some((w, h)) = terminal_size() {
                if let Some(ref mutex) = SIG_CHANNEL {
                    let inner = mutex.lock().unwrap();
                    let _ = inner.try_send(Message::Resize(w, h));
                }
            }
        }
    }
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
            let (k, r, f) = tui::keys::advanced_keys(key_buffer, key_buffer_filled);
            key_buffer = r;
            key_buffer_filled = f;

            outgoing_cp.send(Message::Key(k)).unwrap();
        }
    });
}

fn setup_datasources(state: &AppState, outgoing: &mpsc::SyncSender<Message>) {
    message_adapter(
        hs_crawler::timetable::get_async(hs_crawler::timetable::Query::ThisWeek, &state.course),
        &outgoing,
        |r| match r {
            Ok(content) => Message::TimetableData(content),
            Err(s) => Message::Error(s),
        },
    );

    message_adapter(
        hs_crawler::timetable::get_async(hs_crawler::timetable::Query::NextWeek, &state.course),
        &outgoing,
        |r| match r {
            Ok(content) => Message::TimetableData(content),
            Err(s) => Message::Error(s),
        },
    );

    message_adapter(
        hs_crawler::canteen_plan::get_async(hs_crawler::canteen_plan::Query::ThisWeek),
        &outgoing,
        |r| match r {
            Ok(content) => Message::CanteenData(content),
            Err(s) => Message::Error(s),
        },
    );

    message_adapter(
        hs_crawler::canteen_plan::get_async(hs_crawler::canteen_plan::Query::NextWeek),
        &outgoing,
        |r| match r {
            Ok(content) => Message::CanteenData(content),
            Err(s) => Message::Error(s),
        },
    );
}


const VERSION: &'static str = env!("CARGO_PKG_VERSION");
fn render(size: (isize, isize), state: &AppState) {
    use tui::*;

    let theme = &state.theme;
    let today = &state.day;
    let canteen = &state.data.canteen;
    let timetable = &state.data.timetable;

    let mut table_widget = GridV::new();
    for (i,d) in timetable.get(&today).unwrap_or(&Default::default()).iter().enumerate() {

        let background = if i % 2 == 1 {
            theme.textback1
        } else {
            theme.textback2
        };

        table_widget.push(
            VText::colored(theme.text, d)
                .margin(1,0)
                .centered()
                .with_background(background)
                .margin(1,0)
        );
    }

    let mut canteen_widget = GridV::new();
    for (i,d) in canteen.get(&today).unwrap_or(&Default::default()).iter().enumerate() {

        let background = if i % 2 == 1 {
            theme.textback1
        } else {
            theme.textback2
        };

        canteen_widget.push(
            VText::colored(theme.text, d)
                .margin(1,0)
                .centered()
                .with_background(background)
                .margin(1,0)
        );
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

    let heading = VText::colored(theme.heading, &info_str)
        .margin(2,1)
        .with_background(theme.textback1)
        .margin(1,1);

    let help =
        VText::colored(
            theme.heading,
            "\
    HELP

    q => Quit
    m => Modus
    ▶ => Next
    ◀ => Prev
    ",
        ).margin(4,2);

    let grid_root = GridH::new()
        .add(
            GridV::new().add(heading).add(help).margin(2,1).centered(),
        ).add(
            table_widget.margin(2,1).centered()
        )
        .add(
            canteen_widget.margin(2,1).centered()
        );

    let mut root = grid_root.centered().with_background(theme.background);

    let (w, h) = size;
    root.try_set_size(w, h);
    root.render_to_stdout();
}

fn table_render(
    size: (isize, isize),
    state: &AppState,
    content: &HashMap<Date<Local>, Vec<String>>,
) {
    use tui::*;

    let theme = &state.theme;
    let mut today = state.day.clone();

    let mut grid_root = GridH::new();

    let mut i = 0;
    for _ in 0..7 {
        let info_str = format!(
            "{:10}\n{:02}.{:02}.{}",
            german_weekday(today.weekday()),
            today.day(),
            today.month(),
            today.year()
        );

        let mut table_widget = GridV::new();
        table_widget.push(
            VText::colored(theme.heading, &info_str).centered()
        );

        for d in content.get(&today).unwrap_or(&Default::default()) {
            let bg = if i % 2 == 1 {
                theme.textback1
            } else {
                theme.textback2
            };

            table_widget.push(
                VText::colored(theme.text, d).centered().with_background(bg)
            );

            i += 1;
        }

        if !content.get(&today).is_none() {
            grid_root.push(table_widget);
        }

        today = today.succ();
    }

    let mut root = grid_root.centered().with_background(theme.background);

    let (w, h) = size;
    root.try_set_size(w as isize, h as isize);
    root.render_to_stdout();
}

fn german_weekday(day: chrono::Weekday) -> &'static str {
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

fn render_errors(size: (isize, isize), state: &AppState) {
    use tui::*;
    let theme = &state.theme;

    let mut root = VText::colored(theme.heading, &(state.errors
        .last()
        .map(|s| s as &str)
        .unwrap_or("This is a Bug.").to_string()
        + "\n\nPress Enter to continue.")
    ).boxed(boxtype::DOUBLE_BORDER_BOX, theme.error)
        .with_background(theme.textback1)
        .max_size(40,80)
        .centered()
        .with_background(theme.background);

    let (w, h) = size;

    root.try_set_size(w as isize, h as isize);
    root.render_to_stdout();
}
