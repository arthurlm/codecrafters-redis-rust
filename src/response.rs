use std::io;

use tokio::io::AsyncWrite;

use crate::resp2::Message;

#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    Pong,
    Echo(Vec<u8>),
    Error(String),
}

impl Response {
    pub async fn write<W: AsyncWrite + Unpin + Send>(&self, writer: &mut W) -> io::Result<()> {
        let msg = match self {
            Response::Pong => Message::text("PONG"),
            Response::Echo(data) => Message::bin(data),
            Response::Error(msg) => Message::error(msg),
        };

        msg.write(writer).await
    }
}
