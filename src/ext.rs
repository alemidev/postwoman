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

