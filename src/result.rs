use crate::error;
use std::result;

pub type Result<T> = result::Result<T, error::Error>;
