#[cfg(feature = "xml")]
fn main() {
	use basket::Request;

	#[derive(serde::Deserialize)]
	struct Output {
		#[serde(rename = "@title")]
		title: String,
	}

	let response = Request::get("http://httpbin.org/xml")
		.send()
		.expect("could not send request");

	let xml = response.xml::<Output>().expect("could not parse body");

	assert_eq!(xml.title, "Sample Slide Show");
}

#[cfg(not(feature = "xml"))]
fn main() {
	panic!("this example requires the `xml` feature to be enabled");
}
