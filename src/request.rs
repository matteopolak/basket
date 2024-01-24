use std::io::{self, Read, Write};
use std::net::TcpStream;

use serde::Serialize;
use url::{ParseError, Url};

use crate::Error;

use super::header::Header;
use super::response::Response;

#[derive(Debug)]
pub enum Method {
	Get,
	Post,
}

impl Method {
	pub fn as_bytes(&self) -> &[u8] {
		match self {
			Self::Get => b"GET",
			Self::Post => b"POST",
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
	pub const BUF_SIZE: usize = 1024;

	pub fn get<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder {
		RequestBuilder::new(Method::Get, url)
	}

	pub fn post<U: TryInto<Url, Error = ParseError>>(url: U) -> RequestBuilder {
		RequestBuilder::new(Method::Post, url)
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
		stream.write_all(self.method.as_bytes())?;
		stream.write_all(b" ")?;
		stream.write_all(self.url.path().as_bytes())?;

		if let Some(query) = self.url.query() {
			stream.write_all(query.as_bytes())?;
		}

		stream.write_all(b" HTTP/1.1\r\n")?;

		for header in &self.headers {
			stream.write_all(header.name.as_bytes())?;
			stream.write_all(b": ")?;
			stream.write_all(header.value.as_bytes())?;
			stream.write_all(b"\r\n")?;
		}

		if let Some(body) = &self.body {
			stream.write_all(b"\r\n")?;
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
			})
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

		self.request.send()
	}

	pub fn header<N: Into<String>, V: Into<String>>(mut self, name: N, value: V) -> Self {
		self.request.headers.push(Header {
			name: name.into(),
			value: value.into(),
		});
		self
	}

	pub fn json<T: Serialize + ?Sized>(mut self, payload: &T) -> Self {
		let bytes = match serde_json::to_vec(payload) {
			Ok(b) => b,
			Err(e) => {
				self.error = Some(e.into());

				return self;
			}
		};

		let len = bytes.len();

		self.request.body = Some(bytes);
		self.header("content-length", format!("{len}"))
			.header("content-type", "application/json")
	}
}
