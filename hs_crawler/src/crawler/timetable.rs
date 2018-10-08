use super::*;

use ::util::*;

use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;

use std::collections::HashMap;

use select::document::Document;
use select::predicate::*;

use chrono::{Date, Local};
use reqwest;

type Timetable = HashMap<Date<Local>, Vec<String>>;

use std::sync::mpsc::Receiver;
pub fn get_async(q: Query, course: &str) -> Receiver<Result<Timetable, String>> {
    let course_copy = course.to_string();

    dirty_err_async(move || get(q, &course_copy))
}

pub enum Query {
    ThisWeek,
    NextWeek,
}

pub fn get(q: Query, course: &str) -> Result<Timetable, DirtyError> {
    let index = download_timetable_index()?;

    let course_url = index
        .get(&course.to_lowercase())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Course not found."))?;

    match q {
        Query::ThisWeek => {
            let mut date = last_monday();
            download_timetable_from_url(&date, course_url)
        }
        Query::NextWeek => {
            let mut date = last_monday();
            for _ in 0..7 {
                date = date.succ();
            }
            download_timetable_from_url(&date, &course_url.replace("week=0", "week=1"))
        }
    }
}

/// Downloads the timetable for a given url. This is blocking.
/// Returns Days as Columns, Hours as Rows.
fn download_timetable_from_url(
    start_date: &Date<Local>,
    url: &str,
) -> Result<Timetable, DirtyError> {
    let mut date = start_date.clone();

    let res = reqwest::get(url)?;

    if res.status() != 200 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Didn't get course table.").into());
    }

    let mut html = String::new();
    res.take(MAX_RESPONSE_SIZE).read_to_string(&mut html)?;

    let dom = Document::from(&*html);

    let timetable_node = dom.find(Class("timetable")).next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Expected timetable class in html",
        )
    })?;

    let timetable: Timetable = timetable_node
        .find(Attr("scope", "row"))
        .map(|row| {
            row.find((Class("lastcol")).and(Name("td")))
                .map(|column| column.text().ihh_fix())
                .collect::<Vec<String>>()
        }).collect::<Vec<Vec<String>>>()
        .transpose()
        .into_iter()
        .map(|d| {
            let ret = (date.clone(), d);
            date = date.succ();
            ret
        }).collect();

    Ok(timetable)
}

/// Data is stored as (lowercase_course_name : String, url : String).
type LowercaseCourseToUrl = HashMap<String, String>;

pub const TIMETABLE_INDEX: &str = "https://www.hs-offenburg.de/studium/vorlesungsplaene/";
/// Downloads all the links for the timetable of each course.
/// `TIMETABLE_INDEX` is used as source.
/// This call is blocking.
fn download_timetable_index() -> Result<LowercaseCourseToUrl, DirtyError> {
    // Some constants for Parsing.
    const LINK_FILTER_A: &str = "<a href=\"http://www.hs-offenburg.de/index.php?id=6627";
    const LINK_FILTER_B: &str = "<a href=\"https://www.hs-offenburg.de/index.php?id=6627";
    const LINK_START: &str = "<a href=\"";
    const LINK_MIDDLE: &str = "\">";
    const LINK_END: &str = "</a>";

    let res = reqwest::get(TIMETABLE_INDEX)?;
    if res.status() != 200 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Didn't get course index.").into());
    }

    // we need this to iterate over lines.
    let reader = BufReader::new(res.take(MAX_RESPONSE_SIZE));

    // Does MAGIC #oldschool, don't ask.   // TODO: use select;
    let course_to_url: HashMap<String, String> = reader
        .lines()
        .flat_map(|line| line)
        .filter(|line| line.starts_with(LINK_FILTER_A) || line.starts_with(LINK_FILTER_B))
        .flat_map(|line| {
            let parts = line
                .replace(LINK_START, "")
                .replace(LINK_END, "")
                .split(LINK_MIDDLE)
                .map(|item| item.to_string()) // Borrow Checker Stuff
                .collect::<Vec<String>>();

            match &parts[..] {
                [link, name] => Some((name.to_lowercase(), link.replace("http://", "https://"))),
                _ => None,
            }
        }).collect();

    if course_to_url.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Failed to parse HTML for course index",
        ).into());
    }

    Ok(course_to_url)
}
