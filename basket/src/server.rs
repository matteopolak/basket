use std::{
	io::{self, Write},
	net::TcpListener,
};

use crate::{Error, Request, Response};

pub type Handler<S> = fn(S, Request) -> Response;

pub struct Router<'a, S> {
	routes: Vec<(&'a str, Handler<S>)>,
	state: S,
}

impl<'a, S> Router<'a, S>
where
	S: Clone,
{
	pub fn new(state: S) -> Self
	where
		S: Clone,
	{
		Self {
			routes: vec![],
			state,
		}
	}

	/// Adds a new route to the router. To require a trailing slash, add a slash to the end of the route.
	pub fn route(mut self, route: &'a str, handler: Handler<S>) -> Self {
		self.routes.push((route, handler));
		self
	}

	pub fn listen(self, listener: TcpListener) -> Result<!, Error> {
		loop {
			let (stream, _) = listener.accept()?;
			let mut reader = io::BufReader::new(stream);

			let request = Request::from_reader(&mut reader)?;
			let path = request.url.path();

			// remove trailing slash, unless it's the root path
			let path = if path.len() > 1 && path.ends_with('/') {
				&path[..path.len() - 1]
			} else {
				path
			};

			let mut response = self
				.routes
				.iter()
				.find(|(route, _)| path.starts_with(route))
				.map(|(_, handler)| handler(self.state.clone(), request))
				.unwrap_or_else(|| Response::builder().status(404).build());

			let mut stream = reader.into_inner();

			response.header(("server", "basket"));
			response.write(&mut stream)?;
			stream.flush()?;
		}
	}
}
