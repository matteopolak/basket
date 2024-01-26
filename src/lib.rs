pub mod error;
pub mod header;
pub mod request;
pub mod response;

pub use error::Error;
pub use header::*;
pub use request::*;
pub use response::*;

#[cfg(test)]
mod tests {
	use super::*;

	use serde::{Deserialize, Serialize};

	#[test]
	fn test_get_request() {
		let response = Request::get("http://localhost:1337").send().unwrap();

		assert_eq!(response.status(), 200);
		assert_eq!(response.text().unwrap(), "hello, world!");
	}

	#[test]
	fn test_post_request() {
		const INPUT: &str = "Hello, world!";

		let response = Request::post("http://localhost:1337/text")
			.body(INPUT)
			.send()
			.unwrap();

		assert_eq!(response.status(), 200);
		assert_eq!(response.text().unwrap(), INPUT);
	}

	#[test]
	fn test_json_request() {
		#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
		struct Input {
			name: String,
			age: u8,
		}

		let input = Input {
			name: "John Doe".into(),
			age: 42,
		};

		let response = Request::post("http://localhost:1337/json")
			.json(&input)
			.send()
			.unwrap();

		assert_eq!(response.status(), 200);
		assert_eq!(response.json::<Input>().unwrap(), input);
	}

	#[test]
	fn test_get_request_status() {
		let response = Request::get("http://localhost:1337/status/418")
			.send()
			.unwrap();

		assert_eq!(response.status(), 418);
	}
}
