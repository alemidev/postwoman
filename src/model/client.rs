
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientConfig {
	/// user agent for requests, defaults to 'postwoman/<version>'
	pub user_agent: Option<String>,
}
