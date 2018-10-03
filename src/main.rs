#![deny(warnings)]
//^ Warnings as Errors.
#![allow(dead_code)]

extern crate reqwest;
extern crate select;
#[macro_use]
extern crate prettytable;



use std::io::Read;
use std::io::BufRead;
use std::io::BufReader;

use std::io;

use std::vec::Vec;
use std::collections::HashMap;

pub const MAX_RESPONSE_SIZE : u64 = 10*1024*1024;
type DirtyError = Box<std::error::Error>;

fn main() -> Result<(), DirtyError> {
    use prettytable::{Table, Row, format};
    let my_course = "AI3"; //TODO: Proper Commandline interface

    println!("HS-OG Timetable for {}", my_course);
    println!("\t ver. 0.0.1 pre-alpha");


    // Get
    let my_timetable = {

        let course_to_link = download_timetable_index()?;

        let ai_timetable_url = course_to_link.get(&my_course.to_lowercase())
            .expect("Your Course not found!");

        println!("\t(Using {} )", ai_timetable_url);

        download_timetable_from_url(&ai_timetable_url)?
    };

    println!("\n\n\n");

    // Out
    {
        let mut table = Table::new();

        let format = format::FormatBuilder::new()
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
            .build();

        table.set_format(format);

        table.set_titles(row![b=>"Mo", "Di", "Mi", "Do", "Fr", "Sa"]);

        for row in my_timetable.into_iter() {
            table.add_row(Row::from(row));
        }

        table.printstd();
    }

    Ok(())
}

fn transpose_vec_vec<T : Default+Clone>(vec : Vec<Vec<T>>) -> Vec<Vec<T>> {

    let w = vec.len();
    let h = vec.iter().map(|row| row.len()).max().unwrap_or(0);

    let mut vec_out : Vec<Vec<T>> = vec![vec![Default::default(); w]; h];

    for (x, row) in vec.into_iter().enumerate() {
        for (y, elm) in row.into_iter().enumerate() {
            vec_out[y][x] = elm;
        }
    }

    vec_out
}

trait Fixable{
    /// Deuglyfies a thing.
    fn ihh_fix(self) -> Self;
}
impl Fixable for String {
    fn ihh_fix(self) -> Self {
        self
            .lines()
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .fold(String::new(), |a,b| a+"\n"+&b)
    }
}

/// Downloads the timetable for a given url. This is blocking.
/// Returns Days as Columns, Hours as Rows.
fn download_timetable_from_url(url : &str) -> Result<Vec<Vec<String>>, DirtyError> {
    use select::document::Document;
    use select::predicate::{Attr, Class, Name, Predicate};

    let res = reqwest::get(url)?;

    if res.status() != 200 {
        return Err(
            io::Error::new(io::ErrorKind::InvalidData,"Didn't get course table.").into()
        );
    }


    let mut html = String::new();
    res.take(MAX_RESPONSE_SIZE).read_to_string(&mut html)?;

    let dom = Document::from(&*html);

    let timetable_node = dom.find(Class("timetable"))
        .next().ok_or_else(||
            io::Error::new(io::ErrorKind::InvalidData, "Expected timetable class in html")
        )?;

    let timetable = timetable_node.find(Attr("scope", "row")).map(|row| {
        row.find((Class("lastcol")).and(Name("td"))).map(|column| {
            column.text().ihh_fix()
        }).collect::<Vec<String>>()
    }).collect::<Vec<Vec<String>>>();

    Ok(timetable)
}

/// Data is stored as (lowercase_course_name : String, url : String).
type LowercaseCourseToUrl = HashMap<String, String>;

pub const TIMETABLE_INDEX : &str = "https://www.hs-offenburg.de/studium/vorlesungsplaene/";
/// Downloads all the links for the timetable of each course.
/// `TIMETABLE_INDEX` is used as source.
/// This call is blocking.
fn download_timetable_index() -> Result<LowercaseCourseToUrl, DirtyError> {

    // Some constants for Parsing.
    const LINK_FILTER_A : &str = "<a href=\"http://www.hs-offenburg.de/index.php?id=6627";
    const LINK_FILTER_B : &str = "<a href=\"https://www.hs-offenburg.de/index.php?id=6627";
    const LINK_START : &str = "<a href=\"";
    const LINK_MIDDLE : &str = "\">";
    const LINK_END : &str = "</a>";


    let res = reqwest::get(TIMETABLE_INDEX)?;
    if res.status() != 200 {
        return Err(
            io::Error::new(io::ErrorKind::InvalidData,"Didn't get course index.").into()
        );
    }

    // we need this to iterate over lines.
    let reader = BufReader::new(res.take(MAX_RESPONSE_SIZE));

    // Does MAGIC #oldschool, don't ask.   // TODO: use select;
    let course_to_url : HashMap<String,String> =  reader
        .lines()
        .flat_map(|line| line)
        .filter(|line|
            line.starts_with(LINK_FILTER_A) || line.starts_with(LINK_FILTER_B)
        )
        .flat_map( |line| {
            let parts = line
                .replace(LINK_START, "")
                .replace(LINK_END, "")
                .split(LINK_MIDDLE)
                .map(|item| item.to_string()) // Borrow Checker Stuff
                .collect::<Vec<String>>();

            match &parts[..] {
                [link, name] => Some((name.to_lowercase(), link.clone())),
                _ => None
            }
        }).collect();

    if course_to_url.is_empty() {
        return Err(
            io::Error::new(io::ErrorKind::InvalidData, "Failed to parse HTML for course index").into()
        );
    }

    Ok(course_to_url)
}
