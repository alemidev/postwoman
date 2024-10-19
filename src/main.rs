mod model;

use std::collections::HashMap;

use clap::{Parser, Subcommand};
use model::{Endpoint, Extractor, PostWomanClient, PostWomanConfig, PostWomanError, StringOr};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// API tester and debugger from your CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct PostWomanArgs {
	/// collection file to use
	#[arg(short, long, default_value = "postwoman.toml")]
	collection: String,

	/// action to run
	#[clap(subcommand)]
	action: PostWomanActions,
}

#[derive(Subcommand, Debug)]
pub enum PostWomanActions {
	/// print an example configragion, pipe to file and start editing
	Sample,

	/// execute specific endpoint requests
	Run {
		/// regex query filter, run all with '.*'
		query: String,
	},

	// Save {
	// 	/// name for new endpoint
	// 	name: String,
	// 	/// url of endpoint
	// 	url: String,
	// 	/// method
	// 	method: Option<String>,
	// 	/// headers
	// 	headers: Vec<String>,
	// 	/// body
	// 	body: Option<String>,
	// }
}

#[tokio::main]
async fn main() -> Result<(), PostWomanError> {
	let args = PostWomanArgs::parse();

	if matches!(args.action, PostWomanActions::Sample) {
		let a = Endpoint {
			url: "https://api.alemi.dev/debug".into(),
			query: None,
			method: None,
			headers: None,
			body: None,
			extract: None,
		};

		let b = Endpoint {
			url: "https://api.alemi.dev/debug".into(),
			query: None,
			method: Some("PUT".into()),
			headers: Some(vec![
				"Authorization: Bearer asdfg".into(),
				"Cache: skip".into(),
			]),
			body: Some(StringOr::T(toml::Table::from_iter([("hello".into(), toml::Value::String("world".into()))]))),
			extract: Some(StringOr::T(Extractor::Body)),
		};

		let client = PostWomanClient {
			user_agent: Some(APP_USER_AGENT.into()),
		};

		let cfg = PostWomanConfig {
			client,
			route: HashMap::from_iter([
				("simple".to_string(), a),
				("json".to_string(), b),
			]),
		};

		println!("{}", toml_edit::ser::to_string_pretty(&cfg)?);

		return Ok(());
	}

	let collection = std::fs::read_to_string(args.collection)?;
	let config: PostWomanConfig = toml::from_str(&collection)?;

	match args.action {
		PostWomanActions::Run { query } => {
			let pattern = regex::Regex::new(&query)?;
			for (name, endpoint) in config.route {
				if pattern.find(&name).is_some() {
					eprintln!("> executing {name}");
					let res = endpoint
						.fill()
						.execute()
						.await?;
					println!("{res}");
				}
			}
		},

		// PostWomanActions::Save { name, url, method, headers, body } => {
		// 	todo!();
		// },

		PostWomanActions::Sample => unreachable!(),
	}

	Ok(())
}
