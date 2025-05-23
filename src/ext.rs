use std::{collections::HashMap, sync::OnceLock};

pub fn sid() -> &'static str {
	static SID: std::sync::OnceLock<String> = std::sync::OnceLock::new();
	SID.get_or_init(|| uuid::Uuid::new_v4().to_string())
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

	fn replace<T: ReplaceableValue>(from: T, env: &HashMap<String, String>) -> Result<T, FillError> {
		from.replace_in_self(env)
	}

	fn default_vars(env: &toml::Table) -> std::collections::HashMap<String, String> {
		let mut vars: std::collections::HashMap<String, String> = std::collections::HashMap::default();

		vars.insert("POSTWOMAN_TIMESTAMP".to_string(), chrono::Local::now().timestamp().to_string());
		vars.insert("POSTWOMAN_TODAY".to_string(), chrono::Local::now().format("%d/%m/%Y").to_string());
		vars.insert("POSTWOMAN_LOCAL_ID".to_string(), uuid::Uuid::new_v4().to_string());
		vars.insert("POSTWOMAN_SESSION_ID".to_string(), sid().to_string());

		for (k, v) in env {
			vars.insert(k.to_string(), stringify_toml(v));
		}

		for (k, v) in std::env::vars() {
			vars.insert(k, v);
		}

		vars
	}
}

pub trait ReplaceableValue: Sized {
	fn replace_in_self(self, env: &HashMap<String, String>) -> Result<Self, FillError>;
}

impl ReplaceableValue for String {
	fn replace_in_self(mut self, env: &HashMap<String, String>) -> Result<Self, FillError> {
		let placeholders: Vec<(String, String)> = var_matcher()
			.captures_iter(&self)
			.map(|m| m.extract())
			.map(|(txt, [var])| (txt.to_string(), var.to_string()))
			.collect();

		// TODO can we avoid cloning all matches??? can't mutate `from` as captures_iter holds an
		// immutable reference to original string

		for (txt, var) in placeholders {
			let value = env.get(&var).ok_or(FillError(var.to_string()))?;
			self = self.replace(&txt, value);
		}

		Ok(self)
	}
}

impl ReplaceableValue for toml::Table {
	fn replace_in_self(self, env: &HashMap<String, String>) -> Result<Self, FillError> {
		let mut new_map = toml::Table::default();
		for (k, v) in self {
			new_map.insert(
				k.replace_in_self(env)?,
				v.replace_in_self(env)?,
			);
		}

		Ok(new_map)
	}
}

impl ReplaceableValue for toml::Value {
	fn replace_in_self(self, env: &HashMap<String, String>) -> Result<Self, FillError> {
		match self {
			toml::Value::String(x) => Ok(toml::Value::String(x.replace_in_self(env)?)),
			toml::Value::Integer(x) => Ok(toml::Value::Integer(x)),
			toml::Value::Float(x) => Ok(toml::Value::Float(x)),
			toml::Value::Boolean(x) => Ok(toml::Value::Boolean(x)),
			toml::Value::Datetime(x) => Ok(toml::Value::Datetime(x)),
			toml::Value::Array(arr) => Ok(toml::Value::Array(arr.replace_in_self(env)?)),
			toml::Value::Table(map) => Ok(toml::Value::Table(map.replace_in_self(env)?)),
		}
	}
}

impl<T: ReplaceableValue> ReplaceableValue for Vec<T> {
	fn replace_in_self(self, env: &HashMap<String, String>) -> Result<Self, FillError> {
		let mut new_arr = Vec::new();
		for v in self {
			new_arr.push(v.replace_in_self(env)?);
		}
		Ok(new_arr)
	}
}

impl<T: ReplaceableValue> ReplaceableValue for StringOr<T> {
	fn replace_in_self(self, env: &HashMap<String, String>) -> Result<Self, FillError> {
		match self {
			StringOr::Str(x) => Ok(StringOr::Str(x.replace_in_self(env)?)),
			StringOr::T(x) => Ok(StringOr::T(x.replace_in_self(env)?)),
		}
	}
}
