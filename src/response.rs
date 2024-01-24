use std::io;

use serde::de::DeserializeOwned;

use super::header::Header;

#[derive(Debug)]
pub struct Response {
	headers: Vec<Header>,
	status: u16,
	pub body: Option<Vec<u8>>,
}

impl Response {
	pub fn json<T: DeserializeOwned>(self) -> Result<T, serde_json::Error> {
		self.body
			.as_deref()
			.ok_or_else(|| panic!("no body"))
			.and_then(serde_json::from_slice)
	}

	pub fn from_bytes(mut bytes: Vec<u8>) -> Result<Self, io::Error> {
		let mut response = Response {
			headers: vec![],
			status: 0,
			body: None,
		};

		let mut slice = bytes.as_slice();

		slice = extract_http_version(slice)?;
		slice = expect_skip(slice, b" ")?;

		let (mut slice, status) = extract_until(slice, b" ");
		let status: u16 = std::str::from_utf8(status).unwrap().parse().unwrap();

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

fn expect_skip<'a>(bytes: &'a [u8], seq: &[u8]) -> Result<&'a [u8], io::Error> {
	if !bytes.starts_with(seq) {
		panic!("expected {:?}", seq);
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

fn extract_http_version(bytes: &[u8]) -> Result<&[u8], io::Error> {
	if !bytes.starts_with(b"HTTP/1.1") {
		panic!("unsupported http version");
	}

	Ok(&bytes[b"HTTP/1.1".len()..])
}
