use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, Config};
use std::path::{Path};
use std::sync::mpsc;
use thiserror::Error as TError;

#[derive(Debug, TError)]
pub enum Error {
    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),

    #[error("Sync error: {0}")]
    Sync(#[from] mpsc::RecvError),
}

pub struct Monitor {
    watcher: Box<RecommendedWatcher>,
    rx: mpsc::Receiver<Result<Event, notify::Error>>,
}

impl Monitor {

    pub fn new() -> Monitor {
        let (tx, rx) = mpsc::channel();

        let mut watcher: Result<RecommendedWatcher, notify::Error> = Watcher::new_immediate(move |result| {
            tx.send(result).unwrap();
        });

        Monitor {
            watcher: Box::new(watcher.unwrap()),
            rx,
        }
    }

    pub fn set_precise(&mut self) -> Result<bool, Error> {
        let precise_event = self.watcher.configure(Config::PreciseEvents(true));
        match precise_event {
            Ok(_) => Ok(true),
            Err(e) => Err(Error::Notify(e)),
        }
    }

    pub fn watch<P>(&mut self, d: P) -> Result<(), Error>
    where
        P: AsRef<Path>
    {
        let _watch = self.watcher.watch(d, RecursiveMode::Recursive);
        match _watch {
            Ok(()) => Ok(()),
            Err(error) => Err(Error::Notify(error)),
        }
    }

    pub fn unwatch<P>(&mut self, d: P) -> Result<(), Error>
        where
            P: AsRef<Path>
    {
        let _unwatch = self.watcher.unwatch(d);
        match _unwatch {
            Ok(()) => Ok(()),
            Err(error) => Err(Error::Notify(error)),
        }
    }

    pub fn get(&mut self) -> Result<Result<Event, notify::Error>, Error> {
        match self.rx.recv() {
            Ok(event) => Ok(event),
            Err(e) => Err(Error::Sync(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::fs::File;
    use std::path::Path;

    fn touch(path: &Path) -> () {
        File::create(path).unwrap();
    }

    fn mkdir(path: &Path) -> () {
        fs::create_dir(path).unwrap();
    }

    fn rm(path: &Path) -> () {
        fs::remove_file(path).unwrap();
    }

    fn rmdir(path: &Path) -> () {
        fs::remove_dir_all(path);
    }

    fn monitor_init() -> Monitor {
        Monitor::new()
    }

    #[test]
    fn init() {
        let mut m = monitor_init();
        assert_eq!(m.watch("./").is_ok(), true);
        //touch(&Path::new("./test"));
        //assert_eq!(m.test_out().is_ok(), true);
        //rm(&Path::new("./test"));
    }

    #[test]
    fn add_one_watched_directory() {
        let mut m = monitor_init();
        assert_eq!(m.watch("./").is_ok(), true);
        assert_eq!(m.unwatch("./").is_ok(), true);
        //touch(&Path::new("./test"));
        //assert_eq!(m.test_out().is_ok(), false);
        //rm(&Path::new("./test"));
    }
    // Event: Ok(Event {
    //   kind: Create(Any),
    //   paths: ["D:\\Projects\\quind\\./test"],
    //   attr:tracker: None,
    //   attr:flag: None,
    //   attr:info: None,
    //   attr:source: None
    // })
}