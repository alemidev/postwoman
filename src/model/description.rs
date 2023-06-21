use postman_collection::{v2_0_0, v2_1_0};

pub trait IntoOptionalString {
	fn as_string(&self) -> Option<String>;
}

impl IntoOptionalString for v2_0_0::DescriptionUnion {
	fn as_string(&self) -> Option<String> {
		match self {
			v2_0_0::DescriptionUnion::String(x) => Some(x.clone()),
			v2_0_0::DescriptionUnion::Description(
				v2_0_0::Description { content, .. }
			) => content.clone(),
		}
	}
}

impl IntoOptionalString for v2_1_0::DescriptionUnion {
	fn as_string(&self) -> Option<String> {
		match self {
			v2_1_0::DescriptionUnion::String(x) => Some(x.clone()),
			v2_1_0::DescriptionUnion::Description(
				v2_1_0::Description { content, .. }
			) => content.clone(),
		}
	}
}
