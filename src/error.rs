use kv::Error as DBError;
use std::io;
use thiserror::Error as TError;

#[derive(Debug, TError)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("Database Error: {0}")]
    DB(#[from] DBError),
}
