use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PostWomanCollection {
	pub variables: Vec<String>, // TODO these sure aren't just strings for sure...
	pub info: CollectionInfo,
	pub item: Vec<Item>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CollectionInfo {
	pub name: String,
	pub description: String,
	pub schema: String,

	pub _postman_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
	pub name: String,
	pub event: Option<Vec<Event>>,
	pub request: Request,
	pub response: Vec<String>, // TODO surely isn't just strings
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
	pub listen: String,
	pub script: Script,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Script {
	pub r#type: String,
	pub exec: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
	pub url: Url,
	pub method: String,
	pub header: Option<Vec<Header>>,
	pub body: Option<String>,
	pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Header {
	pub key: String,
	pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Body {
	Json(serde_json::Value),
	Text(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Url {
	Object {
		raw: Option<String>,
		protocol: String,
		host: Vec<String>,
		path: Vec<String>,
		query: Option<Vec<Query>>,
		variable: Option<Vec<String>>, // TODO surely aren't just strings
	},
	String(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Query {
	pub key: String,
	pub value: String,
	pub equals: bool,
	pub description: Option<String>,
}

impl ToString for Body {
	fn to_string(&self) -> String {
		match self {
			Body::Json(v) => serde_json::to_string(v).unwrap(),
			Body::Text(s) => s.clone(),
		}
	}
}

impl ToString for Query {
	fn to_string(&self) -> String {
		format!("{}={}", self.key, self.value)
	}
}

impl ToString for Url {
	fn to_string(&self) -> String {
		match self {
			Url::String(s) => s.clone(),
			Url::Object {
				raw, protocol,
				host,path, query,
				variable: _
			} => {
				match &raw {
					Some(s) => s.clone(),
					None => {
						let mut url = String::new();
						url.push_str(&protocol);
						url.push_str("://");
						url.push_str(&host.join("."));
						url.push_str("/");
						url.push_str(&path.join("/"));

						if let Some(query) = &query {
							url.push_str("?");
							let q : Vec<String> = query.iter().map(|x| x.to_string()).collect();
							url.push_str(&q.join("&"));
						}

						url
					}
				}
			}
		}
	}
}
