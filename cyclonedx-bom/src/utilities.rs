/// Convert an optional list of a type
///
/// Used to translate between a common structure in the data model for going between the model version and the specification version
pub(crate) fn convert_optional_vec<A, B: From<A>>(value: Option<Vec<A>>) -> Option<Vec<B>> {
    value.map(|v| v.into_iter().map(std::convert::Into::into).collect())
}

pub(crate) fn convert_optional<A, B: From<A>>(value: Option<A>) -> Option<B> {
    value.map(std::convert::Into::into)
}
