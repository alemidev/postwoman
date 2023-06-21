use crate::{requestor::{TestResultTree, TestResultNode, TestResultCase}, model::collector::{RequestTree, RequestNode}};

// TODO maybe make a generic trait Displayable and make just one recursive function?


pub async fn show_results(tree: TestResultTree, verbose: bool, pretty: bool) {
	show_results_r(tree, verbose, pretty, 0).await
}

#[async_recursion::async_recursion]
pub async fn show_results_r(tree: TestResultTree, verbose: bool, pretty: bool, depth: usize) {
	let indent_skip = " │".repeat(depth);
	let indent_node = match depth {
		0 => "".into(),
		1 => " ├".into(),
		x => " │".repeat(x - 1) + " ├",
	};
	match tree.result {
		TestResultNode::Leaf(rx) => {
			let res = rx.await.unwrap();
			let method = res.request.method().as_str();
			let url = res.request.url().as_str();
			match res.result {
				TestResultCase::Skip => {
					println!("{} ? --- >> {} {}", indent_node, method, url);
					if verbose {
						println!("{}   [skipped]", indent_skip);
						println!("{}", indent_skip);
					}
				},
				TestResultCase::Success(response) => {
					let status_code = response.status().as_u16();
					let marker = if status_code < 400 { '✓' } else { '×' };
					println!("{} {} {} >> {} {}", indent_node, marker, status_code, method, url);
					if verbose {
						let body = process_json_body(response.text().await.unwrap(), pretty);
						println!("{}   {}", indent_skip, body.replace("\n", &format!("\n{}   ", indent_skip)));
						println!("{}", indent_skip);
					}
				},
				TestResultCase::Failure(err) => {
					println!("{} ! ERROR >> {} {}", indent_node, method, url);
					if verbose {
						println!("{}   {}", indent_skip, err);
						println!("{}", indent_skip);
					}
				}
			}
		},
		TestResultNode::Branch(results) => {
			println!("{}─┐ {}", indent_node, tree.name);
			if verbose {
				if let Some(descr) = tree.description {
					println!("{} │   {}", indent_skip, descr);
				}
				println!("{} │", indent_skip);
			}
			for res in results {
				show_results_r(res, verbose, pretty, depth + 1).await;
			}
			println!("{} ╵", indent_skip);
		},
	}
}

pub fn show_requests(tree: RequestTree, verbose: bool, pretty: bool) {
	show_requests_r(tree, verbose, pretty, 0);
}

pub fn show_requests_r(tree: RequestTree, verbose: bool, pretty: bool, depth: usize) {
	let indent_skip = " │".repeat(depth);
	let indent_node = match depth {
		0 => "".into(),
		1 => " ├".into(),
		x => " │".repeat(x - 1) + " ├",
	};
	match tree.request {
		RequestNode::Leaf(request) => {
			let method = request.method().as_str();
			let url = request.url().as_str();
			println!("{} * {} {}", indent_node, method, url);
			if verbose {
				let headers = request.headers()
					.iter()
					.map(|(k, v)| format!("{}:{}", k.as_str(), std::str::from_utf8(v.as_bytes()).unwrap()))
					.collect::<Vec<String>>();
				if headers.len() > 0 {
					if pretty {
						println!("{}   [", indent_skip);
						for h in headers {
							println!("{}     {}", indent_skip, h);
						}
						println!("{}   ]", indent_skip);
					} else {
						println!("{}   [ {} ]", indent_skip, headers.join(", "));
					}
				}
				if let Some(body) = request.body() {
					if let Some(bytes) = body.as_bytes() {
						let txt = process_json_body(std::str::from_utf8(bytes).unwrap().to_string(), pretty);
						println!("{}   {}", indent_skip, txt.replace("\n", &format!("\n{}   ", indent_skip)));
					} else {
						println!("{}   << streaming body >>", indent_skip);
					}
				}
				println!("{}", indent_skip);
			}
		},
		RequestNode::Branch(requests) => {
			println!("{}─┐ {}", indent_node, tree.name);
			if verbose {
				if let Some(descr) = tree.description {
					println!("{} │   {}", indent_skip, descr);
				}
				println!("{} │", indent_skip);
			}
			for req in requests {
				show_requests_r(req, verbose, pretty, depth + 1);
			}
			println!("{} ╵", indent_skip);
		},
	}
}

fn process_json_body(txt: String, pretty: bool) -> String {
	if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
		if pretty {
			if let Ok(t) = serde_json::to_string_pretty(&v) {
				return t;
			}
		} else {
			if let Ok(t) = serde_json::to_string(&v) { // try to minify it anyway
				return t;
			}
		}
	}
	txt
}
