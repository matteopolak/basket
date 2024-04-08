use core::fmt;
use std::{io, num::ParseIntError, str::Utf8Error, string::FromUtf8Error};

use url::ParseError;

#[derive(Debug)]
pub enum Error {
	ExpectedBody,
	Io(io::Error),
	InvalidFormat,
	InvalidInt(ParseIntError),
	InvalidUrl(ParseError),
	InvalidUtf8(Utf8Error),
	#[cfg(feature = "json")]
	Json(serde_json::Error),
	#[cfg(feature = "xml")]
	Xml(quick_xml::DeError),
	UnsupportedHttp,
	UnknownMethod,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Error::ExpectedBody => write!(f, "expected body"),
			Error::Io(e) => write!(f, "io error: {e}"),
			Error::InvalidFormat => write!(f, "invalid format when parsing resposne"),
			Error::InvalidInt(e) => write!(f, "invalid int: {e}"),
			Error::InvalidUrl(e) => write!(f, "invalid url: {e}"),
			Error::InvalidUtf8(e) => write!(f, "invalid utf8: {e}"),
			#[cfg(feature = "json")]
			Error::Json(e) => write!(f, "json error: {e}"),
			#[cfg(feature = "xml")]
			Error::Xml(e) => write!(f, "xml error: {e}"),
			Error::UnsupportedHttp => write!(f, "only HTTP/1.1 is supported"),
			Error::UnknownMethod => write!(f, "unknown method"),
		}
	}
}

#[cfg(feature = "xml")]
impl From<quick_xml::DeError> for Error {
	fn from(value: quick_xml::DeError) -> Self {
		Self::Xml(value)
	}
}

impl From<ParseIntError> for Error {
	fn from(value: ParseIntError) -> Self {
		Self::InvalidInt(value)
	}
}

impl From<Utf8Error> for Error {
	fn from(value: Utf8Error) -> Self {
		Self::InvalidUtf8(value)
	}
}

impl From<FromUtf8Error> for Error {
	fn from(value: FromUtf8Error) -> Self {
		Self::InvalidUtf8(value.utf8_error())
	}
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for Error {
	fn from(value: serde_json::Error) -> Self {
		Self::Json(value)
	}
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Self::Io(value)
	}
}

impl From<ParseError> for Error {
	fn from(value: ParseError) -> Self {
		Self::InvalidUrl(value)
	}
}
