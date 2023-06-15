use postman_collection::{PostmanCollection, v1_0_0, v2_0_0, v2_1_0};

pub trait IntoRequest {
	fn make_request(&self) -> reqwest::Request;
}

impl IntoRequest for v2_0_0::RequestClass {
	fn make_request(&self) -> reqwest::Request {
		todo!()
	}
}

impl IntoRequest for v2_1_0::RequestClass {
	fn make_request(&self) -> reqwest::Request {
		todo!()
	}
}

impl IntoRequest for v1_0_0::RequestClass {
	fn make_request(&self) -> reqwest::Request {
		todo!()
	}
}
