use std::collections::HashMap;
use std::str::FromStr;

use base64::{prelude::BASE64_STANDARD, Engine};
use http::method::InvalidMethod;
use http::{HeaderMap, HeaderName, HeaderValue};
use jaq_interpret::FilterT;

use crate::errors::InvalidHeaderError;
use crate::{PostWomanError, APP_USER_AGENT};

use crate::ext::{stringify_json, FillError, FillableFromEnvironment, StringOr};
use super::{ExtractorConfig, ClientConfig};


#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct EndpointConfig {
	/// endpoint path, composed from client base and query params
	pub path: String,
	/// absolute url, don't compose with client base url
	pub absolute: Option<bool>,
	/// http method for request, default GET
	pub method: Option<String>,
	/// query parameters, appended to base url
	pub query: Option<Vec<String>>,
	/// headers for request, array of "key: value" pairs
	pub headers: Option<Vec<String>>,
	/// body, optional string
	pub body: Option<StringOr<toml::Table>>,
	/// expected error code, will fail if different, defaults to 200
	pub status: Option<u16>,
	/// response extractor
	pub extract: Option<StringOr<ExtractorConfig>>,
	/// expected result, will fail if different when provided
	pub expect: Option<String>,
}

impl EndpointConfig {
	pub fn body(&self) -> Result<String, serde_json::Error> {
		match &self.body {
			None => Ok("".to_string()),
			Some(StringOr::Str(x)) => Ok(x.clone()),
			Some(StringOr::T(json)) => Ok(serde_json::to_string(&json)?),
		}
	}

	pub fn method(&self) -> Result<reqwest::Method, InvalidMethod> {
		match self.method {
			Some(ref m) => Ok(reqwest::Method::from_str(m)?),
			None => Ok(reqwest::Method::GET),
		}
	}

	pub fn headers(&self) -> Result<HeaderMap, InvalidHeaderError> {
		let mut headers = HeaderMap::default();
		for header in self.headers.as_deref().unwrap_or(&[]) {
			let (k, v) = header.split_once(':')
				.ok_or_else(|| InvalidHeaderError::Format(header.clone()))?;
			headers.insert(
				HeaderName::from_str(k)?,
				HeaderValue::from_str(v)?
			);
		}
		Ok(headers)
	}

	pub fn url(&self, base: Option<&str>) -> String {
		let mut url = if self.absolute.unwrap_or(false) {
			self.path.clone()
		} else {
			format!("{}{}", base.unwrap_or_default(), self.path)
		};
		if let Some(ref query) = self.query {
			url = format!("{url}?{}", query.join("&"));
		}
		url
	}

	pub async fn execute(self, opts: &ClientConfig) -> Result<String, PostWomanError> {
		let body = self.body()?;
		let method = self.method()?;
		let headers = self.headers()?;
		let url = self.url(opts.base.as_deref());

		let client = reqwest::Client::builder()
			.user_agent(opts.user_agent.as_deref().unwrap_or(APP_USER_AGENT))
			.timeout(std::time::Duration::from_secs(opts.timeout.unwrap_or(30)))
			.redirect(opts.redirects.map(reqwest::redirect::Policy::limited).unwrap_or(reqwest::redirect::Policy::none()))
			.danger_accept_invalid_certs(opts.accept_invalid_certs.unwrap_or(false))
			.build()?;


		let res = client
			.request(method, url)
			.headers(headers)
			.body(body)
			.send()
			.await?;

		if res.status().as_u16() != self.status.unwrap_or(200) {
			return Err(PostWomanError::UnexpectedStatusCode(res));
		}

		let res = match self.extract.unwrap_or_default() {
			StringOr::T(ExtractorConfig::Discard) => "".to_string(),
			StringOr::T(ExtractorConfig::Body) => format_body(res).await?,
			StringOr::T(ExtractorConfig::Debug) => {
				// TODO needless double format
				let res_dbg = format!("{res:#?}");
				let body = format_body(res).await?; 
				format!("{res_dbg}\nBody: {body}")
			},
			StringOr::T(ExtractorConfig::Header { key }) => res
				.headers()
				.get(&key)
				.ok_or(PostWomanError::HeaderNotFound(key))?
				.to_str()?
				.to_string(),
			StringOr::T(ExtractorConfig::Regex { pattern }) => {
				let pattern = regex::Regex::new(&pattern)?;
				let body = format_body(res).await?;
				pattern.find(&body)
					.ok_or_else(|| PostWomanError::NoMatch(body.clone()))?
					.as_str()
					.to_string()
			},
			// bare string defaults to JQL query
			StringOr::T(ExtractorConfig::JQ { query }) | StringOr::Str(query) => {
				let json: serde_json::Value = res.json().await?;
				let selection = jq(&query, json)?;
				if selection.len() == 1 {
					stringify_json(&selection[0])
				} else {
					serde_json::to_string_pretty(&selection)?
				}
			},
		};

		if let Some(expected) = self.expect {
			if expected != res {
				return Err(PostWomanError::UnexpectedResult(res, expected));
			}
		}

		Ok(res)
	}
}

