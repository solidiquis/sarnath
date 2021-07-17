use crate::errors::Error;
use crate::signals::Signal;
use std::{
    collections::HashMap,
    fs,
    hash,
    io,
    path::Path,
    sync::mpsc::Sender,
    thread::sleep,
    time::{Duration, SystemTime}
};

pub struct FsWatch {
    dirpath: String,
    mod_times: HashMap<String, SystemTime>,
    tx: Sender<Signal>
}

impl FsWatch {
    pub fn new(dirpath: String, tx: Sender<Signal>) -> Result<Self, Error> {
        let mod_times = match fs::metadata(&dirpath) {
            Err(_) => return Err(Error::PathProblem),
            _ => make_mt_map(&dirpath)
        };

        Ok(FsWatch { dirpath, mod_times, tx })
    }

    pub fn poll(&mut self) {
        loop {
            let mt_map = make_mt_map(&self.dirpath);

            if new_entries_or_modified(&self.mod_times, &mt_map) {
                self.tx.send(Signal::FsMod).unwrap();
                self.mod_times = mt_map
            }

            sleep(Duration::from_millis(500))
        } 
    }
}

fn make_mt_map(dirpath: &str) -> HashMap<String, SystemTime> {
    let mut mod_times = HashMap::new();

    walk(Path::new(dirpath), &mut |dir: &fs::DirEntry| {
        let md = dir.metadata().unwrap();

        mod_times.insert(
            dir.file_name().into_string().unwrap(),
            md.modified().unwrap()
        );
    });

    mod_times
}

fn walk(dir: &Path, cb: &mut dyn FnMut(&fs::DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn new_entries_or_modified<T: hash::Hash + Eq, U: PartialEq>(
    hash_a: &HashMap<T, U>,
    hash_b: &HashMap<T, U>
    ) -> bool
{
    if hash_a.len() != hash_b.len() { return true }

    for (k, _) in hash_a {
        if hash_a[k] != hash_b[k] {
            return true
        }
    }

    false
}

