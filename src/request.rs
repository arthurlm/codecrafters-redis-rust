use tokio::io::AsyncRead;

use crate::{error::MiniRedisError, resp2::Message};

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Ping,
    Echo(Vec<u8>),
    Get(Vec<u8>),
    Set(Vec<u8>, Vec<u8>),
    UnhandledCommand,
}

impl Request {
    pub async fn read<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, MiniRedisError> {
        let msg = Message::read(reader).await?;

        Ok(match &msg {
            Message::Array(args) => match &args[..] {
                // Debug commands
                [Message::Binary(arg1)] if arg1.eq_ignore_ascii_case(b"PING") => Self::Ping,
                [Message::Binary(arg1), Message::Binary(data)]
                    if arg1.eq_ignore_ascii_case(b"ECHO") =>
                {
                    Self::Echo(data.clone())
                }
                // Get & Set
                [Message::Binary(arg1), Message::Binary(key)]
                    if arg1.eq_ignore_ascii_case(b"GET") =>
                {
                    Self::Get(key.to_vec())
                }
                [Message::Binary(arg1), Message::Binary(key), Message::Binary(value)]
                    if arg1.eq_ignore_ascii_case(b"SET") =>
                {
                    Self::Set(key.to_vec(), value.to_vec())
                }
                // Unhandled command
                _ => {
                    eprintln!("Unhandled command: {msg:?}");
                    Self::UnhandledCommand
                }
            },
            _ => {
                eprintln!("Unhandled command: {msg:?}");
                Self::UnhandledCommand
            }
        })
    }
}
