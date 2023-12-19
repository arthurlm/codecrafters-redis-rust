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
