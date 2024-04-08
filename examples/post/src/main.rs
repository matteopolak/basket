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

fn main() {
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
