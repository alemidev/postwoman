use crate::ext::{FillError, FillableFromEnvironment, StringOr};


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
	/// headers to add to all requests from this client
	pub headers: Option<StringOr<toml::Table>>,
}

impl FillableFromEnvironment for ClientConfig {
	fn fill(mut self, env: &toml::Table) -> Result<Self, FillError> {
		let vars = Self::default_vars(env);

		if let Some(base) = self.base {
			self.base = Some(Self::replace(base, &vars)?);
		}

		if let Some(user_agent) = self.user_agent {
			self.user_agent = Some(Self::replace(user_agent, &vars)?);
		}

		if let Some(headers) = self.headers {
			self.headers = Some(Self::replace(headers, &vars)?);
		}

		Ok(self)
	}
}
