use super::*;

use ::util::*;

use std::io;
use std::io::Read;

use std::collections::HashMap;

use select::document::Document;
use select::predicate::*;

use reqwest;
use chrono::{Date, Local};

type CanteenPlan = HashMap<Date<Local>, Vec<String>>;

const URL : &str = "https://www.swfr.de/de/essen-trinken/speiseplaene/mensa-offenburg/";

pub fn get_async() -> impl FnOnce() -> Result<CanteenPlan, String> {
    dirty_err_async(DEFAULT_TIMEOUT_SEC, move || get())
}

pub fn get() -> Result<CanteenPlan, DirtyError> {

    let res = reqwest::get(URL)?;

    if res.status() != 200 {
        return Err(
            io::Error::new(io::ErrorKind::InvalidData,"Didn't get course table.").into()
        );
    }


    let mut html = String::new();
    res.take(MAX_RESPONSE_SIZE).read_to_string(&mut html)?;

    // Strange workaround.
    html = html.replace("<br>", "\n");

    let dom = Document::from(&*html);

    let mut date = last_monday();

    let menu_plan = dom
        .find(Class("tab-content"))
        .flat_map(|maybe_plan| {
            maybe_plan.find(Class("menu-tagesplan")).map(|day_node| {
                day_node.find(Class("menu-info")).map(|menu| {
                    let mut full = menu.text().ihh_fix();
                    let _alergies = menu.find(Class("zusatzsstoffe"))
                        .fold(String::new(), |a,b| {
                            full = full.replace(b.text().trim(), "");
                            a +" "+ &b.text()
                        })
                        .ihh_fix();

                    full
                }).collect::<Vec<String>>()
            })
        }).collect::<Vec<Vec<String>>>();

    let daily_menu_plan : CanteenPlan = menu_plan
        .into_iter()
        .map(|d| {
            let ret = (date.clone(), d);
            date = date.succ();
            ret
        }).collect();


    Ok(daily_menu_plan)
}