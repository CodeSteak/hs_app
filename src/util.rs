extern crate std;
use chrono::{Date, Datelike, Local, Weekday};
use std::error::Error;

pub type DirtyError = Box<Error>;

pub(crate) fn last_monday() -> Date<Local> {
    let mut now = Local::today();

    for _ in 0..8 {
        if now.weekday() == Weekday::Mon {
            return now;
        }

        now = now.pred();
    }

    panic!("No monday found in this Week?!");
}

pub(crate) trait Fixable {
    /// Deuglyfies a thing.
    fn ihh_fix(&self) -> Self;
}

impl Fixable for String {
    fn ihh_fix(&self) -> Self {
        self.lines()
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .fold(String::new(), |a, b| a + "\n" + &b)
    }
}

pub(crate) trait TransposeAble {
    fn transpose(self) -> Self;
}

impl<T: Default + Clone> TransposeAble for Vec<Vec<T>> {
    fn transpose(self) -> Self {
        let w = self.len();
        let h = self.iter().map(|row| row.len()).max().unwrap_or(0);

        let mut vec_out: Vec<Vec<T>> = vec![vec![Default::default(); w]; h];

        for (x, row) in self.into_iter().enumerate() {
            for (y, elm) in row.into_iter().enumerate() {
                vec_out[y][x] = elm;
            }
        }

        vec_out
    }
}

pub(crate) fn dirty_err_async<F, T>(func: F) -> std::sync::mpsc::Receiver<Result<T, String>>
where
    F: 'static + Send + FnOnce() -> Result<T, DirtyError>,
    T: 'static + Send,
{
    use std::sync::mpsc::channel;
    use std::thread;

    let (sx, rx) = channel();

    thread::spawn(move || {
        let ret: Result<T, String> = func().map_err(|e| e.to_string());

        let _ = sx.send(ret);
    });

    rx
}
