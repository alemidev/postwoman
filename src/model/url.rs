use postman_collection::{v1_0_0, v2_0_0, v2_1_0};

use super::{query::IntoQueryString, host::IntoHost, path::IntoPath};

pub trait IntoUrl {
	fn make_url(&self) -> String;
}

impl IntoUrl for v1_0_0::Url {
	fn make_url(&self) -> String {
		todo!()
	}
}

impl IntoUrl for v2_0_0::Url {
	fn make_url(&self) -> String {
		todo!()
	}
}

impl IntoUrl for v2_1_0::Url {
	fn make_url(&self) -> String {
		match self {
			v2_1_0::Url::String(x) => x.clone(),
			v2_1_0::Url::UrlClass(x) => {
				match x {
					v2_1_0::UrlClass { raw: Some(x), .. } => x.clone(),
					v2_1_0::UrlClass {
						raw: None,
						hash, host, path, port, protocol, query, variable
					} => build_url(
						&protocol.unwrap_or("http".into()),
						&host.map(|x| x.make_host()).unwrap_or("localhost".into()),
						&path.map(|x| x.make_path().as_str()),
						&query.map(|x| x.make_query().as_str()),
						&hash.map(|x| x.as_str())
					)
				}
			},
		}
	}
}

fn build_url(
	proto: &str,
	host: &str,
	path: &Option<&str>,
	query: &Option<&str>,
	hash: &Option<&str>
) -> String {
	let mut url = format!("{}://{}", proto, host);

	if let Some(p) = path {
		url.push('/');
		url.push_str(p);
	}

	if let Some(q) = query {
		url.push('?');
		url.push_str(&q);
	}

	if let Some(h) = hash {
		url.push('#');
		url.push_str(&h);
	}

	url
}
