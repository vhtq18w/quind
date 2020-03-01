use crate::fdb::Error as FdbError;
use crate::monitor::Error as MonitorError;
use std::io;
use thiserror::Error as TError;

#[derive(Debug, TError)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("Database error: {0}")]
    DB(#[from] FdbError),

    #[error("Monitor error: {0}")]
    Monitor(#[from] MonitorError),
}
