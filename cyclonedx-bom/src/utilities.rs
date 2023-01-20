use std::convert::TryFrom;
use crate::errors::BomError;

/// Convert an optional list of a type
///
/// Used to translate between a common structure in the data model for going between the model version and the specification version
pub(crate) fn convert_optional_vec<A, B: From<A>>(value: Option<Vec<A>>) -> Option<Vec<B>> {
    value.map(convert_vec)
}

/*
pub(crate) fn try_convert_optional_vec<A, B: TryFrom<A, Error = BomError>>(value: Option<Vec<A>>) -> Option<Result<Vec<B>, BomError>> {
    value.map(try_convert_vec)
}
*/

pub(crate) fn convert_optional<A, B: From<A>>(value: Option<A>) -> Option<B> {
    value.map(std::convert::Into::into)
}

pub(crate) fn try_convert_optional<A, B: TryFrom<A, Error = BomError>>(value: Option<A>) -> Result<Option<B>, BomError> {
    // todo: check if this could be handled more elegantly
    let value = value.map(std::convert::TryInto::try_into);
    match value {
        Some(result ) => {
            match result {
                Err(e) => Err(e),
                Ok(value) => Ok(Some(value)),
            }
        }
        None => Ok(None),
    }
}

pub(crate) fn convert_vec<A, B: From<A>>(value: Vec<A>) -> Vec<B> {
    value.into_iter().map(std::convert::Into::into).collect()
}

pub(crate) fn try_convert_vec<A, B: TryFrom<A, Error = BomError>>(value: Vec<A>) -> Result<Vec<B>, BomError> {
    value.into_iter().map(std::convert::TryInto::try_into).collect()
}

/*
For cases where you return Result, it's useful to know that .collect() can take iterator of Result<T,E> and collect into Result<Vec<T>, E>, so you can then use ? on the collection:

a.into_iter()
    .map(|x| x.ok_or(NoneError))
    .collect::<Result<Vec<_>,_>>()?
*/