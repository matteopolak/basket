pub mod into;
pub use into::*;

use std::io;
use std::io::{Read, Write};

#[cfg(any(feature = "json", feature = "xml"))]
use serde::{de::DeserializeOwned, Serialize};

use crate::header;
use crate::Error;
use crate::{extract, IntoHeader};

use super::header::Header;

#[must_use]
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

	#[must_use]
	pub fn status(&self) -> u16 {
		self.status
	}

	/// Parses the body as JSON.
	///
	/// # Errors
	/// - If the body is not present.
	/// - Forwards errors from [``serde_json``].
	#[cfg(feature = "json")]
	pub fn json<T: DeserializeOwned>(self) -> Result<T, Error> {
		let Some(body) = self.body else {
			return Err(Error::ExpectedBody);
		};

		Ok(serde_json::from_slice(&body)?)
	}

	/// Parses the body as XML.
	///
	/// # Errors
	/// - If the body is not present.
	/// - Forwards errors from [``quick_xml``].
	#[cfg(feature = "xml")]
	pub fn xml<T: DeserializeOwned>(self) -> Result<T, Error> {
		let Some(body) = self.body else {
			return Err(Error::ExpectedBody);
		};

		let content = std::str::from_utf8(&body)?;

		Ok(quick_xml::de::from_str(content)?)
	}

	/// Returns the body as a string.
	///
	/// # Errors
	/// - If the body is not present.
	/// - If the body is not valid UTF-8.
	pub fn text(self) -> Result<String, Error> {
		let Some(body) = self.body else {
			return Err(Error::ExpectedBody);
		};

		String::from_utf8(body).map_err(|e| e.utf8_error().into())
	}

	/// Returns the body as a byte vector.
	///
	/// # Errors
	/// - If the body is not present.
	pub fn bytes(self) -> Result<Vec<u8>, Error> {
		self.body.ok_or(Error::ExpectedBody)
	}

	/// Parses a response from a reader.
	///
	/// # Errors
	/// - If the response does not adhere to the HTTP/1.1 format.
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

		extract::until(reader, b"\r\n")?;

		response.status = status;

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

	/// Writes the response to a writer.
	///
	/// # Errors
	/// - If the response could not be written.
	pub fn write<W>(&self, sink: &mut W) -> io::Result<()>
	where
		W: Write,
	{
		write!(sink, "HTTP/1.1 {status}\r\n", status = self.status)?;

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

	/// Returns the value of a header.
	#[must_use]
	pub fn header(&self, name: &str) -> Option<&str> {
		self.headers
			.iter()
			.find(|header| header.name.eq_ignore_ascii_case(name))
			.map(|header| header.value.as_ref())
	}
}

#[allow(clippy::module_name_repetitions)]
#[must_use]
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

impl<'h> From<Response<'h>> for ResponseBuilder<'h> {
	fn from(response: Response<'h>) -> Self {
		Self { response }
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
		let len = body.len();

		self.response.body = Some(body);
		self.header((header::CONTENT_LENGTH, len))
	}

	pub fn build(self) -> Response<'h> {
		self.response
	}

	pub fn header<H>(mut self, header: H) -> Self
	where
		H: IntoHeader<'h>,
	{
		self.response.headers.push(header.into_header());
		self
	}

	/// Sets the body as JSON.
	///
	/// # Errors
	/// - Forwards errors from [``serde_json``].
	#[cfg(feature = "json")]
	pub fn json<T: Serialize>(self, payload: &T) -> Result<Self, Error> {
		let bytes = serde_json::to_vec(payload)?;

		Ok(self.body(bytes).header(header::CONTENT_TYPE_JSON))
	}

	/// Sets the body as XML.
	///
	/// # Errors
	/// - Forwards errors from [``quick_xml``].
	#[cfg(feature = "xml")]
	pub fn xml<T: Serialize>(self, payload: &T) -> Result<Self, Error> {
		let bytes = quick_xml::se::to_string(payload)?.into_bytes();

		Ok(self.body(bytes).header(header::CONTENT_TYPE_XML))
	}
}
