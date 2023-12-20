use std::{collections::HashMap, fmt, io::Write, string::FromUtf8Error};

use tokio::io::{AsyncRead, AsyncReadExt};

use crate::error::MiniRedisError;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Rdb {
    pub version: u32,
    pub aux_redis_ver: Option<String>,
    pub aux_redis_bits: Option<String>,
    pub aux_ctime: Option<String>,
    pub aux_used_men: Option<String>,
    pub values: HashMap<RedisString, RedisString>,
    pub expiry: HashMap<RedisString, u64>,
}

impl Rdb {
    pub async fn read<R: AsyncRead + Unpin>(input: &mut R) -> Result<Self, MiniRedisError> {
        // Check magic number and version
        let mut magic = [0_u8; 5];
        input.read_exact(&mut magic).await?;
        if &magic != b"REDIS" {
            return Err(MiniRedisError::InvalidRdbMagicNumber);
        }

        let mut version_bytes = [0_u8; 4];
        input.read_exact(&mut version_bytes).await?;
        let version: u32 = std::str::from_utf8(&version_bytes)?.parse()?;

        let mut next_expire_ms = None;
        let mut output = Self {
            version,
            ..Default::default()
        };

        // Iter over each fields
        loop {
            let op_codes = input.read_u8().await?;
            match op_codes {
                // Auxiliary field
                0xFA => {
                    let key = RedisString::read(input).await?;
                    let value = RedisString::read(input).await?;
                    match key.as_slice() {
                        b"redis-ver" => {
                            output.aux_redis_ver = Some(value.try_into()?);
                        }
                        b"redis-bits" => {
                            output.aux_redis_bits = Some(value.try_into()?);
                        }
                        b"ctime" => {
                            output.aux_ctime = Some(value.try_into()?);
                        }
                        b"used-mem" => {
                            output.aux_used_men = Some(value.try_into()?);
                        }
                        _ => {
                            eprintln!("Ignoring aux field: {:?}", key);
                        }
                    }
                }
                // Select DB
                0xFE => {
                    let db_id = RedisString::read(input).await?;
                    assert_eq!(db_id, RedisString::new(b""), "Multi DB is not supported");
                }
                // Resize DB
                0xFB => {
                    let db_values_size = read_integer(input).await?;
                    let db_expiry_size = read_integer(input).await?;

                    output.values.reserve(db_values_size as usize);
                    output.expiry.reserve(db_expiry_size as usize);
                }
                // Expire time millis
                0xFC => {
                    let expire_at = input.read_u64_le().await?;
                    next_expire_ms = Some(expire_at);
                }
                // Expire time secs
                0xFD => {
                    let expire_at = input.read_u32_le().await?;
                    next_expire_ms = Some(expire_at as u64 * 1000);
                }

                // Key / values
                // type: String
                0x00 => {
                    let key = RedisString::read(input).await?;
                    let value = RedisString::read(input).await?;

                    if let Some(expire_at) = next_expire_ms.take() {
                        output.expiry.insert(key.clone(), expire_at);
                    }

                    output.values.insert(key, value);
                }
                // End of file
                0xFF => break,
                _ => {}
            }
        }

        Ok(output)
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct RedisString(Vec<u8>);

impl RedisString {
    pub fn new(input: &[u8]) -> Self {
        Self(input.to_vec())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub async fn read<R: AsyncRead + Unpin>(input: &mut R) -> Result<Self, MiniRedisError> {
        match LengthEncoding::read(input).await? {
            LengthEncoding::Fixed(len) => {
                let mut payload = vec![0_u8; len];
                input.read_exact(&mut payload).await?;
                Ok(Self(payload))
            }
            LengthEncoding::Int8 => {
                let value = input.read_u8().await?;
                let mut payload = Vec::with_capacity(3);
                write!(payload, "{value}").expect("Fail to write in memory number");
                Ok(Self(payload))
            }
            LengthEncoding::Int16 => {
                let value = input.read_u16_le().await?;
                let mut payload = Vec::with_capacity(5);
                write!(payload, "{value}").expect("Fail to write in memory number");
                Ok(Self(payload))
            }
            LengthEncoding::Int32 => {
                let value = input.read_u32_le().await?;
                let mut payload = Vec::with_capacity(10);
                write!(payload, "{value}").expect("Fail to write in memory number");
                Ok(Self(payload))
            }
        }
    }
}

impl TryFrom<RedisString> for String {
    type Error = FromUtf8Error;

    fn try_from(value: RedisString) -> Result<Self, Self::Error> {
        Self::from_utf8(value.0.to_vec())
    }
}

impl fmt::Debug for RedisString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = f.debug_tuple("RedisString");

        match std::str::from_utf8(&self.0) {
            Ok(value) => out.field(&value),
            Err(_) => out.field(&self.0),
        };

        out.finish()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LengthEncoding {
    Fixed(usize),
    Int8,
    Int16,
    Int32,
}

impl LengthEncoding {
    pub async fn read<R: AsyncRead + Unpin>(input: &mut R) -> Result<Self, MiniRedisError> {
        let b0 = input.read_u8().await?;

        match b0 & 0b1100_0000 {
            0b0000_0000 => Ok(Self::Fixed(b0 as usize)),
            0b0100_0000 => {
                let b1 = input.read_u8().await?;
                Ok(Self::Fixed(
                    u16::from_be_bytes([b0 & 0b0011_1111, b1]) as usize
                ))
            }
            0b1000_0000 => {
                let b1 = input.read_u8().await?;
                let b2 = input.read_u8().await?;
                let b3 = input.read_u8().await?;
                Ok(Self::Fixed(
                    u32::from_be_bytes([b0 & 0b0011_1111, b1, b2, b3]) as usize,
                ))
            }
            0b1100_0000 => match b0 & 0b0011_1111 {
                0 => Ok(Self::Int8),
                1 => Ok(Self::Int16),
                2 => Ok(Self::Int32),
                _ => Err(MiniRedisError::UnsupportedLengthEncoding),
            },
            _ => unreachable!("Bit mask did not works ?"),
        }
    }
}

pub async fn read_integer<R: AsyncRead + Unpin>(input: &mut R) -> Result<i64, MiniRedisError> {
    match LengthEncoding::read(input).await? {
        LengthEncoding::Fixed(x) => Ok(x as i64),
        LengthEncoding::Int8 => {
            let value = input.read_u8().await?;
            Ok(value.into())
        }
        LengthEncoding::Int16 => {
            let value = input.read_u16_le().await?;
            Ok(value.into())
        }
        LengthEncoding::Int32 => {
            let value = input.read_u32_le().await?;
            Ok(value.into())
        }
    }
}
