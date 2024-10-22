use indexmap::IndexMap;

use crate::{PostWomanCollection, PostWomanError};

pub const TIMESTAMP_FMT: &str = "%H:%M:%S%.6f"; 

pub trait PrintableResult {
	fn print(self);
}

pub trait ReportableResult {
	fn report(self);
}

// TODO this is not really nice, maybe a struct? Maybe pass them in some other way??
pub type RunResult = (Result<String, PostWomanError>, String, String, i64);

impl PrintableResult for RunResult {
	fn print(self) {
		let (result, _namespace, _name, _elapsed) = self;
		match result {
			Ok(x) => println!("{x}"),
			Err(e) => eprintln!(" ! {e}"),
		}
	}
}

impl ReportableResult for RunResult {
	fn report(self) {
		let (res, namespace, name, elapsed) = self;
		let success = res.is_ok();
		let result = match res {
			Ok(x) => x,
			Err(e) => e.to_string(),
		};

		println!(
			"{}",
			serde_json::to_string(
				&serde_json::json!({
					"namespace": namespace,
					"route": name,
					"success": success,
					"result": result,
					"elapsed": elapsed,
				})
			)
				.expect("failed serializing literal json")
		);
	}
}

// TODO the last tuple element is "compact"... this really needs a better way, maybe a struct!!
pub type ListResult = (IndexMap<String, PostWomanCollection>, bool);

impl PrintableResult for ListResult {
	fn print(self) {
		let (collections, compact) = self;
		for (namespace, collection) in collections {
			println!("-> {namespace}");

			for (key, value) in collection.env {
				println!(" + {key}={}", crate::ext::stringify_toml(&value));
			}

			for (name, endpoint) in collection.route {
				let url = endpoint.url(collection.client.base.as_deref())
					.split('?')
					.next()
					.unwrap_or_default()
					.to_string();
				let method = endpoint.method.as_deref().unwrap_or("GET");
				println!(" - {name} \t{method} \t{url}");
				if ! compact {
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
		}
	}
}

impl ReportableResult for ListResult {
	fn report(self) {
		let (collections, _compact) = self;
		println!("{}", serde_json::to_string(&collections).expect("collections are not valid json"));
	}
}
