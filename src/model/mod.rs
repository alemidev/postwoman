pub mod request;
pub mod collector;
pub mod description;

use postman_collection::{PostmanCollection, v2_0_0, v2_1_0};
use regex::Regex;

use self::collector::{CollectRequests, RequestTree};

#[derive(Debug)]
pub struct PostWomanCollection {
	collection: PostmanCollection
}

impl From<PostmanCollection> for PostWomanCollection {
	fn from(value: PostmanCollection) -> Self {
		Self { collection: value }
	}
}

impl PostWomanCollection {

	pub fn from_path(path: &str) -> postman_collection::errors::Result<Self> {
		Ok(postman_collection::from_path(path)?.into())
	}

	pub fn name(&self) -> &String {
		match &self.collection {
			PostmanCollection::V1_0_0(_spec) => todo!(),
			PostmanCollection::V2_0_0(spec) => &spec.info.name,
			PostmanCollection::V2_1_0(spec) => &spec.info.name,
		}
	}

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

	pub fn requests(&self, filter: Option<&Regex>) -> Option<RequestTree> {
		match &self.collection {
			PostmanCollection::V1_0_0(_) => todo!(),
			PostmanCollection::V2_0_0(spec) => spec.collect_requests(filter),
			PostmanCollection::V2_1_0(spec) => spec.collect_requests(filter),
		}
	}
}
