use std::{collections::HashMap, str::FromStr};

use base64::{prelude::BASE64_STANDARD, Engine};
use http::{HeaderMap, HeaderName, HeaderValue};
use jaq_interpret::FilterT;

use crate::{PostWomanError, APP_USER_AGENT};

use super::{Extractor, PostWomanClient, StringOr};


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
	/// expected error code, will fail if different
	pub expect: Option<u16>,
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
	pub fn fill(mut self, env: &toml::Table) -> Self {
		let mut vars: HashMap<String, String> = HashMap::default();

		vars.insert("POSTWOMAN_TIMESTAMP".to_string(), chrono::Local::now().timestamp().to_string());

		for (k, v) in env {
			vars.insert(k.to_string(), crate::ext::stringify_toml(v));
		}

		for (k, v) in std::env::vars() {
			vars.insert(k, v);
		}

		for (k, v) in vars {
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
			if let Some(query) = self.query {
				self.query = Some(
					query.into_iter()
						.map(|x| x.replace(&k_var, &v))
						.collect()
				);
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

	pub async fn execute(self, opts: &PostWomanClient) -> Result<String, PostWomanError> {
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

		let mut url = self.url;
		if let Some(query) = self.query {
			url = format!("{url}?{}", query.join("&"));
		}

		let client = reqwest::Client::builder()
			.user_agent(opts.user_agent.as_deref().unwrap_or(APP_USER_AGENT))
			.build()?;

		let res = client
			.request(method, url)
			.headers(headers)
			.body(body)
			.send()
			.await?;

		if res.status().as_u16() != self.expect.unwrap_or(200) {
			return Err(PostWomanError::UnexpectedStatusCode(res));
		}

		Ok(match self.extract.unwrap_or_default() {
			StringOr::T(Extractor::Discard) => "".to_string(),
			StringOr::T(Extractor::Body) => format_body(res).await?,
			StringOr::T(Extractor::Debug) => {
				// TODO needless double format
				let res_dbg = format!("{res:#?}");
				let body = format_body(res).await?; 
				format!("{res_dbg}\nBody: {body}\n")
			},
			StringOr::T(Extractor::Header { key }) => res
				.headers()
				.get(&key)
				.ok_or(PostWomanError::HeaderNotFound(key))?
				.to_str()?
				.to_string()
				+ "\n",
			StringOr::T(Extractor::Regex { pattern }) => {
				let pattern = regex::Regex::new(&pattern)?;
				let body = format_body(res).await?;
				pattern.find(&body)
					.ok_or_else(|| PostWomanError::NoMatch(body.clone()))?
					.as_str()
					.to_string()
					+ "\n"
			},
			// bare string defaults to JQL query
			StringOr::T(Extractor::Jql { query }) | StringOr::Str(query) => {
				let json: serde_json::Value = res.json().await?;
				let selection = jq(&query, json)?;
				if selection.len() == 1 {
					crate::ext::stringify_json(&selection[0]) + "\n"
				} else {
					serde_json::to_string_pretty(&selection)? + "\n"
				}
			},
		})
	}
}

async fn format_body(res: reqwest::Response) -> Result<String, PostWomanError> {
	match res.headers().get("Content-Type") {
		None => Ok(res.text().await? + "\n"),
		Some(v) => match v.to_str()? {
			"application/json" => Ok(serde_json::to_string_pretty(&res.json::<serde_json::Value>().await?)? + "\n"),
			"text/plain" | "text/html" => Ok(res.text().await? + "\n"),
			_ => Ok(format!("base64({})\n", BASE64_STANDARD.encode(res.bytes().await?))),
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
