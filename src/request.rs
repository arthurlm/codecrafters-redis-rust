use tokio::io::AsyncRead;

use crate::{error::MiniRedisError, resp2::Message};

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Ping,
    Echo(Vec<u8>),
    UnhandledCommand,
}

impl Request {
    pub async fn read<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, MiniRedisError> {
        let msg = Message::read(reader).await?;

        match &msg {
            Message::Array(args) => match &args[..] {
                // Debug commands
                [Message::Binary(arg1)] if arg1.eq_ignore_ascii_case(b"PING") => Ok(Self::Ping),
                [Message::Binary(arg1), Message::Binary(data)]
                    if arg1.eq_ignore_ascii_case(b"ECHO") =>
                {
                    Ok(Self::Echo(data.clone()))
                }
                // Unhandled command
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
