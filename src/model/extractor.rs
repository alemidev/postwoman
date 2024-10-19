
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Extractor {
	#[default]
	Debug,
	Body,
	Discard,
	// JQL { query: String },
	// Regex { pattern: String },
	Header { key: String },
}