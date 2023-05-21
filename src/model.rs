use std::str::FromStr;

use reqwest::{Method, Result, Client, Response, Url};

use crate::proto::Item;

impl Item {
	pub async fn send(&self) -> Result<Response> {
		let method = Method::from_bytes(
			self.request.method.as_bytes() // TODO lol?
		).unwrap_or(Method::GET); // TODO throw an error rather than replacing it silently

		let url = Url::from_str(&self.request.url.to_string()).unwrap();

		let mut req = Client::new().request(method, url);

		if let Some(headers) = &self.request.header {
			for h in headers {
				req = req.header(h.key.clone(), h.value.clone())
			}
		}

		if let Some(body) = &self.request.body {
			req = req.body(body.to_string());
		}

		req.send().await
	}
}
