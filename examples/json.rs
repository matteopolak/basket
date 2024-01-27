#[cfg(feature = "json")]
fn main() {
	use basket::Request;

	#[derive(serde::Deserialize)]
	struct Output {
		slideshow: Slideshow,
	}

	#[derive(serde::Deserialize)]
	struct Slideshow {
		title: String,
	}

	let response = Request::get("http://httpbin.org/json")
		.send()
		.expect("could not send request");

	let json = response.json::<Output>().expect("could not parse body");

	assert_eq!(json.slideshow.title, "Sample Slide Show");
}

#[cfg(not(feature = "json"))]
fn main() {
	panic!("this example requires the `json` feature to be enabled");
}
