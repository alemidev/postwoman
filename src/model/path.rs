use postman_collection::{v1_0_0, v2_0_0, v2_1_0};

pub trait IntoPath {
	fn make_path(&self) -> String;
}

impl IntoPath for v1_0_0::UrlPath {
	fn make_path(&self) -> String {
		todo!()
	}
}

impl IntoPath for v2_0_0::UrlPath {
	fn make_path(&self) -> String {
		todo!()
	}
}

impl IntoPath for v2_1_0::UrlPath {
	fn make_path(&self) -> String {
		match self {
			v2_1_0::UrlPath::String(x) => x.clone(),
			v2_1_0::UrlPath::UnionArray(v) => {
				let mut out = String::new();
				for p in v {
					match p {
						v2_1_0::PathElement::PathClass(v2_1_0::PathClass { value: Some(x), ..}) => out.push_str(&x),
						v2_1_0::PathElement::String(x) => out.push_str(&x),
						_ => {},
					}
					out.push('/');
				}
				out
			},
		}
	}
}
