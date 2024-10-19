mod model;

use clap::{Parser, Subcommand};
use model::{PostWomanConfig, PostWomanError};

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
						.execute(&config.client)
						.await?;
					println!("{res}");
				}
			}
		},

		// PostWomanActions::Save { name, url, method, headers, body } => {
		// 	todo!();
		// },
	}

	Ok(())
}
