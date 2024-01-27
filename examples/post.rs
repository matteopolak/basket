#[cfg(feature = "json")]
fn main() {
	use basket::Request;

	#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
	struct Input {
		name: String,
		age: u8,
	}

	#[derive(serde::Deserialize, Debug)]
	struct Output {
		json: Input,
	}

	let input = Input {
		name: "John".into(),
		age: 20,
	};

	let response = Request::post("http://httpbin.org/anything")
		.json(&input)
		.send()
		.expect("could not send request");

	let body = response.json::<Output>().unwrap();

	assert_eq!(input, body.json);
}

#[cfg(not(feature = "json"))]
fn main() {
	panic!("this example requires the `json` feature to be enabled");
}
