use tokio::io::AsyncRead;

use crate::{error::MiniRedisError, resp2::Message};

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Ping,
    UnhandledCommand,
}

impl Request {
    pub async fn read<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, MiniRedisError> {
        let msg = Message::read(reader).await?;

        match &msg {
            Message::Array(args) => match &args[..] {
                [Message::Binary(arg1)] if arg1 == b"PING" => Ok(Self::Ping),
                _ => {
                    eprintln!("Unhandled command: {msg:?}");
                    Ok(Self::UnhandledCommand)
                }
            },
            _ => {
                eprintln!("Unhandled command: {msg:?}");
                Ok(Self::UnhandledCommand)
            }
        }
    }
}
