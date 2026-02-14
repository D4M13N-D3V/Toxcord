use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToxError {
    #[error("Failed to create Tox instance: {0}")]
    New(String),

    #[error("Null pointer from Tox API")]
    NullPointer,

    #[error("Failed to bootstrap: {0}")]
    Bootstrap(String),

    #[error("Failed to add friend: {0}")]
    FriendAdd(String),

    #[error("Failed to send message: {0}")]
    SendMessage(String),

    #[error("Failed to set name: {0}")]
    SetName(String),

    #[error("Failed to set status message: {0}")]
    SetStatusMessage(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Save data error: {0}")]
    SaveData(String),

    #[error("Group error: {0}")]
    Group(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("ToxAV error: {0}")]
    ToxAv(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type ToxResult<T> = Result<T, ToxError>;
