use std::str::FromStr;

use reqwest::{Method, Result, Client, Response, Url};

impl Request {
	pub async fn send(self) -> Result<Response> {
		match self {
			Self::Object { url, method, header, body, description: _ } => {
				let method = Method::from_bytes(
					method.as_bytes() // TODO lol?
				).unwrap_or(Method::GET); // TODO throw an error rather than replacing it silently

				let url = Url::from_str(&url.to_string()).unwrap();

				let mut req = Client::new().request(method, url);

				if let Some(headers) = header {
					for h in headers {
						req = req.header(h.key.clone(), h.value.clone())
					}
				}

				if let Some(body) = body {
					req = req.body(body.to_string());
				}

				req.send().await
			},
			Self::String(url) => reqwest::get(url).await
		}
	}
}

impl PostWomanCollection { // TODO repeated code from Item.collect()
	pub fn collect(&self) -> Vec<Request> {
		let mut out = Vec::new();
		for i in &self.item {
			out.append(&mut i.collect())
		}
		out
	}
}

impl Item {
	pub fn collect(&self) -> Vec<Request> {
		let mut out = Vec::new(); // TODO very inefficient to always allocate

		if let Some(items) = &self.item {
			for item in items {
				out.append(&mut item.collect());
			}
		}

		if let Some(req) = &self.request {
			out.push(req.clone());
		}

		out
	}
}
