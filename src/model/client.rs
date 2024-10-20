
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientConfig {
	/// base url for composing endpoints
	pub base: Option<String>,
	/// user agent for requests, defaults to 'postwoman/<version>'
	pub user_agent: Option<String>,
	/// max total duration of each request, in seconds. defaults to 30
	pub timeout: Option<u64>,
	/// max number of redirects to allow, defaults to 0
	pub redirects: Option<usize>,
	/// accept invalid SSL certificates, defaults to false (be careful: this is dangerous!)
	pub accept_invalid_certs: Option<bool>,
}
