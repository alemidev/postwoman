use postman_collection::{v1_0_0, v2_0_0, v2_1_0};

pub trait IntoHost {
	fn make_host(&self) -> String;
}

impl IntoHost for v2_1_0::Host {
	fn make_host(&self) -> String {
		match self {
			v2_1_0::Host::String(x) => x.clone(),
			v2_1_0::Host::StringArray(v) => v.join("."),
		}
	}
}

impl IntoHost for v2_0_0::Host {
	fn make_host(&self) -> String {
		match self {
			v2_0_0::Host::String(x) => x.clone(),
			v2_0_0::Host::StringArray(v) => v.join("."),
		}
	}
}

impl IntoHost for v1_0_0::Host {
	fn make_host(&self) -> String {
		match self {
			v1_0_0::Host::String(x) => x.clone(),
			v1_0_0::Host::StringArray(v) => v.join("."),
		}
	}
}
