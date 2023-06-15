mod model;

use clap::{Parser, Subcommand};

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
	Test {},
	/// list saved requests
	Show {},
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = PostWomanArgs::parse();

	let collection = PostWomanCollection::from_path(&args.collection)?;

	if args.verbose {
		println!("╶┐ * {}", collection.name());
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
		PostWomanActions::Test { } => {
			let reqs = collection.requests();

			let mut tasks = Vec::new();

			for req in reqs {
				let t = tokio::spawn(async move {
					let c = reqwest::Client::default(); // TODO maybe make just 1 client for everyone?
					let url = req.url().as_str().to_string();
					let r = c.execute(req).await?;
					println!(" ├ {} >> {}", url, r.status());
					if args.verbose {
						println!(" │  {}", r.text().await?.replace("\n", "\n │  "));
					}
					Ok::<(), reqwest::Error>(())
				});
				tasks.push(t);
			}

			for t in tasks {
				match t.await? {
					Ok(_) => todo!(),
					Err(e) => eprintln!("{}", e),
				}
			}
		},
		PostWomanActions::Show {  } => {
			println!(" ├ {:?}", collection);
		},
	}

	println!(" ╵");
	Ok(())
}