impl FillableFromEnvironment for EndpointConfig {
	fn fill(mut self, env: &toml::Table) -> Result<Self, FillError> {
		let vars = Self::default_vars(env);

		self.path = Self::replace(self.path, &vars)?;
		if let Some(method) = self.method {
			self.method = Some(Self::replace(method, &vars)?);
		}
		if let Some(b) = self.body {
			match b {
				StringOr::Str(body) => {
					self.body = Some(StringOr::Str(Self::replace(body, &vars)?));
				},
				StringOr::T(json) => {
					let wrap = toml::Value::Table(json);
					let toml::Value::Table(out) = replace_recursive(wrap, &vars)?
					else { unreachable!("we put in a table, we get out a table") };
					self.body = Some(StringOr::T(out));
				},
			}
		}
		if let Some(query) = self.query {
			let mut out = Vec::new();
			for q in query {
				out.push(Self::replace(q, &vars)?);
			}
			self.query = Some(out);
		}
		if let Some(headers) = self.headers {
			let mut out = Vec::new();
			for h in headers {
				out.push(Self::replace(h.clone(), &vars)?);
			}
			self.headers = Some(out);
		}
		
		Ok(self)
	}
}

fn replace_recursive(element: toml::Value, env: &HashMap<String, String>) -> Result<toml::Value, FillError> {
	Ok(match element {
		toml::Value::Float(x) => toml::Value::Float(x),
		toml::Value::Integer(x) => toml::Value::Integer(x),
		toml::Value::Boolean(x) => toml::Value::Boolean(x),
		toml::Value::Datetime(x) => toml::Value::Datetime(x),
		toml::Value::String(x) => toml::Value::String(EndpointConfig::replace(x, env)?),
		toml::Value::Array(mut arr) => {
			for v in arr.iter_mut() {
				*v = replace_recursive(v.clone(), env)?;
			}
			toml::Value::Array(arr)
		},
		toml::Value::Table(map) => {
			let mut out = toml::map::Map::new();
			for (k, v) in map {
				let new_v = replace_recursive(v.clone(), env)?;
				let new_k = EndpointConfig::replace(k, env)?;
				out.insert(new_k, new_v);
			}
			toml::Value::Table(out)
		},
	})
}

async fn format_body(res: reqwest::Response) -> Result<String, PostWomanError> {
	let content_type = res.headers().get("Content-Type").cloned();
	let raw = res.bytes().await?;
	match content_type {
		None => Ok(String::from_utf8_lossy(&raw).to_string()),
		Some(v) => {
			let content_type = v.to_str()?;
			if content_type.starts_with("application/json") {
				match serde_json::from_slice::<serde_json::Value>(&raw) {
					Ok(x) => Ok(serde_json::to_string_pretty(&x)?),
					Err(e) => {
						eprintln!(" ? content-type is 'json' but content is not valid json: {e}");
						Ok(String::from_utf8_lossy(&raw).to_string())
					},
				}
			} else if content_type.starts_with("text/plain") || content_type.starts_with("text/html") {
				Ok(String::from_utf8_lossy(&raw).to_string())
			} else {
				Ok(format!("base64({})\n", BASE64_STANDARD.encode(raw)))
			}
		},
	}
}

fn jq(query: &str, value: serde_json::Value) -> Result<Vec<serde_json::Value>, PostWomanError> {
	// TODO am i not getting jaq api? or is it just this weird????
	let mut defs = jaq_interpret::ParseCtx::new(Vec::new());
	let (filter, errs) = jaq_parse::parse(query, jaq_parse::main());
	let Some(filter) = filter else {
		return Err(PostWomanError::JQError(
			errs.into_iter().map(|x| format!("{x:?}")).collect::<Vec<String>>().join(", ")
		));
	};
	let out: Vec<serde_json::Value> = defs
		.compile(filter)
		.run((
			jaq_interpret::Ctx::new([], &jaq_interpret::RcIter::new(core::iter::empty())),
			jaq_interpret::Val::from(value)
		))
		.filter_map(|x| Some(x.ok()?.into()))
		.collect();

	Ok(out)
}
