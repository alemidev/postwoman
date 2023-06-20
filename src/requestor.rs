use crate::model::collector::{RequestTree, RequestNode};
use regex::Regex;
use tokio::sync::oneshot;

#[async_recursion::async_recursion]
pub async fn send_requests(tree: RequestTree, filter: Option<Regex>) -> TestResultTree {
	let result : TestResultNode;

	match tree.request {
		RequestNode::Leaf(req) => {
			let (tx, rx) = oneshot::channel();
			tokio::spawn(async move {
				let request = req.build().unwrap();
				if filter.is_some() && !filter.unwrap().is_match(request.url().as_str()) {
					tx.send(TestResultHolder { request, result: TestResultCase::Skip }).unwrap();
				} else {
					let c = reqwest::Client::default();
					let res = match c.execute(request.try_clone().unwrap()).await {
						Ok(res) => TestResultCase::Success(res),
						Err(e) => TestResultCase::Failure(e),
					};
					tx.send(TestResultHolder { request, result: res }).unwrap();
				}
			});
			result = TestResultNode::Leaf(rx);
		},
		RequestNode::Branch(requests) => {
			let mut out = Vec::new();
			for req in requests {
				out.push(send_requests(req, filter.clone()).await);
			}
			result = TestResultNode::Branch(out);
		}
	}

	TestResultTree { name: tree.name, result }
}

pub async fn show_results(tree: TestResultTree, verbose: bool, pretty: bool) {
	show_results_r(tree, verbose, pretty, 0).await
}

#[async_recursion::async_recursion]
pub async fn show_results_r(tree: TestResultTree, verbose: bool, pretty: bool, depth: usize) {
	let indent = " │".repeat(depth);
	match tree.result {
		TestResultNode::Leaf(rx) => {
			let res = rx.await.unwrap();
			let method = res.request.method().as_str();
			let url = res.request.url().as_str();
			match res.result {
				TestResultCase::Skip => {},
				TestResultCase::Success(response) => {
					let status_code = response.status().as_u16();
					let marker = if status_code < 400 { '✓' } else { '×' };
					println!("{} ├ {} {} >> {} {}", indent, marker, status_code, method, url);
					if verbose {
						let mut body = response.text().await.unwrap();
						if pretty {
							if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
								if let Ok(t) = serde_json::to_string_pretty(&v) {
									body = t;
								}
							}
						}
						println!("{} │  {}", indent, body.replace("\n", &format!("\n │{}  ", indent)));
						println!("{} │", indent);
					}
				},
				TestResultCase::Failure(err) => {
					println!("{} ├ ! ERROR >> {} {}", indent, method, url);
					if verbose {
						println!("{} │  {}", indent, err);
						println!("{} │", indent);
					}
				}
			}
		},
		TestResultNode::Branch(results) => {
			println!("{} ├─┐ {}", indent, tree.name);
			for res in results {
				show_results_r(res, verbose, pretty, depth + 1).await;
			}
			println!("{} │ ╵", indent);
		},
	}
}

#[derive(Debug)]
pub enum TestResultNode {
	Leaf(oneshot::Receiver<TestResultHolder>),
	Branch(Vec<TestResultTree>),
}

#[derive(Debug)]
pub struct TestResultTree {
	name: String,
	result: TestResultNode,
}

#[derive(Debug)]
pub struct TestResultHolder {
	request: reqwest::Request,
	result: TestResultCase,
}

#[derive(Debug)]
pub enum TestResultCase {
	Skip,
	Success(reqwest::Response),
	Failure(reqwest::Error),
}
