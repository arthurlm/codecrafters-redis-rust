use redis_starter_rust::{error::MiniRedisError, resp2::Message};
use tokio::io::{BufReader, BufWriter};

async fn decode(input: &[u8]) -> Message {
    let mut reader = BufReader::new(input);
    Message::read(&mut reader).await.unwrap()
}

async fn decode_err(input: &[u8]) -> MiniRedisError {
    let mut reader = BufReader::new(input);
    Message::read(&mut reader).await.unwrap_err()
}

macro_rules! eof_err {
    () => {
        MiniRedisError::Io("unexpected end of file".to_string())
    };
}

#[test]
fn test_debug() {
    assert_eq!(format!("{:?}", Message::text("hello")), "Text(\"hello\")");
}

#[tokio::test]
async fn test_empty() {
    assert_eq!(decode_err(b"").await, eof_err!());
}

#[tokio::test]
async fn test_invalid_message_type() {
    assert_eq!(
        decode_err(b"!e").await,
        MiniRedisError::InvalidMessageType('!')
    );
}

#[tokio::test]
async fn test_text() {
    // Valid
    assert_eq!(decode(b"+\r\n").await, Message::text(""));
    assert_eq!(decode(b"+\r\nHello").await, Message::text(""));
    assert_eq!(decode(b"+Hello\r\n").await, Message::text("Hello"));
    assert_eq!(decode(b"+Hello\r\nworld").await, Message::text("Hello"));

    // Invalid
    assert_eq!(decode_err(b"+Hell").await, eof_err!());
}

#[tokio::test]
async fn test_error() {
    // Valid
    assert_eq!(decode(b"-\r\n").await, Message::error(""));
    assert_eq!(decode(b"-\r\nHello").await, Message::error(""));
    assert_eq!(decode(b"-Hello\r\n").await, Message::error("Hello"));
    assert_eq!(decode(b"-Hello\r\nworld").await, Message::error("Hello"));

    // Invalid
    assert_eq!(decode_err(b"-Hell").await, eof_err!());
}

#[tokio::test]
async fn test_integer() {
    // Valid
    assert_eq!(decode(b":0\r\n").await, Message::Integer(0));
    assert_eq!(decode(b":42\r\n").await, Message::Integer(42));
    assert_eq!(decode(b":+42\r\n").await, Message::Integer(42));
    assert_eq!(decode(b":-42\r\n").await, Message::Integer(-42));

    // Invalid
    assert_eq!(decode_err(b":Hell").await, eof_err!());
    assert_eq!(
        decode_err(b":\r\n").await,
        MiniRedisError::InvalidNumber("cannot parse integer from empty string".to_string())
    );
    assert_eq!(
        decode_err(b":\r\nHello").await,
        MiniRedisError::InvalidNumber("cannot parse integer from empty string".to_string())
    );
    assert_eq!(
        decode_err(b":Hello\r\n").await,
        MiniRedisError::InvalidNumber("invalid digit found in string".to_string())
    );
    assert_eq!(
        decode_err(b":Hello\r\nworld").await,
        MiniRedisError::InvalidNumber("invalid digit found in string".to_string())
    );
}

#[tokio::test]
async fn test_binary() {
    // Valid null
    assert_eq!(decode(b"$-1\r\n").await, Message::Null);
    assert_eq!(decode(b"$0\r\n\r\n").await, Message::bin(&[]));
    assert_eq!(decode(b"$5\r\nhello\r\n").await, Message::bin(b"hello"));

    // Invalid size
    assert_eq!(decode_err(b"$Hell").await, eof_err!());
    assert_eq!(
        decode_err(b"$foo\r\n").await,
        MiniRedisError::InvalidNumber("invalid digit found in string".to_string())
    );
    assert_eq!(
        decode_err(b"$5\r\nhel").await,
        MiniRedisError::Io("early eof".to_string())
    );

    // Invalid end
    assert_eq!(
        decode_err(b"$5\r\nhelloxx").await,
        MiniRedisError::InvalidMessageEnd
    );
    assert_eq!(
        decode_err(b"$5\r\nhello\rx").await,
        MiniRedisError::InvalidMessageEnd
    );
}

#[tokio::test]
async fn test_array() {
    // Valid empty / null
    assert_eq!(decode(b"*-1\r\n").await, Message::Null);
    assert_eq!(decode(b"*0\r\n").await, Message::Array(vec![]));

    // Valid
    assert_eq!(
        decode(b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n").await,
        Message::Array(vec![Message::bin(b"hello"), Message::bin(b"world")])
    );
    assert_eq!(
        decode(b"*3\r\n:1\r\n:2\r\n:3\r\n").await,
        Message::Array(vec![
            Message::Integer(1),
            Message::Integer(2),
            Message::Integer(3),
        ])
    );
    assert_eq!(
        decode(b"*5\r\n:1\r\n:2\r\n:3\r\n:-4\r\n$5\r\nhello\r\n").await,
        Message::Array(vec![
            Message::Integer(1),
            Message::Integer(2),
            Message::Integer(3),
            Message::Integer(-4),
            Message::bin(b"hello"),
        ])
    );
}

#[tokio::test]
async fn test_write() {
    async fn check(msg: Message, expected: &str) {
        let mut buf = BufWriter::new(Vec::new());
        msg.write(&mut buf).await.unwrap();
        assert_eq!(buf.get_ref(), expected.as_bytes());
    }

    // Text
    check(Message::text(""), "+\r\n").await;
    check(Message::text("Hello"), "+Hello\r\n").await;

    // Error
    check(Message::error(""), "-\r\n").await;
    check(Message::error("Hello"), "-Hello\r\n").await;

    // Integer
    check(Message::Integer(0), ":0\r\n").await;
    check(Message::Integer(42), ":42\r\n").await;
    check(Message::Integer(-42), ":-42\r\n").await;

    // Binary
    check(Message::bin(b""), "$0\r\n\r\n").await;
    check(Message::bin(b"heLLo"), "$5\r\nheLLo\r\n").await;

    // Null
    check(Message::Null, "$-1\r\n").await;

    // Array
    check(Message::Array(vec![]), "*0\r\n").await;
    check(
        Message::Array(vec![Message::Integer(42), Message::Integer(-50)]),
        "*2\r\n:42\r\n:-50\r\n",
    )
    .await;
}
