use http_client::Request;

#[derive(serde::Serialize)]
struct Input {
	name: String,
	age: u8,
}

#[derive(serde::Deserialize)]
struct Output {
	method: String,
}

fn main() {
	let response = Request::post("http://httpbin.org/anything")
		.json(&Input {
			name: "John".into(),
			age: 20,
		})
		.send()
		.expect("could not send request");

	let json = response.json::<Output>().expect("could not parse body");

	assert_eq!(json.method, "POST");
}
