use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ErrorKind {
    InternalFailure,
    ExternalFailure,
    AutoConfigFailure,
    StateUpdateFailure,
    AmqpFailure,
    ApiFailure,
}

impl From<cooplan_auth::error::ErrorKind> for ErrorKind {
    fn from(error_kind: cooplan_auth::error::ErrorKind) -> Self {
        match error_kind {
            _ => ErrorKind::InternalFailure,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Error {
        Error {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<cooplan_auth::error::Error> for Error {
    fn from(error: cooplan_auth::error::Error) -> Self {
        let error_kind: ErrorKind = error.kind().into();

        Error {
            kind: error_kind,
            message: error.message,
        }
    }
}
