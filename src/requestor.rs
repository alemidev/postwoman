use crate::model::collector::{RequestTree, RequestNode};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum TestResultNode {
	Leaf(oneshot::Receiver<TestResultHolder>),
	Branch(Vec<TestResultTree>),
}

#[derive(Debug)]
pub struct TestResultTree {
	pub name: String,
	pub description: Option<String>,
	pub result: TestResultNode,
}

#[derive(Debug)]
pub struct TestResultHolder {
	pub request: reqwest::Request,
	pub result: TestResultCase,
}

#[derive(Debug)]
pub enum TestResultCase {
	Skip,
	Success(reqwest::Response),
	Failure(reqwest::Error),
}

#[async_recursion::async_recursion]
pub async fn send_requests(tree: RequestTree, client: Option<reqwest::Client>, dry_run: bool) -> TestResultTree {
	let result : TestResultNode;

	match tree.request {
		RequestNode::Leaf(request) => {
			let (tx, rx) = oneshot::channel();
			let c = client.unwrap_or(
				reqwest::Client::builder()
					.user_agent(crate::APP_USER_AGENT)
					.build()
					.unwrap()
			);
			tokio::spawn(async move {
				let res = if dry_run { TestResultCase::Skip } else {
					match c.execute(request.try_clone().unwrap()).await {
						Ok(res) => TestResultCase::Success(res),
						Err(e) => TestResultCase::Failure(e),
					}
				};
				tx.send(TestResultHolder { request, result: res }).unwrap();
			});
			result = TestResultNode::Leaf(rx);
		},
		RequestNode::Branch(requests) => {
			let mut out = Vec::new();
			for req in requests {
				out.push(send_requests(req, client.clone(), dry_run).await);
			}
			result = TestResultNode::Branch(out);
		}
	}

	TestResultTree { name: tree.name, description: tree.description, result }
}
