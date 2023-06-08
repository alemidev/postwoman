use postman_collection::{PostmanCollection, v1_0_0, v2_0_0, v2_1_0};

pub struct PostWomanCollection {
	collection: PostmanCollection
}

impl From<PostmanCollection> for PostWomanCollection {
	fn from(value: PostmanCollection) -> Self {
		Self { collection: value }
	}
}

impl PostWomanCollection {
	pub fn description(&self) -> Option<&String> {
		match &self.collection {
			PostmanCollection::V1_0_0(spec) => {
				spec.description.as_ref()
			},
			PostmanCollection::V2_0_0(spec) => {
				match &spec.info.description {
					Some(v2_0_0::DescriptionUnion::String(x)) => Some(x),
					Some(v2_0_0::DescriptionUnion::Description(v2_0_0::Description { content, .. })) => content.as_ref(),
					None => None,
				}
			},
			PostmanCollection::V2_1_0(spec) => {
				match &spec.info.description {
					Some(v2_1_0::DescriptionUnion::String(x)) => Some(x),
					Some(v2_1_0::DescriptionUnion::Description(v2_1_0::Description { content, .. })) => content.as_ref(),
					None => None,
				}
			},
		}
	}

	pub fn requests(&self) -> Vec<reqwest::Request> {
		match self.collection {
			PostmanCollection::V1_0_0(_) => todo!(),
			PostmanCollection::V2_0_0(spec) => {
				let mut out = Vec::new();
				for item in spec.item {
					out.append(&mut collect_requests_2_0_0_r(&item));
				}
				out
			},
			PostmanCollection::V2_1_0(spec) => {
				let mut out = Vec::new();
				for item in spec.item {
					out.append(&mut collect_requests_2_1_0_r(&item));
				}
				out
			},
		}
	}
}

pub fn collect_requests_1_0_0_r(root: &v1_0_0::Spec) -> Vec<reqwest::Request> {
	todo!()
}

pub trait IntoRequest {
	fn make_request(&self) -> reqwest::Request;
}

impl IntoRequest for v2_0_0::RequestClass {
	fn make_request(&self) -> reqwest::Request {
		todo!()
	}
}

pub trait CollectRequests {
	fn from_self(&self) -> &reqwest::Request;
}

pub fn collect_requests_2_0_0_r(root: &v2_0_0::Items) -> Vec<reqwest::Request> {
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
			requests.append(&mut collect_requests_2_0_0_r(&item));
		}

	}
	requests
}

pub fn collect_requests_2_1_0_r(root: &v2_1_0::Items) -> Vec<reqwest::Request> {
	todo!()
}
