use core::fmt;
use std::io;

use url::ParseError;

#[derive(Debug)]
pub enum Error {
	Io(io::Error),
	InvalidUrl(ParseError),
	UnsupportedHttp,
	ExpectedBody,
	Json(serde_json::Error),
	InvalidFormat,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Error::ExpectedBody => write!(f, "expected body"),
			Error::InvalidUrl(e) => write!(f, "invalid url: {e}"),
			Error::UnsupportedHttp => write!(f, "only HTTP/1.1 is supported"),
			Error::Json(e) => write!(f, "json error: {e}"),
			Error::InvalidFormat => write!(f, "invalid format when parsing resposne"),
			Error::Io(e) => write!(f, "io error: {e}"),
		}
	}
}

impl From<serde_json::Error> for Error {
	fn from(value: serde_json::Error) -> Self {
		Error::Json(value)
	}
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Error::Io(value)
	}
}

impl From<ParseError> for Error {
	fn from(value: ParseError) -> Self {
		Error::InvalidUrl(value)
	}
}
