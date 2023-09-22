use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("malformed chord: {0}")]
    MalformedChord(String),

    #[error("malformed tab")]
    MalformedTab,

    #[error("unknown error")]
    Unknown,
}
