use std::io::{self, Read, Write};
use std::net::TcpStream;

#[cfg(feature = "json")]
use serde::Serialize;
use url::{ParseError, Url};

use crate::Error;

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
}

#[derive(Debug)]
pub struct Request {
	url: Url,
	method: Method,
	body: Option<Vec<u8>>,
	headers: Vec<Header>,
}

impl Request {
	pub fn delete<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder {
		RequestBuilder::new(Method::Delete, url)
	}

	pub fn get<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder {
		RequestBuilder::new(Method::Get, url)
	}

	pub fn options<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder {
		RequestBuilder::new(Method::Options, url)
	}

	pub fn patch<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder {
		RequestBuilder::new(Method::Patch, url)
	}

	pub fn post<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder {
		RequestBuilder::new(Method::Post, url)
	}

	pub fn put<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder {
		RequestBuilder::new(Method::Put, url)
	}

	pub fn send(self) -> Result<Response, Error> {
		let mut stream = TcpStream::connect(self.url.socket_addrs(|| None)?.as_slice())?;

		self.write(&mut stream)?;
		stream.flush()?;

		let mut sink = Vec::new();

		// NOTE: this works because of the `Connection: close` header
		stream.read_to_end(&mut sink)?;

		Response::from_bytes(sink)
	}

	fn write(&self, stream: &mut TcpStream) -> io::Result<()> {
		write!(stream, "{} {}", self.method.as_str(), self.url.path())?;

		if let Some(query) = self.url.query() {
			stream.write_all(query.as_bytes())?;
		}

		stream.write_all(b" HTTP/1.1\r\n")?;

		for header in &self.headers {
			write!(stream, "{}: {}\r\n", header.name, header.value)?;
		}

		stream.write_all(b"\r\n")?;

		if let Some(body) = &self.body {
			stream.write_all(body.as_slice())?;
		}

		Ok(())
	}
}

pub struct RequestBuilder {
	request: Request,
	error: Option<Error>,
}

impl RequestBuilder {
	pub fn new<U: TryInto<Url, Error = ParseError>>(method: Method, url: U) -> Self {
		let url = url.try_into().unwrap();
		let mut headers = vec![Header {
			name: "connection".into(),
			value: "close".into(),
		}];

		if let Some(host) = url.host_str() {
			headers.push(Header {
				name: "host".into(),
				value: host.into(),
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

	pub fn send(self) -> Result<Response, Error> {
		if let Some(error) = self.error {
			return Err(error);
		}

		if let Some(body) = self.request.body.as_ref() {
			let len = body.len();

			self.header("content-length", format!("{}", len))
		} else {
			self
		}
		.request
		.send()
	}

	pub fn header<N: Into<String>, V: Into<String>>(mut self, name: N, value: V) -> Self {
		self.request.headers.push(Header {
			name: name.into(),
			value: value.into(),
		});

		self
	}

	#[cfg(feature = "json")]
	pub fn json<T: Serialize + ?Sized>(mut self, payload: &T) -> Self {
		let bytes = match serde_json::to_vec(payload) {
			Ok(b) => b,
			Err(e) => {
				self.error = Some(e.into());

				return self;
			}
		};

		self.request.body = Some(bytes);
		self.header("content-type", "application/json")
	}

	#[cfg(feature = "xml")]
	pub fn xml<T: Serialize + ?Sized>(mut self, payload: &T) -> Self {
		let bytes = match quick_xml::se::to_string(payload) {
			Ok(b) => b.into_bytes(),
			Err(e) => {
				self.error = Some(e.into());

				return self;
			}
		};

		self.request.body = Some(bytes);
		self.header("content-type", "application/xml")
	}

	pub fn body<T: Into<Vec<u8>>>(mut self, payload: T) -> Self {
		let bytes: Vec<u8> = payload.into();

		self.request.body = Some(bytes);
		self.header("content-type", "text/plain")
	}
}
