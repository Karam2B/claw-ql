
#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct optional_to_many_inverse<F, T> {
    pub foriegn_key: String,
    pub from: F,
    pub to: T,
}

impl<F, T> LinkedViaIds for optional_to_many_inverse<F, T> {}
