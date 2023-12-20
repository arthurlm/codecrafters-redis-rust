use tokio::io::AsyncRead;

use crate::{error::MiniRedisError, rdb::RedisString, resp2::Message};

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Ping,
    Echo(RedisString),
    Get(RedisString),
    Set(RedisString, RedisString),
    SetExpire(RedisString, RedisString, u64),
    Keys,
    ConfigGet(RedisString),
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
                    Self::Echo(RedisString::new(data))
                }
                // Get & Set
                [Message::Binary(arg1), Message::Binary(key)]
                    if arg1.eq_ignore_ascii_case(b"GET") =>
                {
                    Self::Get(RedisString::new(key))
                }
                [Message::Binary(arg1), Message::Binary(key), Message::Binary(value)]
                    if arg1.eq_ignore_ascii_case(b"SET") =>
                {
                    Self::Set(RedisString::new(key), RedisString::new(value))
                }
                [Message::Binary(arg1), Message::Binary(key), Message::Binary(value), Message::Binary(arg_px), Message::Binary(expiry_raw)]
                    if arg1.eq_ignore_ascii_case(b"SET") && arg_px.eq_ignore_ascii_case(b"PX") =>
                {
                    let ms_delta = String::from_utf8_lossy(expiry_raw)
                        .parse()
                        .unwrap_or_default();
                    Self::SetExpire(RedisString::new(key), RedisString::new(value), ms_delta)
                }
                // Keys
                [Message::Binary(arg1), Message::Binary(pattern)]
                    if arg1.eq_ignore_ascii_case(b"KEYS") && pattern == b"*" =>
                {
                    Self::Keys
                }

                // Config get
                [Message::Binary(arg1), Message::Binary(arg2), Message::Binary(key)]
                    if arg1.eq_ignore_ascii_case(b"CONFIG")
                        && arg2.eq_ignore_ascii_case(b"GET") =>
                {
                    Self::ConfigGet(RedisString::new(key))
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
