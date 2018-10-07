//#![deny(warnings)]
//^ Warnings as Errors.
#![allow(dead_code)]

extern crate chrono;
extern crate nix;
extern crate reqwest;
extern crate select;
extern crate unicode_segmentation;

use chrono::Date;
use chrono::Datelike;
use chrono::Local;

use std::io;

mod data_source;
mod ui;
mod util;

use util::*;

use std::thread;
use std::time::{Duration, Instant};

use nix::sys::signal::SIGINT;

use std::collections::HashMap;

use std::sync::mpsc;

type State = u64;

use ui::Color;
struct Theme {
    background: Color,

    textback1: Color,
    textback2: Color,

    text: Color,
    heading: Color,
}

struct AppState {
    course : String,

    theme : Theme,
    day : Date<Local>,

    canteen : HashMap<Date<Local>, Vec<String>>,
    timetable : HashMap<Date<Local>, Vec<String>>,

    loading : (usize, usize),

    errors : Vec<String>,

    display_mode : usize,
}

enum Message {
    CanteenData(HashMap<Date<Local>, Vec<String>>),
    TimetableData(HashMap<Date<Local>, Vec<String>>),
    Error(String),
    Key(ui::keys::Key),
    Resize(usize, usize),
}

use std::sync::Mutex;
static mut SIG_CHANNEL : Option<Mutex<mpsc::Sender<Message>>> = None;
extern "C" fn sigint(_ : i32) {
    unsafe {
        if let Some(ref mutex) = SIG_CHANNEL {
            let inner = mutex.lock().unwrap();
            let _ = inner.send(Message::Key(ui::keys::Key::ESC));
        }
    }
}

fn main() -> Result<(), String> {
    use ui::termutil::*;
    use ui::*;

    term_setup();

    let (outgoing, incoming) = mpsc::channel::<Message>();
    unsafe {
        SIG_CHANNEL = Some(Mutex::new(outgoing.clone()));
    }
    register_for_sigint(sigint);

    let mut state = AppState {
        course : "AI3".to_string(),

        theme :  select_colorscheme(),
        day : {
            let mut today = chrono::Local::today();

            if today.weekday() == chrono::Weekday::Sat {
                today = today.succ();
            }

            if today.weekday() == chrono::Weekday::Sun {
                today = today.succ();
            }

            today
        },

        canteen : Default::default(),
        timetable : Default::default(),

        loading : (0,0),

        errors : vec![],

        display_mode : 0,
    };

    setup_datasources( &state, &outgoing);
    setup_keyboard_datasource(&outgoing);

    let mut size : (isize, isize) = (80,40);

    loop {
        // render;

        println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n"); //TODO: Remove me, after impl. resize;

        if state.display_mode % 3 == 0 {
            render(size.clone(), &state);
        } else if state.display_mode % 3 == 2 {
            table_render(size.clone(), &state, &state.canteen);
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
            Message::Key(key) => if handle_key(&mut state, key) {
                break;
            },
            Message::Error(e) => handle_error(&mut state, e),

            Message::CanteenData(data) => {
                state.canteen.extend(data);
            },
            Message::TimetableData(data) => {
                state.timetable.extend(data);

            },

            Message::Resize(w,h) => {
                unimplemented!();
            }
        }

    }
    
    println!("Bye!");
    term_unsetup();
    Ok(())
}


fn handle_key(state : &mut AppState, key : ui::keys::Key) -> bool {
    use ui::keys::Key;

    match key {
        Key::Char('m') | Key::Char('M') =>
            state.display_mode += 1,

        Key::Right | Key::Char('l') | Key::Char('L') =>
            state.day = state.day.succ(),

        Key::Left | Key::Char('h') | Key::Char('H') =>
            state.day = state.day.pred(),

        Key::Ctrl(_) | Key::ESC | Key::Char('q') | Key::Char('Q') =>
            return true,

        Key::Interupt => {
            eprintln!("Interupt!"); //TODO REMOVE ME!
            if ui::termutil::was_sigint() {
                return true;
            }
        },
        _ => (),
    }

    false
}

fn handle_error(state : &mut AppState, err : String) {
    state.errors.push(err);
}

fn setup_keyboard_datasource(outgoing : &mpsc::Sender<Message>) {
    let outgoing_cp = outgoing.clone();

    thread::spawn(move || {

        let mut key_buffer = [0u8; 16];
        let mut key_buffer_filled = 0usize;

        loop {
            let (k, r, f) = ui::keys::advanced_keys(key_buffer, key_buffer_filled);
            key_buffer = r;
            key_buffer_filled = f;

            outgoing_cp.send(Message::Key(k) ).unwrap();
        }
    });
}

