use thiserror::Error;
use crate::protocol::message::MessageError;

#[derive(Error, Debug)]
pub enum MvdParseError {
    #[error("attempting to read beyond demo size({0}) with position({1}) and size({2})")]
    ReadBeyondSize(usize, usize, usize),
    #[error("unhandled command ({0})")]
    UnhandledCommand(u8),
    #[error("cannot handle qwd command")]
    QwdCommand,
    #[error("read error {0}")]
    MessageError(MessageError),
}

impl From<MessageError> for MvdParseError {
    fn from(err: MessageError) -> MvdParseError {
        return MvdParseError::MessageError(err);
    }
}

