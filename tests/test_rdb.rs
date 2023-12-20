use std::collections::HashMap;

use redis_starter_rust::{
    error::MiniRedisError,
    rdb::{self, Rdb, RedisString},
};
use tokio::io::BufReader;

async fn make_rdb(input: &[u8]) -> Result<Rdb, MiniRedisError> {
    let mut reader = BufReader::new(input);
    Rdb::read(&mut reader).await
}

#[test]
fn test_debug() {
    assert_eq!(format!("{:?}", rdb::Rdb::default()), "Rdb { version: 0, aux_redis_ver: None, aux_redis_bits: None, aux_ctime: None, aux_used_men: None, values: {}, expiry: {} }");
    assert_eq!(format!("{:?}", rdb::LengthEncoding::Fixed(41)), "Fixed(41)");
}

#[tokio::test]
async fn test_invalid_magic() {
    assert_eq!(
        make_rdb(b"foo").await,
        Err(MiniRedisError::Io("early eof".to_string()))
    );
    assert_eq!(
        make_rdb(b"REDOX").await,
        Err(MiniRedisError::InvalidRdbMagicNumber)
    );
}

#[tokio::test]
async fn test_parse_empty() {
    assert_eq!(
        make_rdb(include_bytes!("./data/empty.rdb")).await,
        Ok(rdb::Rdb {
            version: 11,
            aux_redis_ver: Some("7.2.3".to_string()),
            aux_redis_bits: Some("64".to_string()),
            aux_ctime: Some("1703063427".to_string()),
            aux_used_men: Some("940152".to_string()),
            ..Default::default()
        })
    );
}

#[tokio::test]
async fn test_parse_single_key() {
    assert_eq!(
        make_rdb(include_bytes!("./data/single-key.rdb")).await,
        Ok(rdb::Rdb {
            version: 11,
            aux_redis_ver: Some("7.2.3".to_string()),
            aux_redis_bits: Some("64".to_string()),
            aux_ctime: Some("1703066385".to_string()),
            aux_used_men: Some("915400".to_string()),
            values: HashMap::from([(RedisString::new(b"foo"), RedisString::new(b"bar"))]),
            ..Default::default()
        })
    );
}

#[tokio::test]
async fn test_parse_multi_key_expire() {
    assert_eq!(
        make_rdb(include_bytes!("./data/multi-key-expire.rdb")).await,
        Ok(rdb::Rdb {
            version: 11,
            aux_redis_ver: Some("7.2.3".to_string()),
            aux_redis_bits: Some("64".to_string()),
            aux_ctime: Some("1703080200".to_string()),
            aux_used_men: Some("1012368".to_string()),
            values: HashMap::from([
                (RedisString::new(b"foo"), RedisString::new(b"bar")),
                (RedisString::new(b"k2"), RedisString::new(b"v2")),
            ]),
            expiry: HashMap::from([(RedisString::new(b"foo"), 1703081197600)]),
        })
    );
}
