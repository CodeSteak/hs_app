//#![deny(warnings)]
//^ Warnings as Errors.
#![allow(dead_code)]

extern crate chrono;
extern crate nix;
extern crate reqwest;
extern crate select;
extern crate signal;
extern crate unicode_segmentation;

use chrono::Date;
use chrono::Datelike;
use chrono::Local;

use std::io;

mod data_source;
mod ui;
mod util;

use std::thread;
use std::time::{Duration, Instant};

use nix::sys::signal::SIGINT;
use signal::trap::Trap;

use std::collections::HashMap;

type State = u64;

fn main() -> Result<(), String> {
    use ui::termutil::*;
    use ui::*;

    term_setup();

    let trap = Trap::trap(&[SIGINT]);

    let my_course = "AI3"; //TODO: Proper Commandline interface

    let mut today = chrono::Local::today();

    if today.weekday() == chrono::Weekday::Sat {
        today = today.succ();
    }

    if today.weekday() == chrono::Weekday::Sun {
        today = today.succ();
    }

    // Get
    let my_timetable_rec =
        data_source::timetable::get_async(data_source::timetable::Query::ThisWeek, my_course);
    let my_timetable_next_rec =
        data_source::timetable::get_async(data_source::timetable::Query::NextWeek, my_course);
    let canteen_plan_rec =
        data_source::canteen_plan::get_async(data_source::canteen_plan::Query::NextWeek);
    let canteen_plan_next_rec =
        data_source::canteen_plan::get_async(data_source::canteen_plan::Query::NextWeek);

    let mut canteen_plan: HashMap<Date<Local>, Vec<String>> = Default::default();
    let mut my_timetable: HashMap<Date<Local>, Vec<String>> = Default::default();

    let mut errors = String::new();

    let mut mode = 0;

    loop {
        match my_timetable_rec.try_recv() {
            Ok(Ok(content)) => my_timetable.extend(content),
            Ok(Err(e)) => errors += &format!("Error getting timetable: \t{}", e),
            Err(_) => (),
        }

        match my_timetable_next_rec.try_recv() {
            Ok(Ok(content)) => my_timetable.extend(content),
            Ok(Err(e)) => errors += &format!("Error getting timetable of next week: \t{}", e),
            Err(_) => (),
        }

        match canteen_plan_rec.try_recv() {
            Ok(Ok(content)) => canteen_plan.extend(content),
            Ok(Err(e)) => errors += &format!("Error getting canteen plan: \t{}", e),
            Err(_) => (),
        }

        match canteen_plan_next_rec.try_recv() {
            Ok(Ok(content)) => canteen_plan.extend(content),
            Ok(Err(e)) => errors += &format!("Error getting canteen plan of next week: \t{}", e),
            Err(_) => (),
        }

        if mode % 3 == 0 {
            render(&today, &canteen_plan, &my_timetable);
        } else if mode % 3 == 1 {
            table_render(&today, &canteen_plan);
        } else {
            table_render(&today, &my_timetable);
        }

        if Some(SIGINT) == trap.wait(Instant::now()) {
            break;
        }

        match step_input() {
            Some(b'l') | Some(b'L') => today = today.succ(),
            Some(b'h') | Some(b'H') => today = today.pred(),
            Some(b'n') | Some(b'N') => today = today.succ(),
            Some(b'p') | Some(b'P') => today = today.pred(),
            Some(b'm') | Some(b'M') => {
                mode += 1;
            }
            Some(b'q') | Some(b'Q') => {
                break;
            }
            Some(b'\x1B') => match read_ansi() {
                b'A' | b'C' => today = today.succ(),
                b'B' | b'D' => today = today.pred(),
                _ => (),
            },
            _ => (),
        }
    }

    term_unsetup();
    Ok(())
}

fn step_input() -> Option<u8> {
    use nix::poll::*;
    use std::io::Read;

    let mut fd = [PollFd::new(0, EventFlags::POLLIN)];

    poll(&mut fd, 100);

    if fd[0].revents()? == EventFlags::POLLIN {
        let mut buf = [0u8; 1];
        std::io::stdin().read_exact(&mut buf);

        if buf[0] != 0 {
            return Some(buf[0]);
        }
    }
    None
}

fn read_ansi() -> u8 {
    use std::io::Read;

    loop {
        let mut buf = [0u8; 1];
        std::io::stdin().read_exact(&mut buf);

        if buf[0] == 0 {
            return 0;
        }
        if (buf[0] as char).is_ascii_alphabetic() {
            return buf[0];
        }
    }
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
fn render(
    today: &Date<Local>,
    canteen: &HashMap<Date<Local>, Vec<String>>,
    timetable: &HashMap<Date<Local>, Vec<String>>,
) {
    use ui::termutil::*;
    use ui::*;

    use solarized::*;

    let mut i = 0;
    let mut table_widget = GridV::new();
    for d in timetable.get(&today).unwrap_or(&Default::default()) {
        i += 1;
        table_widget.push(Margin(
            (1, 0),
            Backgound(
                if i % 2 == 1 { BASE2 } else { BASE3 },
                Spacer::new(Margin((2, 0), VText::colored(BASE00, d))),
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
                if i % 2 == 1 { BASE2 } else { BASE3 },
                Center::new(Margin((2, 0), VText::colored(BASE00, d))),
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
    Hochschul-App \nv{}\n\n\
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
        Backgound(BASE2, Margin((2, 1), VText::colored(BASE01, &info_str))),
    );

    let help = Margin(
        (4, 2),
        VText::colored(
            BASE01,
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

    let mut root = Backgound(GREEN, Spacer::new(grid_root));

    let (w, h) = query_terminal_size_and_reset().unwrap_or((100, 100));
    root.try_set_size(w as isize, h as isize - 1);
    root.render_to_stdout();
}

fn table_render(start: &Date<Local>, content: &HashMap<Date<Local>, Vec<String>>) {
    use ui::termutil::*;
    use ui::*;

    use solarized::*;

    let mut today = start.clone();

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

        let mut table_widget = GridV::new().add(Center::new(VText::colored(BASE2, &info_str)));

        for d in content.get(&today).unwrap_or(&Default::default()) {
            i += 1;
            table_widget.push(Backgound(
                if i % 2 == 1 { BASE2 } else { BASE3 },
                Spacer::new(VText::colored(BASE00, d)),
            ));
        }

        grid_root.push(table_widget);
        today = today.succ();
    }

    let mut root = Backgound(GREEN, Spacer::new(grid_root));

    let (w, h) = query_terminal_size_and_reset().unwrap_or((100, 100));
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
