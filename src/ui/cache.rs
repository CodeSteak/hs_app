
use chrono::prelude::*;

use std::collections::HashMap;
use std::fs::File;

use dirs;
use serde_json;

use ::AppData;

use std::num::Wrapping;

#[derive(Serialize, Deserialize, Debug)]
struct AppDataStorage {
    canteen: HashMap<DateTime<Local>, Vec<String>>,
    timetable: HashMap<DateTime<Local>, Vec<String>>,
}

// Worst hash 3v4r.
fn shitty_hash(input : &str) -> u64 {
    let mut initial = Wrapping(17u64);
    for _ in 0..32 {
        for (i,ch) in input.char_indices() {
            let prime_a = Wrapping(179425943u64);
            let prime_b = Wrapping(1300487u64);
            let xor_const = Wrapping(0xFA49_7643_1546_BABAu64);

            initial = (initial * prime_a) + Wrapping(ch as u64);
            initial.0 = initial.0.rotate_left(((i + 5) % 19) as u32);
            initial = (initial * prime_b) + Wrapping(i as u64);
            initial.0 = initial.0.rotate_left(((i + 15) % 17) as u32);
            initial = initial ^ xor_const;
            initial.0 = initial.0.rotate_left(((i + 24) % 32) as u32);
        }
    }
    return initial.0;
}

pub fn read_cache(course : &str) -> Result<Option<AppData>, String> {
    let mut path = dirs::cache_dir().ok_or("Unable to find cache dir.")?;
    path.push(format!("hs_app.{:X}.json", shitty_hash(course)));

    if ! path.exists() {
        return Ok(None);
    }

    let file = File::open(path)
        .map_err(|e| e.to_string())?;

    let data : AppDataStorage = serde_json::from_reader(&file)
        .map_err(|e| e.to_string())?;

    Ok( Some(
        AppData {
            canteen : data.canteen.into_iter().map(|(k,v)| {
                (k.date(), v)
            }).collect(),
            timetable : data.timetable.into_iter().map(|(k,v)| {
                (k.date(), v)
            }).collect()
        }
    ))
}

pub fn write_cache(data : &AppData, course : &str) ->  Result<(), String> {
    let now = Local::now();

    let storage = AppDataStorage {
        canteen: data.canteen.iter().map(|(k,v)|{
            (k.and_hms(12,0,0),v.clone())

            // collect old stuff v
        }).filter(|(k,_)| now.signed_duration_since(k.clone()).num_days() < 30)
            .collect(),
        timetable: data.timetable.iter().map(|(k,v)|{
            (k.and_hms(12,0,0),v.clone())
        }).filter(|(k,_)| now.signed_duration_since(k.clone()).num_days() < 30)
            .collect(),
    };

    let mut path = dirs::cache_dir().ok_or("Unable to find cache dir.")?;
    path.push(format!("hs_app.{:X}.json", shitty_hash(course)));

    let file = File::create(path)
        .map_err(|e| e.to_string())?;

    serde_json::to_writer(file, &storage)
        .map_err(|e| e.to_string())
}
