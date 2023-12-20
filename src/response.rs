use std::io;

use tokio::io::AsyncWrite;

use crate::resp2::Message;

#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    // Debug response
    Pong,
    Echo(Vec<u8>),
    // Get & Set response
    Ok,
    NoContent,
    Content(Vec<u8>),
    // Config get
    ConfigGet(Vec<u8>, Vec<u8>),
    // Unhandled command
    Error(String),
}

impl Response {
    pub async fn write<W: AsyncWrite + Unpin + Send>(&self, writer: &mut W) -> io::Result<()> {
        let msg = match self {
            Response::Pong => Message::text("PONG"),
            Response::Echo(data) => Message::bin(data),
            Response::Ok => Message::text("OK"),
            Response::NoContent => Message::Null,
            Response::Content(data) => Message::bin(data),
            Response::ConfigGet(key, value) => {
                Message::Array(vec![Message::bin(key), Message::bin(value)])
            }
            Response::Error(msg) => Message::error(msg),
        };

        msg.write(writer).await
    }
}
