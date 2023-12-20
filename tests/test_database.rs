use std::time::Duration;

use redis_starter_rust::{database::Database, rdb::RedisString};

#[tokio::test]
async fn test_database_get_set() {
    let database = Database::new();

    // Check invalid
    assert_eq!(database.get(b"foo").await, None);

    // Set and get
    database.set(b"foo", b"hello").await;
    assert_eq!(database.get(b"foo").await, Some(RedisString::new(b"hello")));

    // Update and get
    database.set(b"foo", b"world").await;
    assert_eq!(database.get(b"foo").await, Some(RedisString::new(b"world")));
}

#[tokio::test]
async fn test_database_expire_in() {
    let database = Database::new();

    // Check invalid
    assert_eq!(database.get(b"foo").await, None);

    // Set and get
    database.set(b"foo", b"hello").await;
    database.expire_in_millis(b"foo", 100).await;
    assert_eq!(database.get(b"foo").await, Some(RedisString::new(b"hello")));

    // Wait and get
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(database.get(b"foo").await, None);
}
