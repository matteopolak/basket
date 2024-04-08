use std::{
	io::{self, Write},
	net::TcpListener,
};

use crate::{Error, Request, Response, ResponseBuilder};

pub type Handler<S> = fn(S, Request) -> Response;

/// A simple HTTP router.
#[must_use]
#[derive(Debug)]
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

	/// Listens for incoming connections on the provided listener.
	///
	/// # Errors
	/// - If an error occurs while accepting a connection.
	/// - If an error occurs while reading from the connection.
	/// - If an error occurs while writing to the connection.
	pub fn listen(self, listener: &TcpListener) -> Result<!, Error> {
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

			let response = self
				.routes
				.iter()
				.find(|(route, _)| path.starts_with(route))
				.map_or_else(
					|| Response::builder().status(404).build(),
					|(_, handler)| handler(self.state.clone(), request),
				);

			let mut stream = reader.into_inner();
			let response: ResponseBuilder = response.into();

			response
				.header(("server", "basket"))
				.build()
				.write(&mut stream)?;
			stream.flush()?;
		}
	}
}
