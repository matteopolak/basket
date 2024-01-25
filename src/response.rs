#[cfg(feature = "json")]
use serde::de::DeserializeOwned;

use crate::Error;

use super::header::Header;

#[derive(Debug)]
pub struct Response {
	headers: Vec<Header>,
	status: u16,
	body: Option<Vec<u8>>,
}

impl Response {
	#[cfg(feature = "json")]
	pub fn json<T: DeserializeOwned>(self) -> Result<T, Error> {
		let Some(body) = self.body else {
			return Err(Error::ExpectedBody);
		};

		Ok(serde_json::from_slice(&body)?)
	}

	#[cfg(feature = "xml")]
	pub fn xml<T: DeserializeOwned>(self) -> Result<T, Error> {
		let Some(body) = self.body else {
			return Err(Error::ExpectedBody);
		};

		let content = std::str::from_utf8(&body)?;

		Ok(quick_xml::de::from_str(content)?)
	}

	pub fn text(self) -> Result<String, Error> {
		let Some(body) = self.body else {
			return Err(Error::ExpectedBody);
		};

		String::from_utf8(body).map_err(|e| e.utf8_error().into())
	}

	pub fn bytes(self) -> Result<Vec<u8>, Error> {
		self.body.ok_or(Error::ExpectedBody)
	}

	pub fn from_bytes(mut bytes: Vec<u8>) -> Result<Self, Error> {
		let mut response = Response {
			headers: vec![],
			status: 0,
			body: None,
		};

		let mut slice = bytes.as_slice();

		slice = extract_http_version(slice)?;
		slice = expect_skip(slice, b" ")?;

		let (mut slice, status) = extract_until(slice, b" ");
		let status = core::str::from_utf8(status)?.parse::<u16>()?;

		response.status = status;

		// skip rest of line
		slice = extract_until(slice, b"\r\n").0;

		// check if headers are next
		while !slice.starts_with(b"\r\n") {
			if slice.is_empty() {
				return Ok(response);
			}

			let (s, name) = extract_until(slice, b": ");
			let (s, value) = extract_until(s, b"\r\n");

			response.headers.push(Header {
				name: String::from_utf8_lossy(name).into_owned(),
				value: String::from_utf8_lossy(value).into_owned(),
			});

			slice = s;
		}

		slice = expect_skip(slice, b"\r\n")?;

		bytes.drain(0..bytes.len() - slice.len());

		response.body = Some(bytes);
		Ok(response)
	}
}

fn expect_skip<'a>(bytes: &'a [u8], seq: &[u8]) -> Result<&'a [u8], Error> {
	if !bytes.starts_with(seq) {
		return Err(Error::InvalidFormat);
	}

	Ok(&bytes[seq.len()..])
}

fn extract_until<'a>(bytes: &'a [u8], seq: &[u8]) -> (&'a [u8], &'a [u8]) {
	let mut i = 0;

	while !bytes[i..].starts_with(seq) {
		i += 1;
	}

	let extracted = &bytes[..i];

	i += seq.len();

	(&bytes[i..], extracted)
}

fn extract_http_version(bytes: &[u8]) -> Result<&[u8], Error> {
	if !bytes.starts_with(b"HTTP/1.1") {
		return Err(Error::UnsupportedHttp);
	}

	Ok(&bytes[b"HTTP/1.1".len()..])
}
