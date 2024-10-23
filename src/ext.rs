use std::sync::OnceLock;

use crate::PostWomanError;

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

pub fn stringify_toml(v: &toml::Value) -> String {
	match v {
		toml::Value::Boolean(x) => x.to_string(),
		toml::Value::Integer(x) => x.to_string(),
		toml::Value::Float(x) => x.to_string(),
		toml::Value::String(x) => x.clone(),
		toml::Value::Datetime(x) => x.to_string(),
		toml::Value::Array(x) => serde_json::to_string(&x).unwrap_or_default(),
		toml::Value::Table(x) => serde_json::to_string(&x).unwrap_or_default(),
	}
}

pub fn stringify_json(v: &serde_json::Value) -> String {
	match v {
		serde_json::Value::Null => "null".to_string(),
		serde_json::Value::Bool(x) => x.to_string(),
		serde_json::Value::Number(x) => x.to_string(),
		serde_json::Value::String(x) => x.clone(),
		serde_json::Value::Array(x) => serde_json::to_string(&x).unwrap_or_default(),
		serde_json::Value::Object(x) => serde_json::to_string(&x).unwrap_or_default(),
	}
}

pub fn var_matcher() -> &'static regex::Regex {
	static MATCHER : OnceLock<regex::Regex> = OnceLock::new();
	MATCHER.get_or_init(|| regex::Regex::new(r"\$\{(\w+)\}").expect("wrong matcher regex"))
}

// keep it as separate fn so we can change it everywhere easily
pub fn full_name(namespace: &str, name: &str) -> String {
	format!("{namespace}:{name}")
}

#[derive(Debug, thiserror::Error)]
#[error("could not fill {0}")]
pub struct FillError(pub String);

pub trait FillableFromEnvironment: Sized {
	fn fill(self, env: &toml::Table) -> Result<Self, FillError>;

	fn replace(mut from: String, env: &toml::Table) -> Result<String, FillError> {
		let placeholders: Vec<(String, String)> = var_matcher()
			.captures_iter(&from)
			.map(|m| m.extract())
			.map(|(txt, [var])| (txt.to_string(), var.to_string()))
			.collect();

		// TODO can we avoid cloning all matches??? can't mutate `from` as captures_iter holds an
		// immutable reference to original string

		for (txt, var) in placeholders {
			let value = env.get(&var).ok_or(FillError(var.to_string()))?;
			from = from.replace(&txt, &stringify_toml(value));
		}

		Ok(from)
	}

	fn default_vars(env: &toml::Table) -> std::collections::HashMap<String, String> {
		let mut vars: std::collections::HashMap<String, String> = std::collections::HashMap::default();

		vars.insert("POSTWOMAN_TIMESTAMP".to_string(), chrono::Local::now().timestamp().to_string());

		for (k, v) in env {
			vars.insert(k.to_string(), stringify_toml(v));
		}

		for (k, v) in std::env::vars() {
			vars.insert(k, v);
		}

		vars
	}
}
