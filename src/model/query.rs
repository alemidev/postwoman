use postman_collection::{v1_0_0, v2_0_0, v2_1_0};

pub trait IntoQueryString {
	fn make_query(&self) -> String;
}

impl IntoQueryString for Vec<v2_1_0::QueryParam> {
	fn make_query(&self) -> String {
		self.iter()
			.filter_map(|x| Some(format!("{}={}", x.key?, x.value?)))
			.collect::<Vec<String>>()
			.join("&")
	}
}

impl IntoQueryString for Vec<v2_0_0::QueryParam> {
	fn make_query(&self) -> String {
		self.iter()
			.filter_map(|x| Some(format!("{}={}", x.key?, x.value?)))
			.collect::<Vec<String>>()
			.join("&")
	}
}

impl IntoQueryString for Vec<v1_0_0::QueryParam> {
	fn make_query(&self) -> String {
		self.iter()
			.filter_map(|x| Some(format!("{}={}", x.key?, x.value?)))
			.collect::<Vec<String>>()
			.join("&")
	}
}
