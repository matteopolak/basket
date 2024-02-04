use basket::RequestBuilder;
use clap::{Parser, ValueEnum, ValueHint};

/// A simple HTTP client.
///
/// # Examples
///
/// basket get http://example.com
///
/// basket post http://example.com --data "Hello, world!"
///
/// basket post http://example.com --json --data '{"message": "Hello, world!"}'
#[derive(Parser)]
struct Args {
	#[arg(value_name = "METHOD")]
	method: Method,
	#[arg(value_name = "URL", value_hint = ValueHint::Url)]
	url: String,
	#[arg(short, long)]
	data: Option<String>,
	#[arg(short, long)]
	json: bool,
	#[arg(short = 'H', long, value_parser = clap::value_parser!(Header))]
	headers: Vec<Header>,
}

#[derive(Clone)]
struct Header(String, String);

impl From<String> for Header {
	fn from(value: String) -> Self {
		let mut parts = value.splitn(2, ':');
		let name = parts.next().expect("header name").trim_start();
		let value = parts.next().expect("header value").trim_start();

		Self(name.into(), value.into())
	}
}

#[derive(Clone, Copy, ValueEnum)]
enum Method {
	Delete,
	Get,
	Options,
	Patch,
	Post,
	Put,
}

impl From<Method> for basket::request::Method {
	fn from(value: Method) -> Self {
		match value {
			Method::Delete => Self::Delete,
			Method::Get => Self::Get,
			Method::Options => Self::Options,
			Method::Patch => Self::Patch,
			Method::Post => Self::Post,
			Method::Put => Self::Put,
		}
	}
}

fn main() {
	let args = Args::parse();
	let mut request = RequestBuilder::new(args.method.into(), args.url.as_str());

	if let Some(data) = args.data {
		if args.json {
			request = request.json(&data);
		} else {
			request = request.body(data);
		}
	}

	for header in args.headers {
		request = request.header(header.0, header.1);
	}

	let response = request.send().expect("could not send request");

	println!("{}", response);
}
