mod model;
mod errors;
mod ext;

use std::{collections::HashMap, str::FromStr};

use clap::{Parser, Subcommand};

use indexmap::IndexMap;
pub use model::PostWomanCollection;
pub use errors::PostWomanError;

pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// API tester and debugger from your CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct PostWomanArgs {
	/// collection file to use
	#[arg(short, long, default_value = "postwoman.toml")]
	collection: std::path::PathBuf,

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
		#[arg(short, long, default_value_t = false)]
		parallel: bool,

		/// force debug extractor on all routes
		#[arg(long, default_value_t = false)]
		debug: bool,
	},

	/// show all registered routes in current collection
	List {
		/// show only limited details for each route
		#[arg(short, long, default_value_t = false)]
		compact: bool,
	},
}

const TIMESTAMP_FMT: &str = "%H:%M:%S%.6f"; 

fn main() {
	let args = PostWomanArgs::parse();
	let multi_thread = args.multi_threaded;

	// if we got a regex, test it early to avoid wasting work when invalid
	if let Some(PostWomanActions::Run { ref query, .. }) = args.action {
		// note that if you remove this test, there's another .expect() below you need to manage too!
		if let Err(e) = regex::Regex::new(query) {
			return eprintln!("! invalid regex filter: {e}");
		}
	}

	let mut collections = IndexMap::new();

	if !load_collections(&mut collections, args.collection.clone()) {
		return;
	}

	let task = async move {

		let mut pool = tokio::task::JoinSet::new();

		for (collection_name, collection) in collections {
			run_postwoman(&args, collection_name, collection, &mut pool).await;
		}

		while let Some(j) = pool.join_next().await {
			match j {
				Err(e) => eprintln!("! error joining task: {e}"),
				Ok(res) => res.print(),
			}
		}
	};

	eprintln!("~@ {APP_USER_AGENT}");
	if multi_thread {
		tokio::runtime::Builder::new_multi_thread()
			.enable_all()
			.build()
			.expect("failed creating tokio multi-thread runtime")
			.block_on(task)
	} else {
		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.expect("failed creating tokio current-thread runtime")
			.block_on(task)
	}
}

fn load_collections(store: &mut IndexMap<String, PostWomanCollection>, mut path: std::path::PathBuf) -> bool {
	let collection_raw = match std::fs::read_to_string(&path) {
		Ok(x) => x,
		Err(e) => {
			eprintln!("! error loading collection {path:?}: {e}");
			return false;
		},
	};

	let collection: PostWomanCollection = match toml::from_str(&collection_raw) {
		Ok(x) => x,
		Err(e) => {
			eprintln!("! error parsing collection {path:?}: {e}");
			return false;
		},
	};

	let name = path.to_string_lossy().to_string();
	let mut to_include = Vec::new();

	if let Some(ref includes) = collection.include {
		path.pop();
		for include in includes {
			let mut base = path.clone();
			let new = std::path::PathBuf::from_str(include).expect("infallible");
			base.push(new);
			to_include.push(base);
		}
	}

	store.insert(name, collection);

	for base in to_include {
		if !load_collections(store, base) {
			return false;
		}
	}

	true
}

const DEFAULT_ACTION: PostWomanActions = PostWomanActions::List { compact: true };
type RunResult = (Result<String, PostWomanError>, String, String, chrono::DateTime<chrono::Local>);

async fn run_postwoman(args: &PostWomanArgs, namespace: String, collection: PostWomanCollection, pool: &mut tokio::task::JoinSet<RunResult>) {
	let action = args.action.as_ref().unwrap_or(&DEFAULT_ACTION);

	match action {
		PostWomanActions::List { compact } => {
			println!("-> {namespace}");

			for (key, value) in collection.env.unwrap_or_default() {
				println!(" + {key}={}", ext::stringify_toml(&value));
			}

			for (name, mut endpoint) in collection.route {
				println!(" - {name} \t{} \t{}", endpoint.method.as_deref().unwrap_or("GET"), endpoint.url);
				if ! *compact {
					if let Some(ref query) = endpoint.query {
						for query in query {
							println!("   |? {query}");
						}
					}
					if let Some(ref headers) = endpoint.headers {
						for header in headers {
							println!("   |: {header}");
						}
					}
					if let Some(ref _x) = endpoint.body {
						if let Ok(body) = endpoint.body() {
							println!("   |> {}", body.replace("\n", "\n   |> "));
						} else {
							println!("   |> [!] invalid body");
						}
					}
				}
			}

			println!();
		},
		PostWomanActions::Run { query, parallel, debug  } => {
			// this is always safe to compile because we tested it beforehand
			let pattern = regex::Regex::new(query).expect("tested it before and still failed here???");
			let client = std::sync::Arc::new(collection.client.unwrap_or_default());
			let env = std::sync::Arc::new(collection.env.unwrap_or_default());
			for (name, mut endpoint) in collection.route {
				if pattern.find(&name).is_some() {
					if *debug { endpoint.extract = Some(ext::StringOr::T(model::ExtractorConfig::Debug)) };
					let _client = client.clone();
					let _env = env.clone();
					let _endpoint = endpoint.clone();
					let _name = name.clone();
					let _namespace = namespace.clone();
					let task = async move {
						let before = chrono::Local::now();
						eprintln!(" : [{}] {_namespace}::{_name} \tsending request...", before.format(TIMESTAMP_FMT));
						let res = _endpoint
							.fill(&_env)
							.execute(&_client)
							.await;
						(res, _namespace, _name, before)
					};
					if *parallel {
						pool.spawn(task);
					} else {
						task.await.print();
					}
				}
			}
		},
	}
}

trait PrintableResult {
	fn print(self);
}

impl PrintableResult for RunResult {
	fn print(self) {
		let (result, namespace, name, before) = self;
		let success = result.is_ok();
		let after = chrono::Local::now();
		let elapsed = (after - before).num_milliseconds();
		let timestamp = after.format(TIMESTAMP_FMT);
		let symbol = if success { " + " } else { "<!>" };
		let verb = if success { "done in" } else { "failed after" };
		eprintln!("{symbol}[{timestamp}] {namespace}::{name} \t{verb} {elapsed}ms", );
		match result {
			Ok(x) => print!("{x}"),
			Err(e) => eprintln!(" ! {e}"),
		}
	}
}
