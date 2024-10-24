
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ExtractorConfig {
	#[default]
	Body,
	Debug,
	Discard,
	JQ { query: String },
	Regex { pattern: String },
	Header { key: String },
}
