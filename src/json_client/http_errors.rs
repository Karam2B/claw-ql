pub trait HttpError {
    fn status_code(&self) -> StatusCode;
    fn sub_code(&self) -> Option<&'static str> {
        None
    }
    fn sub_message(&self) -> Option<String> {
        None
    }
}

pub struct ErrorId(i64);
pub trait ErrorReporter {
    fn report(&self, input: serde_json::Value) -> ErrorId;
}

pub enum JsonError {
    ParseError(String),
    String(String),
}
