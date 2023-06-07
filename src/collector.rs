use std::str::FromStr;
use postman_collection::v2_0_0::{Spec, RequestClass, Items, Url, UrlClass, HeaderUnion, Body};

pub fn collect(collection: Spec) -> Vec<RequestClass> {
	let mut reqs = Vec::new();
	for item in collection.item {
		reqs.append(&mut requests(item)); // TODO creating all these vectors is a waste!
	}
	reqs
}

pub fn requests(root: Items) -> Vec<RequestClass> {
	let mut reqs = Vec::new();

	if let Some(r) = root.request {
		match r {
			postman_collection::v2_0_0::RequestUnion::RequestClass(x) => reqs.push(x),
			postman_collection::v2_0_0::RequestUnion::String(url) => reqs.push(
				RequestClass {
					method: Some("GET".into()),
					url: Some(Url::String(url)),
					..Default::default()
				}
			),
		}
	}

	if let Some(items) = root.item {
		for item in items {
			reqs.append(&mut requests(item));
		}
	}

	reqs
}

pub fn url(req: &RequestClass) -> String {
	match &req.url {
		Some(Url::String(x)) => x.clone(),
		Some(Url::UrlClass(UrlClass { raw: Some(raw) , .. })) => raw.clone(),
		// TODO compose UrlClass
		_ => "".into(),
	}
}

pub async fn send(req: RequestClass) -> reqwest::Result<reqwest::Response> {
	let method = reqwest::Method::from_bytes(
		&req.method.as_ref().unwrap_or(&"GET".into()).as_bytes() // TODO lol?
	).unwrap_or(reqwest::Method::GET); // TODO throw an error rather than replacing it silently

	let url = reqwest::Url::from_str(&url(&req)).unwrap();

	let mut out = reqwest::Client::new().request(method, url);

	match req.header {
		Some(HeaderUnion::HeaderArray(x)) => {
			for h in x {
				out = out.header(h.key, h.value);
			}
		},
		_ => {},
	}

	match req.body {
		Some(Body { raw: Some(x), .. }) => {
			out = out.body(x)
		},
		_ => {},
	}

	out.send().await
}
