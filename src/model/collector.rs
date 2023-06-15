use postman_collection::{PostmanCollection, v1_0_0, v2_0_0, v2_1_0};

pub trait CollectRequests {
	fn from_self(&self) -> Vec<reqwest::Request>;
}

impl CollectRequests for v1_0_0::Spec {
	fn from_self(&self) -> Vec<reqwest::Request> {
		todo!()
	}
}

impl CollectRequests for v2_0_0::Spec {
	fn from_self(&self) -> &reqwest::Request {
		let mut requests = Vec::new();
		if let Some(r) = root.request {
			let clazz = match r {
				v2_0_0::RequestUnion::String(url) => v2_0_0::RequestClass {
					auth: None,
					body: None,
					certificate: None,
					description: None,
					header: None,
					method: None,
					proxy: None,
					url: Some(v2_0_0::Url::String(url)),
				},
				v2_0_0::RequestUnion::RequestClass(r) => r,
			};
			requests.push(clazz.make_request());
		}
		if let Some(sub) = root.item {
			for item in sub {
				requests.append(&mut self.from_self());
			}

		}
		requests
	}
}

impl CollectRequests for v2_1_0::Spec {
	fn from_self(&self) -> Vec<reqwest::Request> {
		todo!()
	}
}
