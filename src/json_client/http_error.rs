#[simple_enum]
pub enum FailedToParse {
    SerdeJsonError,
    String,
}

impl From<&'_ str> for FailedToParse {
    fn from(value: &'_ str) -> Self {
        FailedToParse::String(value.to_string())
    }
}

// #[derive(Debug)]
// #[cfg_attr(feature = "http", derive(serde::Serialize))]
// pub struct FailedToParseBody(pub String);

// impl HttpError for FailedToParseBody {
//     fn status_code(&self) -> hyper::StatusCode {
//         StatusCode::BAD_REQUEST
//     }
// }

// #[derive(Debug)]
// #[cfg_attr(feature = "http", derive(serde::Serialize))]
// pub struct FilterIsNotApplicableForCollection;

// impl HttpError for FilterIsNotApplicableForCollection {
//     fn status_code(&self) -> hyper::StatusCode {
//         StatusCode::BAD_REQUEST
//     }
// }

// #[simple_enum]
// #[derive(Debug)]
// pub enum FilterError {
//     FailedToParseBody,
//     FilterIsNotApplicableForCollection,
// }
