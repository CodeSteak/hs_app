use super::*;

use crate::util::*;

use std::io;
use std::io::Read;

use std::collections::HashMap;

use select::document::Document;
use select::predicate::*;

use chrono::{Date, Local};
use reqwest;

type CanteenPlan = HashMap<Date<Local>, Vec<String>>;

const URL_THIS_WEEK: &str = "https://www.swfr.de/essen-trinken/speiseplaene/mensa-offenburg/";
//const URL_NEXT_WEEK : &str = "https://www.swfr.de/essen-trinken/speiseplaene/mensa-offenburg/?tx_swfrspeiseplan_pi1[weekToShow]=1";

use std::sync::mpsc::Receiver;
pub fn get_async(q: Query) -> Receiver<Result<CanteenPlan, String>> {
    dirty_err_async(move || get(q))
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Query {
    ThisWeek,
    NextWeek,
}

fn get_url_next_week() -> Result<String, DirtyError> {
    let res = reqwest::blocking::get(URL_THIS_WEEK)?;

    if res.status() != 200 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Didn't get course table.").into());
    }

    let mut html = String::new();
    res.take(MAX_RESPONSE_SIZE).read_to_string(&mut html)?;

    let dom = Document::from(&*html);

    let menu_url = dom.find(And(Class("next-week"), Class("text-right")))
        .next().unwrap() // todo
        .attr("href").unwrap()
        .to_owned();


    Ok(format!("https://www.swfr.de{}", menu_url))
}


pub fn get(q: Query) -> Result<CanteenPlan, DirtyError> {
    let res = match q {
        Query::ThisWeek => reqwest::blocking::get(URL_THIS_WEEK)?,
        Query::NextWeek => reqwest::blocking::get(&get_url_next_week()?)?,
    };

    if res.status() != 200 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Didn't get course table.").into());
    }

    let mut html = String::new();
    res.take(MAX_RESPONSE_SIZE).read_to_string(&mut html)?;

    // Strange workaround.
    html = html.replace("<br>", "\n");

    let dom = Document::from(&*html);

    let mut date = last_monday_or_next_monday_on_sundays();
    if q == Query::NextWeek {
        for _ in 0..7 {
            date = date.succ();
        }
    }

    let menu_plan = dom
        .find(Class("tab-content"))
        .flat_map(|maybe_plan| {
            maybe_plan.find(Class("menu-tagesplan")).map(|day_node| {
                day_node
                    .find(Class("menu-info"))
                    .map(|menu| {
                        menu.text()
                            .ihh_fix()
                            .lines()
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .filter(|l| !l.starts_with("enthält Allergene"))
                            .filter(|l| !l.starts_with("Kennzeichnungen"))
                            .fold(String::new(), |a, b| a + "\n" + &b)
                    }).collect::<Vec<String>>()
            })
        }).collect::<Vec<Vec<String>>>();

    let daily_menu_plan: CanteenPlan = menu_plan
        .into_iter()
        .map(|d| {
            let ret = (date.clone(), d);
            date = date.succ();
            ret
        }).collect();

    Ok(daily_menu_plan)
}
