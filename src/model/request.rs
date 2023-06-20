use std::str::FromStr;

use postman_collection::{v1_0_0, v2_0_0, v2_1_0};

fn fill_from_env(mut txt: String) -> String {
	for (k, v) in std::env::vars() {
		let key = format!("{{{{{}}}}}", k);
		if txt.contains(&key) {
			txt = txt.replace(&key, &v);
		}
	}
	txt
}

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

		let mut url_str = match &self.url {
			Some(v2_1_0::Url::String(x)) => x.clone(),
			Some(v2_1_0::Url::UrlClass(v2_1_0::UrlClass { raw: Some(x), .. })) => x.clone(),
			// TODO compose URL from UrlClass rather than only accepting those with raw set
			_ => "http://localhost".into(),
		};

		url_str = fill_from_env(url_str);

		let url = reqwest::Url::from_str(&url_str).unwrap_or_else(|e| {
			eprintln!("error creating url ({}), falling back to localhost", e);
			reqwest::Url::from_str("http://localhost/").unwrap()
		});

		let mut out = reqwest::Client::new().request(method, url);

		// TODO handle more auth types than just bearer
		if let Some(auth) = &self.auth {
			if let Some(bearers) = &auth.bearer {
				for bearer in bearers {
					if let Some(value) = &bearer.value {
						out = out.header("Authorization", format!("Bearer {}", value.as_str().unwrap_or(&value.to_string())))
					}
				}
			}
		}

		match &self.header {
			Some(v2_1_0::HeaderUnion::HeaderArray(x)) => {
				for h in x {
					let k = fill_from_env(h.key.clone());
					let v = fill_from_env(h.value.clone());
					out = out.header(k, v); // TODO avoid cloning
				}
			},
			_ => {},
		}


		match &self.body {
			Some(v2_1_0::Body { raw: Some(x), .. }) => {
				out = out.body(fill_from_env(x.clone())) // TODO try to avoid cloning?
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
