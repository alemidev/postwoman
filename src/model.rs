use std::str::FromStr;

use reqwest::{header::{HeaderMap, HeaderName, HeaderValue}};

#[derive(Debug, thiserror::Error)]
pub enum PostWomanError {
	#[error("network error: {0:?}")]
	Request(#[from] reqwest::Error),

	#[error("invalid method: {0:?}")]
	InvalidMethod(#[from] http::method::InvalidMethod),

	#[error("invalid header name: {0:?}")]
	InvalidHeaderName(#[from] reqwest::header::InvalidHeaderName),

	#[error("invalid header value: {0:?}")]
	InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

	#[error("contains Unprintable characters: {0:?}")]
	Unprintable(#[from] reqwest::header::ToStrError),

	#[error("header '{0}' not found in response")]
	HeaderNotFound(String),

	#[error("invalid header: '{0}'")]
	InvalidHeader(String),

	#[error("error opening collection: {0:?}")]
	ErrorOpeningCollection(#[from] std::io::Error),

	#[error("collection is not valid toml: {0:?}")]
	InvalidCollection(#[from] toml::de::Error),

	#[error("could not represent collection: {0:?}")] // should never happen
	ErrorSerializingInternallyCollection(#[from] toml_edit::ser::Error),

	#[error("invalid json payload: {0:?}")]
	InvalidJson(#[from] serde_json::Error),

	#[error("invalid regex: {0:?}")]
	InvalidRegex(#[from] regex::Error),
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Endpoint {
	/// endpoint url, required
	pub url: String,
	/// http method for request, default GET
	pub method: Option<String>,
	/// query parameters, appended to base url
	pub query: Option<Vec<String>>,
	/// headers for request, array of "key: value" pairs
	pub headers: Option<Vec<String>>,
	/// body, optional string
	pub body: Option<StringOr<toml::Table>>,
	/// response extractor
	pub extract: Option<StringOr<Extractor>>,
}

fn replace_recursive(element: toml::Value, from: &str, to: &str) -> toml::Value {
	match element {
		toml::Value::Float(x) => toml::Value::Float(x),
		toml::Value::Integer(x) => toml::Value::Integer(x),
		toml::Value::Boolean(x) => toml::Value::Boolean(x),
		toml::Value::Datetime(x) => toml::Value::Datetime(x),
		toml::Value::String(x) => toml::Value::String(x.replace(from, to)),
		toml::Value::Array(x) => toml::Value::Array(
			x.into_iter().map(|x| replace_recursive(x, from, to)).collect()
		),
		toml::Value::Table(map) => {
			let mut out = toml::map::Map::new();
			for (k, v) in map {
				let new_v = replace_recursive(v.clone(), from, to);
				if k.contains(from) {
					out.insert(k.replace(from, to), new_v);
				} else {
					out.insert(k.to_string(), new_v);
				}
			}
			toml::Value::Table(out)
		},
	}
}

impl Endpoint {
	pub fn fill(mut self) -> Self {
		for (k, v) in std::env::vars() {
			let k_var = format!("${{{k}}}");
			self.url = self.url.replace(&k_var, &v);
			if let Some(method) = self.method {
				self.method = Some(method.replace(&k_var, &v));
			}
			if let Some(b) = self.body {
				match b {
					StringOr::Str(body) => {
						self.body = Some(StringOr::Str(body.replace(&k_var, &v)));
					},
					StringOr::T(json) => {
						let wrap = toml::Value::Table(json.clone());
						let toml::Value::Table(out) = replace_recursive(wrap, &k_var, &v)
						else { unreachable!("we put in a table, we get out a table") };
						self.body = Some(StringOr::T(out));
					},
				}
			}
			if let Some(headers) = self.headers {
				self.headers = Some(
					headers.into_iter()
						.map(|x| x.replace(&k_var, &v))
						.collect()
				);
			}
		}
		
		self
	}

	pub async fn execute(self) -> Result<String, PostWomanError> {
		let method = match self.method {
			Some(m) => reqwest::Method::from_str(&m)?,
			None => reqwest::Method::GET,
		};
		let mut headers = HeaderMap::default();
		for header in self.headers.unwrap_or_default() {
			let (k, v) = header.split_once(':')
				.ok_or_else(|| PostWomanError::InvalidHeader(header.clone()))?;
			headers.insert(
				HeaderName::from_str(k)?,
				HeaderValue::from_str(v)?
			);
		}
		let body = match self.body.unwrap_or_default() {
			StringOr::Str(x) => x,
			StringOr::T(json) => serde_json::to_string(&json)?,
		};
		let res = reqwest::Client::new()
			.request(method, self.url)
			.headers(headers)
			.body(body)
			.send()
			.await?
			.error_for_status()?;

		Ok(match self.extract.unwrap_or_default() {
			StringOr::Str(query) => todo!(),
			StringOr::T(Extractor::Debug) => format!("{res:#?}"),
			StringOr::T(Extractor::Body) => res.text().await?,
			StringOr::T(Extractor::Header { key }) => res
				.headers()
				.get(&key)
				.ok_or(PostWomanError::HeaderNotFound(key))?
				.to_str()?
				.to_string(),
		})
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum StringOr<T> {
	Str(String),
	T(T),
}

impl<T: Default> Default for StringOr<T> {
	fn default() -> Self {
		Self::T(T::default())
	}
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Extractor {
	#[default]
	Debug,
	Body,
	// JQL { query: String },
	// Regex { pattern: String },
	Header { key: String },
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PostWomanClient {
	pub user_agent: Option<String>,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PostWomanConfig {
	pub client: PostWomanClient,
	// it's weird to name it singular but makes more sense in config
	pub route: indexmap::IndexMap<String, Endpoint>,
}
