#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PostWomanClient {
	pub user_agent: Option<String>,
}
