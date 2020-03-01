use kv::{Config, Store};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error as TError;

#[derive(Debug, TError)]
pub enum Error {
    #[error("KV error: {0}")]
    KV(#[from] kv::Error),

    #[error("KV error: KV database init failed")]
    KVInitError,

    #[error("Json error: Convert failed: {0}")]
    JSON(#[from] serde_json::Error)
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct FileData {
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct File {
    pub name: String,
    pub data: FileData,
}

pub struct Fdb {
    pub name: String,
    pub path: PathBuf,
    pub config: Config,
}

impl Fdb {
    pub fn new<P>(p: P, n: String) -> Fdb
    where
        P: AsRef<Path>,
    {
        Fdb {
            name: n,
            path: p.as_ref().to_path_buf(),
            config: Config::new(p),
        }
    }

    pub fn exists(&self) -> Result<bool, Error> {
        match Path::new(&self.path).exists() {
            true => Ok(true),
            false => Err(Error::KVInitError),
        }
    }

    pub fn load<P>(p: P, n: String) -> Fdb
    where
        P: AsRef<Path>,
    {
        Fdb {
            name: n,
            path: p.as_ref().to_path_buf(),
            config: Config::new(p),
        }
    }

    pub fn check(&self, n: &String) -> Result<bool, Error> {
        let store = Store::new(self.config.clone()).unwrap();
        let bucket = store.bucket::<String, String>(None)?;
        let result = bucket.get(n).unwrap();
        match result {
            None => Ok(false),
            Some(_v) => Ok(true),
        }
    }

    pub fn add(&self, f: &File) -> Result<(), Error> {
        let store = Store::new(self.config.clone()).unwrap();
        let bucket = store.bucket::<String, String>(None)?;
        bucket.set(f.name.clone(), serde_json::to_string(&f.data)?)?;
        Ok(())
    }

    pub fn get(&self, n: &String) -> Result<File, Error> {
        let store = Store::new(self.config.clone()).unwrap();
        let bucket = store.bucket::<String, String>(None)?;
        let result = bucket.get(n)?;
        let f = File {
            name: n.clone(),
            data: serde_json::from_str(result.unwrap().as_str())?,
        };
        Ok(f)
    }

    pub fn update(&self, n: &String, d: FileData) -> Result<(), Error> {
        let store = Store::new(self.config.clone()).unwrap();
        let bucket = store.bucket::<String, String>(None)?;
        let f = File {
            name: n.clone(),
            data: d,
        };
        bucket.set(f.name.clone(), serde_json::to_string(&f.data)?)?;
        Ok(())
    }

    pub fn remove(&self, n: &String) -> Result<(), Error> {
        let store = Store::new(self.config.clone()).unwrap();
        let bucket = store.bucket::<String, String>(None)?;
        bucket.remove(n.clone())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    fn reset(name: &str) -> String {
        let s = format!("./instance/{}", name);
        let _ = fs::remove_dir_all(&s);
        s
    }

    fn set(name: &str) -> String {
        let s = format!("./instance/{}", name);
        s
    }

    fn init() -> Fdb {
        let path = reset("db");
        let _fdb: Fdb = Fdb::new(path.clone(), String::from("test").clone());
        _fdb
    }

    fn load() -> Fdb {
        let path = set("db");
        let _fdb: Fdb = Fdb::load(path.clone(), String::from("test").clone());
        _fdb
    }

    #[test]
    fn init_success() {
        let _fdb: Fdb = init();
        let store = Store::new(_fdb.config).unwrap();
        let bucket = store.bucket::<String, String>(None).unwrap();

        let file_data = FileData {
            path: String::from("/test"),
        };

        let file: File = File {
            name: String::from("test"),
            data: file_data,
        };
        bucket.set(file.name.clone(), serde_json::to_string(&file.data).unwrap()).unwrap();
        assert_eq!(bucket.get(file.name.clone()).unwrap().unwrap(), serde_json::to_string(&file.data).unwrap());
    }

    #[test]
    fn exist_after_loaded() {
        let _fdb: Fdb = load();
        assert_eq!(_fdb.exists().is_ok(), true);
    }

    #[test]
    fn add_one_record() {
        let _fdb: Fdb = load();

        let file_data = FileData {
            path: String::from("/test1"),
        };

        let file: File = File {
            name: String::from("test1"),
            data: file_data,
        };
        assert_eq!(_fdb.add(&file).is_ok(), true);
        assert_eq!(_fdb.get(&file.name).unwrap(), file);
    }

    #[test]
    fn get_one_existed_record() {
        let _fdb: Fdb = load();

        let file_data = FileData {
            path: String::from("/test2"),
        };

        let file: File = File {
            name: String::from("test2"),
            data: file_data,
        };
        let _res = _fdb.add(&file);
        assert_eq!(_fdb.get(&file.name).unwrap(), file);
    }

    #[test]
    fn check_key_existed() {
        let _fdb: Fdb = load();

        let file_data = FileData {
            path: String::from("/test3"),
        };

        let file: File = File {
            name: String::from("test3"),
            data: file_data,
        };
        let _res = _fdb.add(&file);
        assert_eq!(_fdb.check(&file.name).unwrap(), true);
        assert_eq!(_fdb.check(&String::from("no exist")).unwrap(), false);
    }

    #[test]
    fn update_one_existed_record() {
        let _fdb: Fdb = load();

        let file_data = FileData {
            path: String::from("/test4"),
        };

        let file: File = File {
            name: String::from("test4"),
            data: file_data,
        };

        let new_file_data = FileData {
            path: String::from("/another"),
        };

        let new_file = File {
            name: String::from("test4"),
            data: new_file_data.clone(),
        } ;
        let _res = _fdb.add(&file);
        let _res = _fdb.update(&file.name, new_file_data);
        assert_eq!(_fdb.get(&file.name).unwrap(), new_file);
    }

    #[test]
    fn remove_one_existed_record() {
        let _fdb: Fdb = load();

        let file_data = FileData {
            path: String::from("/test5"),
        };

        let file: File = File {
            name: String::from("test5"),
            data: file_data,
        };
        let _res = _fdb.add(&file);
        let _res = _fdb.remove(&file.name);
        assert_eq!(_fdb.check(&file.name).unwrap(), false);
    }
}
