pub mod into;
pub use into::IntoResponse;

use std::io;
use std::io::{Read, Write};

#[cfg(any(feature = "json", feature = "xml"))]
use serde::{de::DeserializeOwned, Serialize};

use crate::header;
use crate::Error;
use crate::{extract, IntoHeader};

use super::header::Header;

#[derive(Debug)]
pub struct Response<'h> {
	headers: Vec<Header<'h>>,
	status: u16,
	body: Option<Vec<u8>>,
}

impl<'h> Response<'h> {
	pub fn builder() -> ResponseBuilder<'h> {
		ResponseBuilder::new()
	}

	pub fn status(&self) -> u16 {
		self.status
	}

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

	pub fn from_reader<R>(reader: &mut R) -> Result<Self, Error>
	where
		R: Read,
	{
		let mut response = Self {
			headers: vec![],
			status: 0,
			body: None,
		};

		extract::http_version(reader)?;
		extract::skip(reader, b" ")?;

		let status = extract::until(reader, b" ")?;
		let status = std::str::from_utf8(&status)?.parse::<u16>()?;

		response.status = status;

		// skip rest of line
		extract::until(reader, b"\r\n")?;

		let (headers, content_length) = header::from_reader(reader)?;
		let mut response = Self {
			headers,
			status,
			body: None,
		};

		if let Some(content_length) = content_length {
			let mut body = Vec::with_capacity(content_length);

			reader.take(content_length as u64).read_to_end(&mut body)?;

			response.body = Some(body);
		}

		Ok(response)
	}

	pub fn write<W>(&self, sink: &mut W) -> io::Result<()>
	where
		W: Write,
	{
		write!(sink, "HTTP/1.1 {status}\r\n", status = self.status)?;

		if let Some(body) = &self.body {
			sink.write_all(header::CONTENT_LENGTH.as_bytes())?;
			write!(sink, ": {}\r\n", body.len())?;
		}

		for header in &self.headers {
			write!(
				sink,
				"{name}: {value}\r\n",
				name = header.name,
				value = header.value
			)?;
		}

		write!(sink, "\r\n")?;

		if let Some(body) = &self.body {
			sink.write_all(body)?;
		}

		Ok(())
	}

	pub fn header<H>(&mut self, header: H)
	where
		H: IntoHeader<'h>,
	{
		self.headers.push(header.into_header());
	}
}

#[derive(Debug)]
pub struct ResponseBuilder<'h> {
	response: Response<'h>,
}

impl Default for ResponseBuilder<'_> {
	fn default() -> Self {
		Self {
			response: Response {
				headers: vec![],
				status: 200,
				body: None,
			},
		}
	}
}

impl<'h> ResponseBuilder<'h> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn status(mut self, status: u16) -> Self {
		self.response.status = status;
		self
	}

	pub fn body(mut self, body: Vec<u8>) -> Self {
		self.response.body = Some(body);
		self
	}

	pub fn build(self) -> Response<'h> {
		self.response
	}

	pub fn header<H>(mut self, header: H) -> Self
	where
		H: IntoHeader<'h>,
	{
		self.response.header(header);
		self
	}

	#[cfg(feature = "json")]
	pub fn json<T: Serialize>(mut self, payload: &T) -> Result<Self, Error> {
		let bytes = serde_json::to_vec(payload)?;

		self.response.body = Some(bytes);
		Ok(self.header(header::CONTENT_TYPE_JSON))
	}

	#[cfg(feature = "xml")]
	pub fn xml<T: Serialize>(mut self, payload: &T) -> Result<Self, Error> {
		let bytes = quick_xml::se::to_string(payload)?.into_bytes();

		self.response.body = Some(bytes);
		Ok(self.header(header::CONTENT_TYPE_XML))
	}
}
