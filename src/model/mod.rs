mod client;
mod endpoint;
mod extractor;

pub use client::PostWomanClient;
pub use endpoint::Endpoint;
pub use extractor::Extractor;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PostWomanConfig {
	pub client: PostWomanClient,
	pub env: toml::Table,
	// it's weird to name it singular but makes more sense in config
	pub route: indexmap::IndexMap<String, Endpoint>,
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
