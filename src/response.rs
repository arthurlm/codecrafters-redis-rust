use std::io;

use tokio::io::AsyncWrite;

use crate::{rdb::RedisString, resp2::Message};

#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    // Debug response
    Pong,
    Echo(RedisString),
    // Get & Set response
    Ok,
    NoContent,
    Content(RedisString),
    // Key matches
    KeyMatches(Vec<RedisString>),
    // Config get
    ConfigGet(RedisString, RedisString),
    // Unhandled command
    Error(String),
}

impl Response {
    pub async fn write<W: AsyncWrite + Unpin + Send>(&self, writer: &mut W) -> io::Result<()> {
        let msg = match self {
            Response::Pong => Message::text("PONG"),
            Response::Echo(data) => Message::bin(data.as_slice()),
            Response::Ok => Message::text("OK"),
            Response::NoContent => Message::Null,
            Response::Content(data) => Message::bin(data.as_slice()),
            Response::KeyMatches(keys) => Message::Array(
                keys.iter()
                    .map(|key| Message::bin(key.as_slice()))
                    .collect(),
            ),
            Response::ConfigGet(key, value) => Message::Array(vec![
                Message::bin(key.as_slice()),
                Message::bin(value.as_slice()),
            ]),
            Response::Error(msg) => Message::error(msg),
        };

        msg.write(writer).await
    }
}
