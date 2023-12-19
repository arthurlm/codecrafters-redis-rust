use std::{future::Future, io, pin::Pin};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::MiniRedisError;

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Text(String),
    Error(String),
    Integer(i64),
    Binary(Vec<u8>),
    Null,
    Array(Vec<Message>),
}

impl Message {
    pub fn text(content: &str) -> Self {
        Self::Text(content.to_string())
    }

    pub fn error(content: &str) -> Self {
        Self::Error(content.to_string())
    }

    pub fn bin(content: &[u8]) -> Self {
        Self::Binary(content.to_vec())
    }

    /// Read reader and decode its content.
    ///
    /// Async recursive function are not supported out of the box.
    /// See: rustc --explain E0733
    pub fn read<R: AsyncRead + Unpin + Send>(
        reader: &mut R,
    ) -> Pin<Box<dyn Future<Output = Result<Self, MiniRedisError>> + Send + '_>> {
        Box::pin(async move {
            let msg_type = reader.read_u8().await?;
            match msg_type {
                b'+' => {
                    let data = read_until_crlf(reader).await?;
                    let text = String::from_utf8(data)?;
                    Ok(Self::Text(text))
                }
                b'-' => {
                    let data = read_until_crlf(reader).await?;
                    let text = String::from_utf8(data)?;
                    Ok(Self::Error(text))
                }
                b':' => {
                    let data = read_until_crlf(reader).await?;
                    let text = String::from_utf8(data)?;
                    let number = text.parse()?;
                    Ok(Self::Integer(number))
                }
                b'$' => {
                    // Parse payload size
                    let data_len_raw = read_until_crlf(reader).await?;
                    let data_len_text = String::from_utf8(data_len_raw)?;
                    let data_len: i64 = data_len_text.parse()?;

                    // Check null string
                    if data_len < 0 {
                        return Ok(Self::Null);
                    }

                    // Read payload
                    let mut data = vec![0_u8; data_len as usize];
                    reader.read_exact(&mut data).await?;

                    // Check termination bytes
                    if reader.read_u8().await? != b'\r' {
                        return Err(MiniRedisError::InvalidMessageEnd);
                    }
                    if reader.read_u8().await? != b'\n' {
                        return Err(MiniRedisError::InvalidMessageEnd);
                    }

                    Ok(Self::Binary(data))
                }
                b'*' => {
                    // Parse element count
                    let elem_count_raw = read_until_crlf(reader).await?;
                    let elem_count_text = String::from_utf8(elem_count_raw)?;
                    let elem_count: i64 = elem_count_text.parse()?;

                    // Handle null array
                    if elem_count < 0 {
                        return Ok(Message::Null);
                    }

                    // Parse each elements
                    let mut items = Vec::with_capacity(elem_count as usize);
                    for _ in 0..elem_count {
                        let item = Self::read(reader).await?;
                        items.push(item);
                    }

                    Ok(Self::Array(items))
                }
                _ => Err(MiniRedisError::InvalidMessageType(msg_type.into())),
            }
        })
    }

    pub fn write<'a, W: AsyncWrite + Unpin + Send>(
        &'a self,
        writer: &'a mut W,
    ) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                Message::Text(content) => {
                    writer
                        .write_all(format!("+{content}\r\n").as_bytes())
                        .await?;
                }
                Message::Error(content) => {
                    writer
                        .write_all(format!("-{content}\r\n").as_bytes())
                        .await?;
                }
                Message::Integer(value) => {
                    writer.write_all(format!(":{value}\r\n").as_bytes()).await?;
                }
                Message::Binary(data) => {
                    writer
                        .write_all(format!("${}\r\n", data.len()).as_bytes())
                        .await?;
                    writer.write_all(data).await?;
                    writer.write_all(b"\r\n").await?;
                }
                Message::Null => {
                    writer.write_all(b"$-1\r\n").await?;
                }
                Message::Array(items) => {
                    writer
                        .write_all(format!("*{}\r\n", items.len()).as_bytes())
                        .await?;

                    for item in items {
                        item.write(writer).await?;
                    }
                }
            };

            writer.flush().await?;
            Ok(())
        })
    }
}

async fn read_until_crlf<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Vec<u8>, MiniRedisError> {
    let mut output = Vec::with_capacity(128);

    while !output.ends_with(b"\r\n") {
        output.push(reader.read_u8().await?);
    }

    output.truncate(output.len() - 2);
    Ok(output)
}
