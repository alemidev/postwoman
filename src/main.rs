mod model;
mod errors;
mod ext;

use clap::{Parser, Subcommand};

pub use model::PostWomanCollection;
pub use errors::PostWomanError;

pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// API tester and debugger from your CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct PostWomanArgs {
	/// collection file to use
	#[arg(short, long, default_value = "postwoman.toml")]
	collection: String,

	/// action to run
	#[clap(subcommand)]
	action: Option<PostWomanActions>,

	/// start a multi-thread runtime, with multiple worker threads
	#[arg(long, default_value_t = false)]
	multi_threaded: bool,
}

#[derive(Subcommand, Debug)]
pub enum PostWomanActions {
	/// execute specific endpoint requests
	Run {
		/// regex query filter, run all with '.'
		query: String,

		/// run requests in parallel
		#[arg(long, default_value_t = false)]
		parallel: bool,

		/// repeat request N times
		#[arg(long, default_value_t = 1)]
		repeat: u32,

		/// force debug extractor on all routes
		#[arg(long, default_value_t = false)]
		debug: bool,
	},

	/// show all registered routes in current collection
	List {
		/// show verbose details for each route
		#[arg(short = 'V', long, default_value_t = false)]
		verbose: bool,
	},
}

const TIMESTAMP_FMT: &str = "%H:%M:%S%.6f"; 

fn main() -> Result<(), PostWomanError> {
	let args = PostWomanArgs::parse();

	let collection_raw = std::fs::read_to_string(&args.collection)?;
	let collection: PostWomanCollection = toml::from_str(&collection_raw)?;

	if args.multi_threaded {
		tokio::runtime::Builder::new_multi_thread()
			.enable_all()
			.build()
			.expect("failed creating tokio multi-thread runtime")
			.block_on(async { run_postwoman(args, collection).await })
	} else {
		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.expect("failed creating tokio current-thread runtime")
			.block_on(async { run_postwoman(args, collection).await })
	}
}

async fn run_postwoman(args: PostWomanArgs, collection: PostWomanCollection) -> Result<(), PostWomanError> {
	let action = args.action.unwrap_or(PostWomanActions::List { verbose: false });

	match action {
		PostWomanActions::List { verbose } => {
			let ua = collection.client.user_agent.unwrap_or(APP_USER_AGENT.to_string());
			println!("> {ua}");

			for (key, value) in collection.env {
				println!("+ {key}: {}", ext::stringify_toml(&value));
			}

			println!();

			for (name, mut endpoint) in collection.route {
				println!("- {name}: \t{} \t{}", endpoint.method.as_deref().unwrap_or("GET"), endpoint.url);
				if verbose {
					if let Some(ref query) = endpoint.query {
						for query in query {
							println!(" |? {query}");
						}
					}
					if let Some(ref headers) = endpoint.headers {
						for header in headers {
							println!(" |: {header}");
						}
					}
					if let Ok(body) = endpoint.body() {
						println!(" |> {body}");
					}
				}
			}
		},
		PostWomanActions::Run { query, parallel, repeat, debug  } => {
			let pattern = regex::Regex::new(&query)?;
			let mut joinset = tokio::task::JoinSet::new();
			let client = std::sync::Arc::new(collection.client);
			let env = std::sync::Arc::new(collection.env);
			for (name, mut endpoint) in collection.route {
				if pattern.find(&name).is_some() {
					if debug { endpoint.extract = Some(ext::StringOr::T(model::ExtractorConfig::Debug)) };
					for i in 0..repeat {
						let suffix = if repeat > 1 {
							format!("#{} ", i+1)
						} else {
							"".to_string()
						};
						let _client = client.clone();
						let _env = env.clone();
						let _endpoint = endpoint.clone();
						let _name = name.clone();
						let task = async move {
							let before = chrono::Local::now();
							eprintln!(" : [{}] sending {_name} {suffix}...", before.format(TIMESTAMP_FMT));
							let res = _endpoint
								.fill(&_env)
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
	}

	Ok(())
}

fn print_results(res: String, name: String, before: chrono::DateTime<chrono::Local>, suffix: String) {
	let after = chrono::Local::now();
	let elapsed = (after - before).num_milliseconds();
	let timestamp = after.format(TIMESTAMP_FMT);
	eprintln!(" + [{timestamp}] {name} {suffix}done in {elapsed}ms", );
	print!("{}", res);
}
