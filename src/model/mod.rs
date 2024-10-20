mod client;
mod endpoint;
mod extractor;

pub use client::ClientConfig;
pub use endpoint::EndpointConfig;
pub use extractor::ExtractorConfig;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PostWomanCollection {
	pub client: Option<ClientConfig>,
	pub env: Option<toml::Table>,
	pub include: Option<Vec<String>>,
	// it's weird to name it singular but makes more sense in config
	pub route: indexmap::IndexMap<String, EndpointConfig>,
}
