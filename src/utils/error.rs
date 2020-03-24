use std::fmt::*;

#[derive(Debug)]
pub enum Error {
	Internal(String),
	FromGotham(u32),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Internal(string) => write!(f, "Module internal error: {}", string),
			Error::FromGotham(num) => write!(f, "Gotham error code: {}", num),
		}
	}
}

pub type Result<T> = std::result::Result<T, Error>;
