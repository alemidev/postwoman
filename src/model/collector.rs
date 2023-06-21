use postman_collection::{v1_0_0, v2_0_0, v2_1_0};
use regex::Regex;

use super::{request::IntoRequest, description::IntoOptionalString};

pub enum RequestNode {
	Leaf(reqwest::Request),
	Branch(Vec<RequestTree>),
}

pub struct RequestTree {
	pub name: String,
	pub description: Option<String>,
	pub request: RequestNode,
}

pub trait CollectRequests {
	fn collect_requests(&self, filter: Option<&Regex>) -> Option<RequestTree>;
}

impl CollectRequests for v1_0_0::Spec {
	fn collect_requests(&self, _filter: Option<&Regex>) -> Option<RequestTree> {
		todo!()
	}
}

impl CollectRequests for v2_0_0::Spec {
	fn collect_requests(&self, filter: Option<&Regex>) -> Option<RequestTree> {
		let requests = self.item.iter()
			.filter_map(|x| x.collect_requests(filter))
			.collect::<Vec<RequestTree>>();
		(!requests.is_empty())
			.then(|| RequestTree {
					name: self.info.name.clone(),
					description: self.info.description.as_ref().map_or(None, |x| x.as_string()),
					request: RequestNode::Branch(requests),
				}
			)
	}
}

impl CollectRequests for v2_1_0::Spec {
	fn collect_requests(&self, filter: Option<&Regex>) -> Option<RequestTree> {
		let requests = self.item.iter()
			.filter_map(|x| x.collect_requests(filter))
			.collect::<Vec<RequestTree>>();
		(!requests.is_empty())
			.then(||
				RequestTree {
					name: self.info.name.clone(),
					description: self.info.description.as_ref().map_or(None, |x| x.as_string()),
					request: RequestNode::Branch(requests),
				}
			)
	}
}

// impl CollectRequests for v1_0_0::Items {
// 	fn collect_requests(&self) -> Vec<reqwest::Request> {
// 		todo!()
// 	}
// }

impl CollectRequests for v2_0_0::Items {
	fn collect_requests(&self, filter: Option<&Regex>) -> Option<RequestTree> {
		if self.request.is_some() && self.item.is_some() {
			panic!("node has both a single request and child requests!");
		}
		let name = self.name.as_ref().unwrap_or(&"".to_string()).to_string();
		let description = self.description.as_ref().map_or(None, |x| x.as_string());
		if let Some(r) = &self.request {
			let clazz = match r {
				v2_0_0::RequestUnion::String(url) => v2_0_0::RequestClass {
					url: Some(v2_0_0::Url::String(url.clone())),
					.. Default::default()
				},
				v2_0_0::RequestUnion::RequestClass(r) => r.clone(),
			};
			Some(RequestTree { name, description, request: RequestNode::Leaf(clazz.make_request(filter)?) })
		} else if let Some(sub) = &self.item {
			let requests = sub.iter()
				.filter_map(|x| x.collect_requests(filter))
				.collect::<Vec<RequestTree>>();
			(!requests.is_empty())
				.then(|| RequestTree { name, description, request: RequestNode::Branch(requests) })
		} else {
			None
		}
	}
}

impl CollectRequests for v2_1_0::Items {
	fn collect_requests(&self, filter: Option<&Regex>) -> Option<RequestTree> {
		if self.request.is_some() && self.item.is_some() {
			panic!("node has both a single request and child requests!");
		}
		let name = self.name.as_ref().unwrap_or(&"".to_string()).to_string();
		let description = self.description.as_ref().map_or(None, |x| x.as_string());
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
			Some(RequestTree { name, description, request: RequestNode::Leaf(clazz.make_request(filter)?) })
		} else if let Some(sub) = &self.item {
			let requests = sub.iter()
				.filter_map(|x| x.collect_requests(filter))
				.collect::<Vec<RequestTree>>();
			(!requests.is_empty())
				.then(|| RequestTree { name, description, request: RequestNode::Branch(requests) })
		} else {
			None
		}
	}
}
