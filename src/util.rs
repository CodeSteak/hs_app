extern crate std;
use std::error::Error;

pub type DirtyError = Box<Error>;

pub(crate) trait Fixable{
    /// Deuglyfies a thing.
    fn ihh_fix(&self) -> Self;
}

impl Fixable for String {
    fn ihh_fix(&self) -> Self {
        self
            .lines()
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .fold(String::new(), |a,b| a+"\n"+&b)
    }
}

pub(crate) trait TransposeAble{
    fn transpose(self) -> Self;
}

impl<T : Default+Clone> TransposeAble for Vec<Vec<T>> {
    fn transpose(self) -> Self {
        let w = self.len();
        let h = self.iter().map(|row| row.len()).max().unwrap_or(0);

        let mut vec_out : Vec<Vec<T>> = vec![vec![Default::default(); w]; h];

        for (x, row) in self.into_iter().enumerate() {
            for (y, elm) in row.into_iter().enumerate() {
                vec_out[y][x] = elm;
            }
        }

        vec_out
    }
}

pub(crate) fn dirty_err_async<F,T>(timeout_sec : u64, func : F) -> impl FnOnce() -> Result<T,String>
    where
        F : 'static + Send + FnOnce() -> Result<T,DirtyError>,
        T : 'static + Send {

    use std::thread;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    let (sx, rx) = channel();

    thread::spawn(move || {
        let ret : Result<T,String> = func()
            .map_err(|e| e.to_string());

        let _ = sx.send(ret);
    });

    move || {
        let res = rx
            .recv_timeout(Duration::from_secs(timeout_sec))
            .map_err(|e| e.to_string())?;

        res
    }
}