fn setup_datasources(state : &AppState, outgoing : &mpsc::Sender<Message>) {
    message_adapter(
        data_source::timetable::get_async(data_source::timetable::Query::ThisWeek, &state.course),
        &outgoing,
        |r| {
            match r {
                Ok(content) => Message::TimetableData(content),
                Err(s) => Message::Error(s),
            }
        }
    );

    message_adapter(
        data_source::timetable::get_async(data_source::timetable::Query::NextWeek, &state.course),
        &outgoing,
        |r| {
            match r {
                Ok(content) => Message::TimetableData(content),
                Err(s) => Message::Error(s),
            }
        }
    );

    message_adapter(
        data_source::canteen_plan::get_async(data_source::canteen_plan::Query::ThisWeek),
        &outgoing,
        |r| {
            match r {
                Ok(content) => Message::CanteenData(content),
                Err(s) => Message::Error(s),
            }
        }
    );

    message_adapter(
        data_source::canteen_plan::get_async(data_source::canteen_plan::Query::NextWeek),
        &outgoing,
        |r| {
            match r {
                Ok(content) => Message::CanteenData(content),
                Err(s) => Message::Error(s),
            }
        }
    );
}



























fn show_error(theme: &Theme, err: &String) {
    use std::io::Read;
    use ui::termutil::*;
    use ui::*;

    unimplemented!();

    let error = GridV::new()
        .add(Center::new(Backgound(
            theme.textback1,
            VBox(
                DOUBLE_BORDER_BOX,
                Color::Red,
                VText::colored(theme.heading, err),
            ),
        ))).add(Center::new(VText::colored(
            theme.heading,
            "Any Key to continue.",
        )));

    let mut root = Backgound(theme.background, error);

    //let (w, h) = query_terminal_size_and_reset().unwrap_or((100, 100));
   // root.try_set_size(w as isize, h as isize - 1);
    root.render_to_stdout();
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
fn render(
    size : (isize, isize),
    state : &AppState,
) {
    use ui::termutil::*;
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
    root.try_set_size(w, h - 1);
    root.render_to_stdout();
}

fn table_render(size : (isize, isize), state : &AppState, content: &HashMap<Date<Local>, Vec<String>>) {
    use ui::termutil::*;
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

    let (w, h) = size;//query_terminal_size_and_reset().unwrap_or((100, 100));
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

mod test {

    pub(crate) fn test_render(_state: isize) {
        use ui::termutil::*;
        use ui::*;

        let (w, h) = query_terminal_size_and_reset().unwrap_or((100, 100));

        let mut root = GridV::new()
            .add(
                GridH::new()
                    .add(VBox(SIMPLE_BOX, Color::BrightYellow, VText::simple("1")))
                    .add(VBox(SIMPLE_BOX, Color::BrightYellow, VText::simple("2")))
                    .add(VBox(SIMPLE_BOX, Color::BrightYellow, VText::simple("3"))),
            ).add(
                GridH::new()
                    .add(VBox(SIMPLE_BOX, Color::BrightYellow, VText::simple("4")))
                    .add(VBox(SIMPLE_BOX, Color::BrightYellow, VText::simple("5/6"))),
            ).add(GridH::new().add(VBox(
                SIMPLE_BOX,
                Color::BrightYellow,
                VText::simple("Full"),
            )));

        root.try_set_size(w as isize, h as isize);
        root.render_to_stdout();
    }

    pub(crate) fn test_render2(state: &isize) {
        use ui::termutil::*;
        use ui::*;

        let (w, h) = query_terminal_size_and_reset().unwrap_or((20, 20));

        let mut root = GridH::new()
            .add(VBox(
                DOUBLE_BORDER_BOX,
                Color::BrightYellow,
                Spacer::new(VText::simple("Hello World")),
            )).add(VBox(
                SIMPLE_BOX,
                Color::BrightYellow,
                Spacer::new(VText::simple(
                    "Hello World. And also: 'Hello Humanity'. And Stuff... This is Filler Text",
                )),
            )).add(VBox(
                BORDER_BOX,
                Color::BrightYellow,
                Margin(
                    (4, 2),
                    Backgound(
                        Color::BrightBlue,
                        Spacer::new(VText::simple(
                            "\t1\t2\t3\t4\t5\t6\t7\t8\t9\t\n\ntab stops are working!",
                        )),
                    ),
                ),
            )).add(VText::simple(&format!(
                "Tic: {:.2}",
                *state as f32 / 1000f32
            )));

        root.try_set_size(w as isize, h as isize);
        root.render_to_stdout();
    }
}
