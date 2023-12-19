use std::sync::Arc;

use redis_starter_rust::{database::Database, request::Request, response::Response};
use tokio::{
    io::{BufReader, BufWriter},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    let database = Arc::new(Database::new());
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                tokio::task::spawn(handle_client(stream, database.clone()));
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}

async fn handle_client(stream: TcpStream, db: Arc<Database>) -> anyhow::Result<()> {
    let (reader, writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut buf_writer = BufWriter::new(writer);

    loop {
        let request = Request::read(&mut buf_reader).await?;

        let response = match request {
            Request::Ping => Response::Pong,
            Request::Echo(data) => Response::Echo(data),
            Request::Get(key) => match db.get(&key).await {
                Some(data) => Response::Content(data),
                None => Response::NoContent,
            },
            Request::Set(key, value) => {
                db.set(&key, &value).await;
                Response::Ok
            }
            Request::UnhandledCommand => {
                Response::Error("BAD_CMD Invalid command received".to_string())
            }
        };

        response.write(&mut buf_writer).await?;
    }
}
