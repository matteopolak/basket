use std::net::TcpListener;

use basket::{server::Router, IntoResponse, Request, Response};
use serde::{Deserialize, Serialize};

fn main() {
	let listener = TcpListener::bind(("0.0.0.0", 3000)).unwrap();
	let router = Router::new(())
		.route("/hello", hello)
		.route("/world", world)
		.route("/", index);

	println!("listening on http://localhost:3000");

	router.listen(&listener).unwrap();
}

#[derive(Deserialize, Serialize)]
struct Person {
	name: String,
	age: u8,
}

fn hello(_: (), _: Request) -> Response {
	(200u16, "hello").into_response()
}

fn world(_: (), _: Request) -> Response {
	(200u16, "world").into_response()
}

fn index(_: (), request: Request) -> Response {
	let json = request.json::<Person>();
	let Ok(json) = json else {
		return Response::builder()
			.status(400)
			.body("invalid json".into())
			.build();
	};

	Response::builder().status(200).json(&json).unwrap().build()
}
