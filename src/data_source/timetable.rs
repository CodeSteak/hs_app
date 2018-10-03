use super::*;

use util::*;

use std::io;
use std::io::Read;
use std::io::BufRead;
use std::io::BufReader;

use std::collections::HashMap;

use select::document::Document;
use select::predicate::*;

use reqwest;

pub fn get(course : &str) -> Result<Vec<Vec<String>>, DirtyError> {
    let index = download_timetable_index()?;

    let course_url = index.get(&course.to_lowercase()).ok_or_else(||
        io::Error::new(io::ErrorKind::InvalidInput, "Course not found.")
    )?;

    download_timetable_from_url(course_url)
}

/// Downloads the timetable for a given url. This is blocking.
/// Returns Days as Columns, Hours as Rows.
fn download_timetable_from_url(url : &str) -> Result<Vec<Vec<String>>, DirtyError> {
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