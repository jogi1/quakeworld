use thiserror::Error;
use serde::Serialize;
use crate::protocol::message::ServerClient;

#[derive(Error, Debug, Serialize)]
pub enum MessageError {
    #[error("attempting to read beyond demo size({0}) with position({1}) and size({2})")]
    ReadBeyondSize(usize, usize, usize),
    #[error("reading unhandled type: {0}")]
    UnhandledType(ServerClient),
    #[error("reading unknown type: {0}")]
    UnknownType(u8),
    #[error("{0}")]
    StringError(String),
    #[error("Bad read")]
    BadRead,
}

impl From<String> for MessageError {
    fn from(err: String) -> MessageError{
        MessageError::StringError(err)
    }
}

