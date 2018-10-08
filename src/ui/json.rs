
use hs_crawler;
use serde_json;

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct JsonState {
    timetable: HashMap<String, String>,
    canteen: HashMap<String, String>,
}

pub fn print_as_json() {

    let state = JsonState {
        timetable: hs_crawler::timetable::get(hs_crawler::timetable::Query::ThisWeek, "AI3")
            .unwrap_or(Default::default())
            .into_iter()
            .map(|(k, v)| (
                k.to_string(),
                v.into_iter().fold(String::new(), |a,b|a+&b)))
            .collect(),
        canteen: hs_crawler::canteen_plan::get(hs_crawler::canteen_plan::Query::ThisWeek)
            .unwrap_or(Default::default())
            .into_iter()
            .map(|(k, v)| (
                k.to_string(),
                v.into_iter().fold(String::new(), |a,b|a+&b)))
            .collect(),
    };

    let out = serde_json::to_string_pretty(&state).expect("Could not print JSON, somehow.");

    println!("{}", out);

}