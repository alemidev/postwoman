
#[derive(Debug, thiserror::Error)]
pub enum PostWomanError {
	#[error("network error: {0:?}")]
	Request(#[from] reqwest::Error),

	#[error("invalid method: {0:?}")]
	InvalidMethod(#[from] http::method::InvalidMethod),

	#[error("invalid header: {0:?}")]
	InvalidHeader(#[from] InvalidHeaderError),

	#[error("contains Unprintable characters: {0:?}")]
	Unprintable(#[from] reqwest::header::ToStrError),

	#[error("header '{0}' not found in response")]
	HeaderNotFound(String),

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

	#[error("request didn't match expected status code: {0:?}")]
	UnexpectedStatusCode(reqwest::Response),

	#[error("request didn't match expected result: got '{0}' expected '{1}'")]
	UnexpectedResult(String, String),

	#[error("invalid Json Query: {0}")]
	JQError(String),

	#[error("regex failed matching in content: {0}")]
	NoMatch(String),
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidHeaderError {
	#[error("invalid header name: {0:?}")]
	Name(#[from] http::header::InvalidHeaderName),
	#[error("invalid header value: {0:?}")]
	Value(#[from] http::header::InvalidHeaderValue),
	#[error("invalid header format: {0}")]
	Format(String)
}
