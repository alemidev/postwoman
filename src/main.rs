mod model;

use std::sync::Arc;

use clap::{Parser, Subcommand};

use regex::Regex;

use crate::model::PostWomanCollection;

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

	/// show response body of each request
	#[arg(short, long, default_value_t = false)]
	verbose: bool,
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
	},
	/// list saved requests
	Show {},
}

enum TestResult {
	Success {
		url: String,
		method: String,
		response: reqwest::Response,
	},
	Failure {
		url: String,
		method: String,
		err: reqwest::Error,
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = PostWomanArgs::parse();

	let collection = PostWomanCollection::from_path(&args.collection)?;

	println!("╶┐ * {}", collection.name());

	if args.verbose {
		if let Some(descr) = &collection.description() {
			println!(" │   {}", descr);
		}
		// if let Some(version) = &collection.version() {
		// 	println!(" │   {}", version);
		// }
		println!(" │");
	}

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
		PostWomanActions::Test { filter, isolated, pretty } => {
			let reqs = collection.requests();

			let matcher = match filter {
				Some(rex) => Some(Regex::new(&rex)?),
				None => None,
			};

			let mut tasks = Vec::new();

			let client = Arc::new(reqwest::Client::default());

			for req in reqs {
				let url = req.url().as_str().to_string();
				if let Some(m) = &matcher {
					if !m.is_match(&url) {
						continue
					}
				}
				let _c = client.clone();
				let t = tokio::spawn(async move {
					let c = if isolated {
						Arc::new(reqwest::Client::default())
					} else { _c };
					let method = req.method().as_str().to_string();
					// if args.verbose {
					// 	println!(" ├─  {:?}", req);
					// 	if let Some(body) = req.body() {
					// 		println!(" ├──  {}", std::str::from_utf8(body.as_bytes().unwrap()).unwrap());
					// 	}
					// }
					let response = c.execute(req).await;
					match response {
						Ok(response) => TestResult::Success { url, method, response },
						Err(err) => TestResult::Failure { url, method, err }
					}
				});
				tasks.push(t);
			}

			for t in tasks {
				match t.await? {
					TestResult::Success { url, method, response } => {
						let status_code = response.status().as_u16();
						let marker = if status_code < 400 { '✓' } else { '×' };
						println!(" ├ {} {} >> {} {}", marker, status_code, method, url);
						if args.verbose {
							let mut body = response.text().await?;
							if pretty {
								if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
									if let Ok(t) = serde_json::to_string_pretty(&v) {
										body = t;
									}
								}
							}
							println!(" │  {}", body.replace("\n", "\n │  "));
							println!(" │");
						}
					},
					TestResult::Failure { url, method, err } => {
						println!(" ├ ! ERROR >> {} {}", method, url);
						if args.verbose {
							println!(" │  {}", err);
							println!(" │");
						}
					}
				}
			}
		},
		PostWomanActions::Show {  } => {
			println!(" ├ {:?}", collection); // TODO nicer print
		},
	}

	println!(" ╵");
	Ok(())
}
