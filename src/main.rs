pub mod model;
mod requestor;
mod printer;

use clap::{Parser, Subcommand};

use regex::Regex;

use crate::{model::PostWomanCollection, requestor::send_requests , printer::{show_results, show_requests}};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// API tester and debugger from your CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct PostWomanArgs {
	/// collection to use
	#[arg(short, long, default_value = "postwoman.json")]
	collection: String,

	/// Action to run
	#[clap(subcommand)]
	action: PostWomanActions,
}

#[derive(Subcommand, Debug)]
pub enum PostWomanActions {
	/// run a single request to given url
	// Send {
	// 	/// request URL
	// 	url: String,

	// 	/// request method
	// 	#[arg(short = 'X', long, default_value_t = Method::GET)]
	// 	method: Method,

	// 	/// headers for request
	// 	#[arg(short = 'H', long, num_args = 0..)]
	// 	headers: Vec<String>,

	// 	/// request body
	// 	#[arg(short, long, default_value = "")]
	// 	data: String,

	// 	/// add action to collection items
	// 	#[arg(short = 'S', long, default_value_t = false)]
	// 	save: bool,
	// },
	/// run all saved requests
	Test {
		/// filter requests to fire by url (regex)
		filter: Option<String>,

		/// isolate each request client from others
		#[arg(long, default_value_t = false)]
		isolated: bool,

		/// pretty-print json outputs
		#[arg(short, long, default_value_t = false)]
		pretty: bool,

		/// show response body of each request
		#[arg(short, long, default_value_t = false)]
		verbose: bool,

		/// don't make any real request
		#[arg(long, default_value_t = false)]
		dry_run: bool,
	},
	/// list saved requests
	Show {
		/// filter requests to display by url (regex)
		filter: Option<String>,

		/// pretty-print json outputs
		#[arg(short, long, default_value_t = false)]
		pretty: bool,

		/// show response body of each request
		#[arg(short, long, default_value_t = false)]
		verbose: bool,
	},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = PostWomanArgs::parse();

	let collection = PostWomanCollection::from_path(&args.collection)?;

	match args.action {
		// PostWomanActions::Send {
		// 	url, headers, method, data, save
		// } => {
		// 	let req = Request::Object {
		// 		url: crate::proto::Url::String(url),
		// 		method: method.to_string(),
		// 		header: Some(
		// 			headers
		// 				.chunks(2)
		// 				.map(|x| Header {
		// 					key: x[0].clone(),
		// 					value: x[1].clone(), // TODO panics
		// 				})
		// 				.collect(),
		// 		),
		// 		body: if data.len() > 0 { Some(Body::String(data)) } else { None },
		// 		description: None,
		// 	};

		// 	let res = req.clone().send().await?;

		// 	if args.verbose {
		// 		println!(" ├┐ {}", res.status());
		// 	}

		// 	if args.verbose {
		// 		println!(" ││  {}", res.text().await?.replace("\n", "\n ││  "));
		// 	} else {
		// 		println!("{}", res.text().await?);
		// 	}

		// 	if save {
		// 		// TODO prompt for name and descr
		// 		let item = Item {
		// 			name: "TODO!".into(),
		// 			event: None,
		// 			item: None,
		// 			request: Some(req),
		// 			response: Some(vec![]),
		// 		};
		// 		collection.item.push(item);
		// 		std::fs::write(&args.collection, serde_json::to_string(&collection)?)?;
		// 		if args.verbose { println!(" ││ * saved") }
		// 	}

		// 	if args.verbose { println!(" │╵") }
		// },
		PostWomanActions::Test { filter, isolated, pretty, verbose, dry_run } => {
			let matcher = match filter {
				Some(rex) => Some(Regex::new(&rex)?),
				None => None,
			};

			let client = if isolated { None } else {
				Some(
					reqwest::Client::builder()
						.user_agent(APP_USER_AGENT)
						.build()
						.unwrap()
				)
			};

			match collection.requests(matcher.as_ref()) {
				Some(tree) => {
					let results = send_requests(tree, client, dry_run).await;
					show_results(results, verbose, pretty).await;
				},
				None => {
					eprintln!("[!] no requests match given filter");
				}
			}

		},
		PostWomanActions::Show { filter, verbose, pretty } => {
			let matcher = match filter {
				Some(rex) => Some(Regex::new(&rex)?),
				None => None,
			};
			match collection.requests(matcher.as_ref()) {
				Some(tree) => {
					show_requests(tree, verbose, pretty);
				},
				None => {
					eprintln!("[!] no requests match given filter");
				}
			}
		},
	}
	Ok(())
}
