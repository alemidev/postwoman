
#[derive(Debug, thiserror::Error)]
pub enum PostWomanError {
	#[error("network error: {0:?}")]
	Request(#[from] reqwest::Error),

	#[error("invalid method: {0:?}")]
	InvalidMethod(#[from] http::method::InvalidMethod),

	#[error("invalid header name: {0:?}")]
	InvalidHeaderName(#[from] reqwest::header::InvalidHeaderName),

	#[error("invalid header value: {0:?}")]
	InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

	#[error("contains Unprintable characters: {0:?}")]
	Unprintable(#[from] reqwest::header::ToStrError),

	#[error("header '{0}' not found in response")]
	HeaderNotFound(String),

	#[error("invalid header: '{0}'")]
	InvalidHeader(String),

	#[error("error opening collection: {0:?}")]
	ErrorOpeningCollection(#[from] std::io::Error),

	#[error("collection is not valid toml: {0:?}")]
	InvalidCollection(#[from] toml::de::Error),

	#[error("could not represent collection: {0:?}")] // should never happen
	ErrorSerializingInternallyCollection(#[from] toml_edit::ser::Error),

	#[error("invalid json payload: {0:?}")]
	InvalidJson(#[from] serde_json::Error),

	#[error("invalid regex: {0:?}")]
	InvalidRegex(#[from] regex::Error),
}
