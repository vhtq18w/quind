use kv::{Config, Store};
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::{Path, PathBuf};
use std::{fs, path};
use thiserror::Error as TError;

#[derive(Debug, TError)]
pub enum Error {
    #[error("KV error: {0}")]
    KV(#[from] kv::Error),

    #[error("KV error: KV database init failed")]
    KVInitError,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset(name: &str) -> String {
        let s = format!("./instance/{}", name);
        let _ = fs::remove_dir_all(&s);
        s
    }

    fn set(name: &str) -> String {
        let s = format!("./instance/{}", name);
        s
    }

    fn fdb_init() -> Fdb {
        let path = reset("db");
        let _fdb: Fdb = Fdb::new(path.clone(), String::from("test").clone());
        _fdb
    }

    fn fdb_load() -> Fdb {
        let path = set("db");
        let _fdb: Fdb = Fdb::load(path.clone(), String::from("test").clone());
        _fdb
    }

    #[test]
    fn fdb_init_success() {
        let _fdb: Fdb = fdb_init();
        let store = Store::new(_fdb.config).unwrap();
        let bucket = store.bucket::<String, String>(None).unwrap();

        let data = serde_json::json!({
          "path": "/test"
        });

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
    fn fdb_exist_after_loaded() {
        let _fdb: Fdb = fdb_load();
        assert_eq!(_fdb.exists().is_ok(), true);
    }
}
