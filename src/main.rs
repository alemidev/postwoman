mod model;

use clap::{Parser, Subcommand};
use reqwest::StatusCode;

/// API tester and debugger from your CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct PostWomanArgs {
	/// collection to use
	#[arg(short, long, default_value = "postwoman.json")]
	collection: String,

	/// Action to run
	#[clap(subcommand)]
	action: Option<PostWomanActions>,

	/// add action to collection items
	#[arg(short = 'S', long, default_value_t = false)]
	save: bool,

	/// user agent for requests
	#[arg(long, default_value = "postwoman")]
	agent: String,
}

#[derive(Subcommand, Debug)]
enum PostWomanActions {
	/// run a single GET request
	Get {
		/// request URL
		url: String,

		/// headers for request
		#[arg(short = 'H', long, num_args = 0..)]
		headers: Vec<String>,
	},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = PostWomanArgs::parse();

	let file = std::fs::File::open(&args.collection)?;
	let collection : model::PostWomanCollection = serde_json::from_reader(file)?;

	println!("╶┐ * {}", collection.info.name);
	println!(" │   {}", collection.info.description);
	println!(" │");

	if let Some(action) = args.action {

		match action {
			PostWomanActions::Get { url, headers } => {
				if headers.len() % 2 != 0 {
					return Err(PostWomanError::throw("headers must come in pairs"));
				}

				let mut req = reqwest::Client::new()
					.get(url);

				for h in headers.chunks(2) {
					let (k, v) = (&h[0], &h[1]);
					req = req.header(k, v);
				}

				let res = req.send().await?;

				println!("{}", res.text().await?);
			}
		}

	} else {

		let mut tasks = Vec::new();

		for item in collection.item {
			let t = tokio::spawn(async move {
				let r = item.exec().await?;
				println!(" ├ {} >> {}", item.name, r);
				Ok::<(), reqwest::Error>(())
			});
			tasks.push(t);
		}

		for t in tasks {
			t.await??;
		}

	}

	println!(" ╵");
	Ok(())
}


impl model::Item {
	async fn exec(&self) -> reqwest::Result<StatusCode> {
		let method = reqwest::Method::from_bytes(
			self.request.method.as_bytes()
		).unwrap_or(reqwest::Method::GET); // TODO throw an error rather than replacing it silently

		let mut req = reqwest::Client::new()
			.request(method, self.request.url.to_string());

		if let Some(headers) = &self.request.header {
			for h in headers {
				req = req.header(h.key.clone(), h.value.clone())
			}
		}

		if let Some(body) = &self.request.body {
			req = req.body(body.to_string().clone());
		}

		let res = req.send().await?;

		Ok(res.status())
	}
}


// barebones custom error

#[derive(Debug)]
struct PostWomanError {
	msg : String,
}

impl PostWomanError {
	pub fn throw(msg: impl ToString) -> Box<dyn std::error::Error> {
		Box::new(
			PostWomanError {
				msg: msg.to_string(),
			}
		)
	}
}

impl std::fmt::Display for PostWomanError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "PostWomanError({})", self.msg)
	}
}

impl std::error::Error for PostWomanError {}
