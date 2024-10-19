mod model;

use std::sync::Arc;

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

		/// run requests in parallel
		#[arg(long, default_value_t = false)]
		parallel: bool,

		/// repeat request N times
		#[arg(long, default_value_t = 1)]
		repeat: u32,
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

const TIMESTAMP_FMT: &str = "%H:%M:%S%.6f"; 

fn print_results(res: String, name: String, before: chrono::DateTime<chrono::Local>, suffix: String) {
	let after = chrono::Local::now();
	let elapsed = (after - before).num_milliseconds();
	let timestamp = after.format(TIMESTAMP_FMT);
	eprintln!(" + [{timestamp}] {name} {suffix}done in {elapsed}ms", );
	print!("{}", res);
}

#[tokio::main]
async fn main() -> Result<(), PostWomanError> {
	let args = PostWomanArgs::parse();

	let collection = std::fs::read_to_string(args.collection)?;
	let config: PostWomanConfig = toml::from_str(&collection)?;

	match args.action {
		PostWomanActions::Run { query, parallel, repeat } => {
			let pattern = regex::Regex::new(&query)?;
			let mut joinset = tokio::task::JoinSet::new();
			let client = Arc::new(config.client);
			for (name, endpoint) in config.route {
				if pattern.find(&name).is_some() {
					for i in 0..repeat {
						let suffix = if repeat > 1 {
							format!("#{} ", i+1)
						} else {
							"".to_string()
						};
						let _client = client.clone();
						let _endpoint = endpoint.clone();
						let _name = name.clone();
						let task = async move {
							let before = chrono::Local::now();
							eprintln!(" : [{}] sending {_name} {suffix}...", before.format(TIMESTAMP_FMT));
							let res = _endpoint
								.fill()
								.execute(&_client)
								.await;
							(res, _name, before, suffix)
						};
						if parallel {
							joinset.spawn(task);
						} else {
							let (res, name, before, num) = task.await;
							print_results(res?, name, before, num);
						}
					}
				}
			}
			while let Some(j) = joinset.join_next().await {
				match j {
					Ok((res, name, before, num)) => print_results(res?, name, before, num),
					Err(e) => eprintln!("! error joining task: {e}"),
				}
			}
		},

		// PostWomanActions::Save { name, url, method, headers, body } => {
		// 	todo!();
		// },
	}

	Ok(())
}
