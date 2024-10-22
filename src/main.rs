mod model;
mod errors;
mod ext;
mod fmt;

use std::str::FromStr;

use clap::{Parser, Subcommand};

use ext::FillableFromEnvironment;
use fmt::{PrintableResult, ReportableResult};
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
	#[arg(short = 'M', long, default_value_t = false)]
	multi_thread: bool,

	/// emit json report document instead of pretty printing
	#[arg(short = 'R', long, default_value_t = false)]
	report: bool,
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

		/// print matched routes but don't perform requests
		#[arg(long, default_value_t = false)]
		dry_run: bool,
	},

	/// show all registered routes in current collection
	List {
		/// show only limited details for each route
		#[arg(short, long, default_value_t = false)]
		compact: bool,
	},
}

const DEFAULT_ACTION: PostWomanActions = PostWomanActions::List { compact: true };

fn main() {
	let args = PostWomanArgs::parse();
	let multi_thread = args.multi_thread;

	// if we got a regex, test it early to avoid wasting work when invalid
	if let Some(PostWomanActions::Run { ref query, .. }) = args.action {
		// note that if you remove this test, there's another .expect() below you need to manage too!
		if let Err(e) = regex::Regex::new(query) {
			return eprintln!("! invalid regex filter: {e}");
		}
	}

	let mut collections = IndexMap::new();

	if !load_collections(&mut collections, args.collection.clone(), &toml::Table::default()) {
		return;
	}

	match args.action.as_ref().unwrap_or(&DEFAULT_ACTION) {
		PostWomanActions::List { compact } => {
			if args.report {
				(collections, *compact).report();
			} else {
				(collections, *compact).print();
			}
		},

		PostWomanActions::Run { query, parallel, debug, dry_run } => {
			let task = async move {
				let mut pool = tokio::task::JoinSet::new();

				for (collection_name, collection) in collections {
					run_collection_endpoints(
						collection_name,
						collection,
						query.clone(),
						*parallel,
						*debug,
						*dry_run,
						args.report,
						&mut pool
					).await;
				}

				while let Some(j) = pool.join_next().await {
					if let Err(e) = j {
						eprintln!("! error joining task: {e}");
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
		},
	}
}

// TODO too many arguments
async fn run_collection_endpoints(
	namespace: String,
	collection: PostWomanCollection,
	query: String,
	parallel: bool,
	debug: bool,
	dry_run: bool,
	report: bool,
	pool: &mut tokio::task::JoinSet<()>
) {
	// this is always safe to compile because we tested it beforehand
	let pattern = regex::Regex::new(&query).expect("tested it before and still failed here???");
	let env = std::sync::Arc::new(collection.env);
	let client = std::sync::Arc::new(collection.client.fill(&env));

	for (name, mut endpoint) in collection.route {
		let full_name = ext::full_name(&namespace, &name);
		if pattern.find(&full_name).is_none() { continue };

		if debug { endpoint.extract = Some(ext::StringOr::T(model::ExtractorConfig::Debug)) };
		let _client = client.clone();
		let _env = env.clone();
		let _namespace = namespace.clone();

		let task = async move {
			let before = chrono::Local::now();
			eprintln!(" : [{}] {full_name} \tsending request...", before.format(fmt::TIMESTAMP_FMT));

			let res = if dry_run {
				Ok("".to_string())
			} else {
				endpoint
					.fill(&_env)
					.execute(&_client)
					.await
			};

			let after = chrono::Local::now();
			let elapsed = (after - before).num_milliseconds();

			let timestamp = after.format(fmt::TIMESTAMP_FMT);
			let symbol = if res.is_ok() { " + " } else { "<!>" };
			let verb = if res.is_ok() { "done in" } else { "failed after" };
			eprintln!("{symbol}[{timestamp}] {_namespace}::{name} \t{verb} {elapsed}ms", );

			if report {
				(res, _namespace, name, elapsed).report();
			} else {
				(res, _namespace, name, elapsed).print();
			}
		};

		if parallel {
			pool.spawn(task);
		} else {
			task.await;
		}
	}
}

fn load_collections(store: &mut IndexMap<String, PostWomanCollection>, mut path: std::path::PathBuf, parent_env: &toml::Table) -> bool {
	let collection_raw = match std::fs::read_to_string(&path) {
		Ok(x) => x,
		Err(e) => {
			eprintln!("! error loading collection {path:?}: {e}");
			return false;
		},
	};

	let mut collection: PostWomanCollection = match toml::from_str(&collection_raw) {
		Ok(x) => x,
		Err(e) => {
			eprintln!("! error parsing collection {path:?}: {e}");
			return false;
		},
	};

	collection.env.extend(parent_env.iter().map(|(k, v)| (k.clone(), v.clone())));

	let name = path.to_string_lossy().replace(".toml", "");
	let mut to_include = Vec::new();

	path.pop();
	for include in &collection.include {
		let mut base = path.clone();
		let new = std::path::PathBuf::from_str(include).expect("infallible");
		base.push(new);
		to_include.push(base);
	}

	let parent_env = collection.env.clone();
	store.insert(name, collection);

	for base in to_include {
		if !load_collections(store, base, &parent_env) {
			return false;
		}
	}

	true
}
