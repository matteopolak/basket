use basket::Request;

#[derive(serde::Deserialize)]
struct Output {
	#[serde(rename = "@title")]
	title: String,
}

fn main() {
	let response = Request::get("http://httpbin.org/xml")
		.send()
		.expect("could not send request");

	let xml = response.xml::<Output>().expect("could not parse body");

	assert_eq!(xml.title, "Sample Slide Show");
}
