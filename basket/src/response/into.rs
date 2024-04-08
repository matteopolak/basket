use crate::Response;

pub trait IntoResponse<'h> {
	fn into_response(self) -> Response<'h>;
}

impl<'h> IntoResponse<'h> for Response<'h> {
	fn into_response(self) -> Response<'h> {
		self
	}
}

impl IntoResponse<'_> for String {
	fn into_response(self) -> Response<'static> {
		Response::builder().body(self.into_bytes()).build()
	}
}

impl IntoResponse<'_> for Vec<u8> {
	fn into_response(self) -> Response<'static> {
		Response::builder().body(self).build()
	}
}

impl IntoResponse<'_> for &'static str {
	fn into_response(self) -> Response<'static> {
		Response::builder().body(self.as_bytes().to_vec()).build()
	}
}

impl IntoResponse<'_> for () {
	fn into_response(self) -> Response<'static> {
		Response::builder().status(204).build()
	}
}

impl IntoResponse<'_> for u16 {
	fn into_response(self) -> Response<'static> {
		Response::builder().status(self).build()
	}
}

impl<'h, T> IntoResponse<'h> for (u16, T)
where
	T: IntoResponse<'h>,
{
	fn into_response(self) -> Response<'static> {
		let mut builder = Response::builder().status(self.0);

		if let Some(body) = self.1.into_response().body {
			builder = builder.body(body);
		}

		builder.build()
	}
}
