use crate::ext::FillableFromEnvironment;


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

impl FillableFromEnvironment for ClientConfig {
	fn fill(mut self, env: &toml::Table) -> Self {
		let vars = Self::default_vars(env);

		for (k, v) in vars {
			let k_var = format!("${{{k}}}");

			if let Some(base) = self.base {
				self.base = Some(base.replace(&k_var, &v));
			}

			if let Some(user_agent) = self.user_agent {
				self.user_agent = Some(user_agent.replace(&k_var, &v));
			}
		}

		self
	}
}
