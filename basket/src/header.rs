use std::{borrow::Cow, io::Read};

use crate::{extract, Error};

#[derive(Debug)]
pub struct Header<'a> {
	pub name: Cow<'a, str>,
	pub value: Cow<'a, str>,
}

pub const CONTENT_TYPE: &str = "content-type";
pub const CONTENT_LENGTH: &str = "content-length";
pub const LOCATION: &str = "location";

pub const CONTENT_TYPE_JSON: Header<'static> = Header {
	name: Cow::Borrowed(CONTENT_TYPE),
	value: Cow::Borrowed("application/json"),
};

pub const CONTENT_TYPE_XML: Header<'static> = Header {
	name: Cow::Borrowed(CONTENT_TYPE),
	value: Cow::Borrowed("application/xml"),
};

pub const CONTENT_TYPE_PLAIN: Header<'static> = Header {
	name: Cow::Borrowed(CONTENT_TYPE),
	value: Cow::Borrowed("text/plain"),
};

/// Parses headers and removes the trailing \r\n
///
/// # Errors
/// - If the headers are not valid UTF-8.
/// - If the headers are not in the correct format.
pub fn from_reader<R>(reader: &mut R) -> Result<(Vec<Header<'static>>, Option<usize>), Error>
where
	R: Read,
{
	let mut headers = Vec::new();
	let mut content_length = None;

	// read headers (assume at least one header is present, the content length)
	// if the next bytes are not \r\n, then there are more headers
	loop {
		let mut line = extract::until(reader, b"\r\n")?;

		if line.is_empty() {
			break;
		}

		let colon = line
			.iter()
			.position(|&b| b == b':')
			.ok_or(Error::InvalidFormat)?;

		if line.get(colon + 1) != Some(&b' ') {
			return Err(Error::InvalidFormat);
		}

		let value = line.split_off(colon + 2);
		let mut name = line;

		// remove the colon and space
		name.truncate(name.len() - 2);

		let mut name = String::from_utf8(name)?;
		let value = String::from_utf8(value)?;

		name.make_ascii_lowercase();

		if name == CONTENT_LENGTH {
			content_length = Some(value.parse()?);
		}

		headers.push(Header {
			name: name.into(),
			value: value.into(),
		});
	}

	Ok((headers, content_length))
}

#[allow(clippy::module_name_repetitions)]
pub trait IntoHeader<'a> {
	fn into_header(self) -> Header<'a>;
}

pub trait IntoHeaderValue<'a> {
	fn into_header_value(self) -> Cow<'a, str>;
}

impl<'a> IntoHeader<'a> for Header<'a> {
	fn into_header(self) -> Header<'a> {
		self
	}
}

impl<'a, N, V> IntoHeader<'a> for (N, V)
where
	N: IntoHeaderValue<'a>,
	V: IntoHeaderValue<'a>,
{
	fn into_header(self) -> Header<'a> {
		Header {
			name: self.0.into_header_value(),
			value: self.1.into_header_value(),
		}
	}
}

impl<'a> IntoHeaderValue<'a> for &'a str {
	fn into_header_value(self) -> Cow<'a, str> {
		Cow::Borrowed(self)
	}
}

impl IntoHeaderValue<'static> for String {
	fn into_header_value(self) -> Cow<'static, str> {
		Cow::Owned(self)
	}
}

impl<'a> IntoHeaderValue<'a> for usize {
	fn into_header_value(self) -> Cow<'a, str> {
		Cow::Owned(self.to_string())
	}
}
