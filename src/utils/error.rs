use std::fmt::*;

#[derive(Debug)]
pub enum Error {
	Internal(String),
	FromJuno(u32),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Internal(string) => write!(f, "Module internal error: {}", string),
			Error::FromJuno(num) => write!(f, "Juno error code: {}", num),
		}
	}
}

pub type Result<T> = std::result::Result<T, Error>;
