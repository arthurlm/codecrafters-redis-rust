use std::io;

use tokio::io::AsyncWrite;

use crate::resp2::Message;

#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    Pong,
    Error(String),
}

impl Response {
    pub async fn write<W: AsyncWrite + Unpin + Send>(&self, writer: &mut W) -> io::Result<()> {
        let msg = match self {
            Response::Pong => Message::text("PONG"),
            Response::Error(text) => Message::error(text),
        };

        msg.write(writer).await
    }
}
