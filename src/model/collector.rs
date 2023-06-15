use postman_collection::{v1_0_0, v2_0_0, v2_1_0};

use super::request::IntoRequest;

pub trait CollectRequests {
	fn collect_requests(&self) -> Vec<reqwest::Request>;
}

impl CollectRequests for v1_0_0::Spec {
	fn collect_requests(&self) -> Vec<reqwest::Request> {
		todo!()
	}
}

impl CollectRequests for v2_0_0::Spec {
	fn collect_requests(&self) -> Vec<reqwest::Request> {
		let mut requests = Vec::new();
		for item in &self.item {
			requests.append(&mut item.collect_requests());
		}
		requests
	}
}

impl CollectRequests for v2_1_0::Spec {
	fn collect_requests(&self) -> Vec<reqwest::Request> {
		let mut requests = Vec::new();
		for item in &self.item {
			requests.append(&mut item.collect_requests());
		}
		requests
	}
}

// impl CollectRequests for v1_0_0::Items {
// 	fn collect_requests(&self) -> Vec<reqwest::Request> {
// 		todo!()
// 	}
// }

impl CollectRequests for v2_0_0::Items {
	fn collect_requests(&self) -> Vec<reqwest::Request> {
		let mut requests = Vec::new();
		if let Some(r) = &self.request {
			let clazz = match r {
				v2_0_0::RequestUnion::String(url) => v2_0_0::RequestClass {
					url: Some(v2_0_0::Url::String(url.clone())),
					.. Default::default()
				},
				v2_0_0::RequestUnion::RequestClass(r) => r.clone(),
			};
			requests.push(clazz.make_request());
		}
		if let Some(sub) = &self.item {
			for item in sub {
				requests.append(&mut item.collect_requests());
			}
		}
		requests
	}
}

impl CollectRequests for v2_1_0::Items {
	fn collect_requests(&self) -> Vec<reqwest::Request> {
		let mut requests = Vec::new();
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
			requests.push(clazz.make_request());
		}
		if let Some(sub) = &self.item {
			for item in sub {
				requests.append(&mut item.collect_requests());
			}
		}
		requests
	}
}
