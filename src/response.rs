use std::io;

use tokio::io::AsyncWrite;

use crate::{rdb::RedisString, resp2::Message, ServerMode};

#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    // Debug response
    Pong,
    Echo(RedisString),
    InfoReplication {
        role: ServerMode,
        master_replid: String,
        master_repl_offset: usize,
        repl_backlog_active: usize,
        repl_backlog_size: usize,
        repl_backlog_first_byte_offset: usize,
        repl_backlog_histlen: usize,
    },
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
            Response::InfoReplication {
                role,
                master_replid,
                master_repl_offset,
                repl_backlog_active,
                repl_backlog_size,
                repl_backlog_first_byte_offset,
                repl_backlog_histlen,
            } => {
                let role = match role {
                    ServerMode::Master => "master",
                    ServerMode::Slave => "slave",
                };

                let data = format!(
                    "role:{role}\n\
                    master_replid:{master_replid}\n\
                    master_repl_offset:{master_repl_offset}\n\
                    repl_backlog_active:{repl_backlog_active}\n\
                    repl_backlog_size:{repl_backlog_size}\n\
                    repl_backlog_first_byte_offset:{repl_backlog_first_byte_offset}\n\
                    repl_backlog_histlen:{repl_backlog_histlen}\n\
                    "
                );
                Message::bin(data.as_bytes())
            }
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
