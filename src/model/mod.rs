mod client;
mod endpoint;
mod extractor;

pub use client::ClientConfig;
pub use endpoint::EndpointConfig;
pub use extractor::ExtractorConfig;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PostWomanCollection {
	#[serde(default)]
	pub client: ClientConfig,
	#[serde(default)]
	pub include: Vec<String>,
	#[serde(default)]
	pub env: toml::Table,
	#[serde(default)]
	pub route: indexmap::IndexMap<String, EndpointConfig>,
	// it's weird to name it singular but makes more sense in config
}
