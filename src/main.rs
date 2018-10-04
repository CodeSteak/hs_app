#![deny(warnings)]
//^ Warnings as Errors.
#![allow(dead_code)]

extern crate reqwest;
extern crate select;
//#[macro_use]
extern crate prettytable;
extern crate chrono;

use chrono::Local;

mod util;
mod data_source;

use prettytable::{Table, Row, format};


fn main() -> Result<(), String> {
    let my_course = "AI3"; //TODO: Proper Commandline interface

    let format = make_table_fmt();


    println!("HS-OG Timetable for {}", my_course);
    println!("\t ver. 0.0.1 pre-alpha");
    println!("\n\n\n");

    // Get
    let my_timetable_future = data_source::timetable::get_async(my_course);
    let canteen_plan_future = data_source::canteen_plan::get_async();

    let today = Local::today();
    let tomorrow = today.succ();

    let canteen_plan = canteen_plan_future().map_err(|e|
        eprintln!("Error getting canteen plan: \n\t{}", e)
    ).unwrap_or(Default::default());

    // Out
    {
        let mut table = Table::new();

        table.set_format(format);

        /*table.set_titles(row![b=>"Mo", "Di", "Mi", "Do", "Fr", "Sa"]);

        for row in canteen_plan.into_iter() {
            table.add_row(Row::from(row));
        }*/

        match canteen_plan.get(&today) {
            Some(data) =>
                table.add_row(Row::from(data)),
            None =>
                table.add_empty_row()
        };

        match canteen_plan.get(&tomorrow) {
            Some(data) =>
                table.add_row(Row::from(data)),
            None =>
                table.add_empty_row()
        };

        table.printstd();
    }


    let my_timetable = my_timetable_future().map_err(|e|
        eprintln!("Error getting timetable: \n\t{}", e)
    ).unwrap_or(Default::default());
    // Out
    {
        let mut table = Table::new();

        table.set_format(format);

        /*table.set_titles(row![b=>"Mo", "Di", "Mi", "Do", "Fr", "Sa"]);

        for row in my_timetable.into_iter() {
            table.add_row(Row::from(row));
        }*/

        match my_timetable.get(&today) {
            Some(data) =>
                table.add_row(Row::from(data)),
            None =>
                table.add_empty_row()
        };

        match my_timetable.get(&tomorrow) {
            Some(data) =>
                table.add_row(Row::from(data)),
            None =>
                table.add_empty_row()
        };

        table.printstd();
    }

    println!("{:#?}", canteen_plan);
    println!("{:#?}", my_timetable);

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