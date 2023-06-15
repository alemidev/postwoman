use std::str::FromStr;

use postman_collection::{v1_0_0, v2_0_0, v2_1_0};

pub trait IntoRequest {
	fn make_request(&self) -> reqwest::Request;
}

impl IntoRequest for v2_0_0::RequestClass {
	fn make_request(&self) -> reqwest::Request {
		todo!()
	}
}

impl IntoRequest for v2_1_0::RequestClass {
	fn make_request(&self) -> reqwest::Request {
		let method = reqwest::Method::from_bytes(
			&self.method.as_ref().unwrap_or(&"GET".into()).as_bytes() // TODO lol?
		).unwrap_or(reqwest::Method::GET); // TODO throw an error rather than replacing it silently

		let url_str = match &self.url {
			Some(v2_1_0::Url::String(x)) => x,
			Some(v2_1_0::Url::UrlClass(v2_1_0::UrlClass { raw: Some(x), .. })) => x,
			// TODO compose URL from UrlClass rather than only accepting those with raw set
			_ => "http://localhost",
		};

		let url = reqwest::Url::from_str(&url_str).unwrap();

		let mut out = reqwest::Client::new().request(method, url);

		match &self.header {
			Some(v2_1_0::HeaderUnion::HeaderArray(x)) => {
				for h in x {
					out = out.header(h.key.clone(), h.value.clone()); // TODO avoid cloning
				}
			},
			_ => {},
		}

		match &self.body {
			Some(v2_1_0::Body { raw: Some(x), .. }) => {
				out = out.body(x.clone()) // TODO try to avoid cloning?
			},
			_ => {},
		}

		out.build().unwrap() // TODO what about this?
	}
}

impl IntoRequest for v1_0_0::Request {
	fn make_request(&self) -> reqwest::Request {
		todo!()
	}
}
