
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Extractor {
	#[default]
	Debug,
	Body,
	Discard,
	JQ { query: String },
	Regex { pattern: String },
	Header { key: String },
}
