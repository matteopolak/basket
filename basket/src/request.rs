use std::io::{self, Read, Write};
use std::net::TcpStream;

#[cfg(any(feature = "json", feature = "xml"))]
use serde::{de::DeserializeOwned, Serialize};
use url::{ParseError, Url};

use crate::{extract, header};
use crate::{Error, IntoHeader};

use super::header::Header;
use super::response::Response;

#[derive(Debug)]
pub enum Method {
	Delete,
	Get,
	Options,
	Patch,
	Post,
	Put,
}

impl Method {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Delete => "DELETE",
			Self::Get => "GET",
			Self::Options => "OPTIONS",
			Self::Patch => "PATCH",
			Self::Post => "POST",
			Self::Put => "PUT",
		}
	}

	pub fn from_bytes(value: &[u8]) -> Result<Method, Error> {
		Ok(match value {
			b"DELETE" => Self::Delete,
			b"GET" => Self::Get,
			b"OPTIONS" => Self::Options,
			b"PATCH" => Self::Patch,
			b"POST" => Self::Post,
			b"PUT" => Self::Put,
			_ => return Err(Error::UnknownMethod),
		})
	}
}

#[derive(Debug)]
pub struct Request<'h> {
	pub url: Url,
	pub method: Method,
	pub body: Option<Vec<u8>>,
	pub headers: Vec<Header<'h>>,
}

impl<'h> Request<'h> {
	pub fn delete<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder<'h> {
		RequestBuilder::new(Method::Delete, url)
	}

	pub fn get<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder<'h> {
		RequestBuilder::new(Method::Get, url)
	}

	pub fn options<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder<'h> {
		RequestBuilder::new(Method::Options, url)
	}

	pub fn patch<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder<'h> {
		RequestBuilder::new(Method::Patch, url)
	}

	pub fn post<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder<'h> {
		RequestBuilder::new(Method::Post, url)
	}

	pub fn put<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder<'h> {
		RequestBuilder::new(Method::Put, url)
	}

	pub fn from_reader<R>(reader: &mut R) -> Result<Self, Error>
	where
		R: Read,
	{
		let method = extract::until(reader, b" ")?;
		let method = Method::from_bytes(&method)?;
		let path = extract::until(reader, b" ")?;

		extract::http_version(reader)?;
		extract::skip(reader, b"\r\n")?;

		let url = Url::parse(&format!("data:{}", String::from_utf8(path)?))?;
		let (headers, content_length) = header::from_reader(reader)?;

		let mut request = Self {
			headers,
			method,
			url,
			body: None,
		};

		if let Some(content_length) = content_length {
			let mut body = Vec::with_capacity(content_length);

			reader.take(content_length as u64).read_to_end(&mut body)?;

			request.body = Some(body);
		}

		Ok(request)
	}

	pub fn send(self) -> Result<Response<'h>, Error> {
		let mut stream = TcpStream::connect(self.url.socket_addrs(|| None)?.as_slice())?;

		self.write(&mut stream)?;
		stream.flush()?;

		// Response::from_reader reads a lot of tiny chunks from the stream
		// so we wrap it in a BufReader to reduce the overhead
		let mut reader = io::BufReader::new(stream);

		Response::from_reader(&mut reader)
	}

	fn write<W>(&self, write: &mut W) -> io::Result<()>
	where
		W: Write,
	{
		write!(write, "{} {}", self.method.as_str(), self.url.path())?;

		if let Some(query) = self.url.query() {
			write.write_all(query.as_bytes())?;
		}

		write.write_all(b" HTTP/1.1\r\n")?;

		for header in &self.headers {
			write!(write, "{}: {}\r\n", header.name, header.value)?;
		}

		write.write_all(b"\r\n")?;

		if let Some(body) = &self.body {
			write.write_all(body.as_slice())?;
		}

		Ok(())
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
}

#[derive(Debug)]
pub struct RequestBuilder<'h> {
	request: Request<'h>,
	error: Option<Error>,
}

impl<'h> RequestBuilder<'h> {
	pub fn new<U: TryInto<Url, Error = ParseError>>(method: Method, url: U) -> Self {
		let url = url.try_into().unwrap();
		let mut headers = vec![Header {
			name: "connection".into(),
			value: "close".into(),
		}];

		if let Some(host) = url.host_str() {
			headers.push(Header {
				name: "host".into(),
				value: host.to_string().into(),
			});
		}

		Self {
			error: None,
			request: Request {
				method,
				body: None,
				headers,
				url,
			},
		}
	}

	pub fn send(self) -> Result<Response<'h>, Error> {
		if let Some(error) = self.error {
			return Err(error);
		}

		if let Some(body) = self.request.body.as_ref() {
			let len = body.len();

			self.header((header::CONTENT_LENGTH, format!("{}", len)))
		} else {
			self
		}
		.request
		.send()
	}

	pub fn header<H>(mut self, header: H) -> Self
	where
		H: IntoHeader<'h>,
	{
		self.request.headers.push(header.into_header());
		self
	}

	#[cfg(feature = "json")]
	pub fn json<T: Serialize>(mut self, payload: &T) -> Self {
		let bytes = match serde_json::to_vec(payload) {
			Ok(b) => b,
			Err(e) => {
				self.error = Some(e.into());

				return self;
			}
		};

		self.request.body = Some(bytes);
		self.header(header::CONTENT_TYPE_JSON)
	}

	#[cfg(feature = "xml")]
	pub fn xml<T: Serialize>(mut self, payload: &T) -> Self {
		let bytes = match quick_xml::se::to_string(payload) {
			Ok(b) => b.into_bytes(),
			Err(e) => {
				self.error = Some(e.into());

				return self;
			}
		};

		self.request.body = Some(bytes);
		self.header(header::CONTENT_TYPE_XML)
	}

	pub fn body<T: Into<Vec<u8>>>(mut self, payload: T) -> Self {
		let bytes: Vec<u8> = payload.into();

		self.request.body = Some(bytes);
		self.header(header::CONTENT_TYPE_PLAIN)
	}
}
