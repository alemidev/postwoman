use postman_collection::{v1_0_0, v2_0_0, v2_1_0};

use super::request::IntoRequest;

pub enum RequestNode {
	Leaf(reqwest::RequestBuilder),
	Branch(Vec<RequestTree>),
}

pub struct RequestTree {
	pub name: String,
	pub request: RequestNode,
}

pub trait CollectRequests {
	fn collect_requests(&self) -> RequestTree;
}

impl CollectRequests for v1_0_0::Spec {
	fn collect_requests(&self) -> RequestTree {
		todo!()
	}
}

impl CollectRequests for v2_0_0::Spec {
	fn collect_requests(&self) -> RequestTree {
		let mut requests = Vec::new();
		for item in &self.item {
			requests.push(item.collect_requests());
		}
		RequestTree {
			name: self.info.name.clone(),
			request: RequestNode::Branch(requests),
		}
	}
}

impl CollectRequests for v2_1_0::Spec {
	fn collect_requests(&self) -> RequestTree {
		let mut requests = Vec::new();
		for item in &self.item {
			requests.push(item.collect_requests());
		}
		RequestTree {
			name: self.info.name.clone(),
			request: RequestNode::Branch(requests),
		}
	}
}

// impl CollectRequests for v1_0_0::Items {
// 	fn collect_requests(&self) -> Vec<reqwest::Request> {
// 		todo!()
// 	}
// }

impl CollectRequests for v2_0_0::Items {
	fn collect_requests(&self) -> RequestTree {
		let request : RequestNode;
		if self.request.is_some() && self.item.is_some() {
			panic!("some node has both a single request and child requests!");
		}
		if let Some(r) = &self.request {
			let clazz = match r {
				v2_0_0::RequestUnion::String(url) => v2_0_0::RequestClass {
					url: Some(v2_0_0::Url::String(url.clone())),
					.. Default::default()
				},
				v2_0_0::RequestUnion::RequestClass(r) => r.clone(),
			};
			request = RequestNode::Leaf(clazz.make_request());
		} else if let Some(sub) = &self.item {
			let mut requests = Vec::new();
			for item in sub {
				requests.push(item.collect_requests());
			}
			request = RequestNode::Branch(requests);
		} else {
			request = RequestNode::Branch(Vec::new()); // TODO make if/elseif/else nicer?
		}
		RequestTree {
			name: self.name.as_ref().unwrap_or(&"".to_string()).to_string(), // TODO meme
			request,
		}
	}
}

impl CollectRequests for v2_1_0::Items {
	fn collect_requests(&self) -> RequestTree {
		let request : RequestNode;
		if let Some(r) = &self.request {
			let clazz = match r {
				v2_1_0::RequestUnion::String(url) => v2_1_0::RequestClass {
					auth: None,
					body: None,
					certificate: None,
					description: None,
					header: None,
					method: None,
					proxy: None,
					url: Some(v2_1_0::Url::String(url.clone())),
				},
				v2_1_0::RequestUnion::RequestClass(r) => r.clone(),
			};
			request = RequestNode::Leaf(clazz.make_request());
		} else if let Some(sub) = &self.item {
			let mut requests = Vec::new();
			for item in sub {
				requests.push(item.collect_requests());
			}
			request = RequestNode::Branch(requests);
		} else {
			request = RequestNode::Branch(Vec::new()); // TODO make if/elseif/else nicer?
		}
		RequestTree {
			name: self.name.as_ref().unwrap_or(&"".to_string()).to_string(), // TODO meme
			request,
		}
	}
}
