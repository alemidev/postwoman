mod proto;
mod model;

use clap::{Parser, Subcommand};
use proto::Body;
use reqwest::Method;

use crate::proto::{Item, Request, Header};

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
	Send {
		/// request URL
		url: String,

		/// request method
		#[arg(short = 'X', long, default_value_t = Method::GET)]
		method: Method,

		/// headers for request
		#[arg(short = 'H', long, num_args = 0..)]
		headers: Vec<String>,

		/// request body
		#[arg(short, long, default_value = "")]
		data: String,

		/// show request that is being sent
		#[arg(long, default_value_t = false)]
		debug: bool,

		/// add action to collection items
		#[arg(short = 'S', long, default_value_t = false)]
		save: bool,
	},
	/// run all saved requests
	Test {},
	/// list saved requests
	Show {},
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = PostWomanArgs::parse();

	let mut collection : proto::PostWomanCollection = {
		let file = std::fs::File::open(&args.collection)?;
		serde_json::from_reader(file)?
	};

	println!("╶┐ * {}", collection.info.name);
	if let Some(descr) = collection.info.description {
		println!(" │   {}", descr);
	}
	println!(" │");

	match args.action {
		PostWomanActions::Send {
			url, headers, method, data, save, debug
		} => {
			let item = Item {
				name: "TODO!".into(),
				event: None,
				item: None,
				request: Some(Request {
					url: crate::proto::Url::String(url),
					method: method.to_string(),
					header: Some(
						headers
							.chunks(2)
							.map(|x| Header {
								key: x[0].clone(),
								value: x[1].clone(), // TODO panics
							})
							.collect(),
					),
					body: if data.len() > 0 { Some(Body::Text(data)) } else { None },
					description: None,
				}),
				response: Some(vec![]),
			};

			if debug {
				println!(" ├ {:?}", item);
			}

			let res = item.send().await?;
			println!(" ├┐ {}", res.status());

			if args.verbose {
				println!(" ││  {}", res.text().await?.replace("\n", "\n ││  "));
			}

			if save {
				// TODO prompt for name and descr
				collection.item.push(item);
				std::fs::write(&args.collection, serde_json::to_string(&collection)?)?;
				println!(" ││ * saved");
			}

			println!(" │╵");
		},
		PostWomanActions::Test { } => {
			let mut tasks = Vec::new();

			for item in collection.item {
				let t = tokio::spawn(async move {
					let r = item.send().await?;
					println!(" ├ {} >> {}", item.name, r.status());
					if args.verbose {
						println!(" │  {}", r.text().await?.replace("\n", "\n │  "));
					}
					Ok::<(), reqwest::Error>(())
				});
				tasks.push(t);
			}

			for t in tasks {
				t.await??;
			}
		},
		PostWomanActions::Show {  } => {
			println!(" ├ {:?}", collection);
		},
	}

	println!(" ╵");
	Ok(())
}
