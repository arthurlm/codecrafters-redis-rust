use std::time::Duration;

use redis_starter_rust::database::Database;

#[tokio::test]
async fn test_database_get_set() {
    let database = Database::new();

    // Check invalid
    assert_eq!(database.get(b"foo").await, None);

    // Set and get
    database.set(b"foo", b"hello").await;
    assert_eq!(database.get(b"foo").await, Some(b"hello".to_vec()));

    // Update and get
    database.set(b"foo", b"world").await;
    assert_eq!(database.get(b"foo").await, Some(b"world".to_vec()));
}

#[tokio::test]
async fn test_database_expire_in() {
    let database = Database::new();

    // Check invalid
    assert_eq!(database.get(b"foo").await, None);

    // Set and get
    database.set(b"foo", b"hello").await;
    database.expire_in(b"foo", 100).await;
    assert_eq!(database.get(b"foo").await, Some(b"hello".to_vec()));

    // Wait and get
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(database.get(b"foo").await, None);
}
