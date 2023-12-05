/// Convert an optional list of a type
///
/// Used to translate between a common structure in the data model for going between the model version and the specification version
pub(crate) fn convert_optional_vec<A, B: From<A>>(value: Option<Vec<A>>) -> Option<Vec<B>> {
    value.map(convert_vec)
}

pub(crate) fn convert_optional<A, B: From<A>>(value: Option<A>) -> Option<B> {
    value.map(std::convert::Into::into)
}

pub(crate) fn convert_vec<A, B: From<A>>(value: Vec<A>) -> Vec<B> {
    value.into_iter().map(std::convert::Into::into).collect()
}
