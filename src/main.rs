#![deny(warnings)]
//^ Warnings as Errors.
#![allow(dead_code)]

extern crate reqwest;
extern crate select;
#[macro_use]
extern crate prettytable;

mod util;
use util::*;

use prettytable::{Table, Row, format};

mod data_source;

fn main() -> Result<(), DirtyError> {
    let my_course = "AI3"; //TODO: Proper Commandline interface

    println!("HS-OG Timetable for {}", my_course);
    println!("\t ver. 0.0.1 pre-alpha");


    // Get
    let my_timetable_fut = dirty_err_async(20, move ||{
        data_source::timetable::get(my_course)
    });

    let mensa_plan_fut = dirty_err_async(20,||
        data_source::canteen_plan::get()
    );

    let my_timetable = match my_timetable_fut() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {}", e);
            Default::default()
        }
    };

    let mensa_plan = match mensa_plan_fut() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {}", e);
            Default::default()
        }
    };


    println!("\n\n\n");

    let format = make_table_fmt();
    // Out
    {
        let mut table = Table::new();

        table.set_format(format);

        table.set_titles(row![b=>"Mo", "Di", "Mi", "Do", "Fr", "Sa"]);

        for row in my_timetable.into_iter() {
            table.add_row(Row::from(row));
        }

        table.printstd();
    }

    // Out
    {
        let mut table = Table::new();

        table.set_format(format);

        table.set_titles(row![b=>"Mo", "Di", "Mi", "Do", "Fr", "Sa"]);

        for row in mensa_plan.into_iter() {
            table.add_row(Row::from(row));
        }

        table.printstd();
    }

    Ok(())
}

fn make_table_fmt() -> format::TableFormat {
    format::FormatBuilder::new()
        .column_separator('│')
        .borders('│')
        .separator(format::LinePosition::Top,
                   format::LineSeparator::new('─', '┬', '┌', '┐'))
        .separator(format::LinePosition::Title,
                   format::LineSeparator::new('─', '┼', '├', '┤'))
        .separator(format::LinePosition::Intern,
                   format::LineSeparator::new('─', '┼', '├', '┤'))
        .separator(format::LinePosition::Bottom,
                   format::LineSeparator::new('─', '┴', '└', '┘'))
        .padding(1, 1)
        .build()
}