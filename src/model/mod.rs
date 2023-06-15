mod request;
mod collector;

use postman_collection::{PostmanCollection, v1_0_0, v2_0_0, v2_1_0};

use self::collector::CollectRequests;

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
					out.append(&mut spec.from_self());
				}
				out
			},
			PostmanCollection::V2_1_0(spec) => {
				let mut out = Vec::new();
				for item in spec.item {
					out.append(&mut spec.from_self());
				}
				out
			},
		}
	}
}
