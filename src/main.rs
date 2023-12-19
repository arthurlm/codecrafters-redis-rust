use redis_starter_rust::{request::Request, response::Response};
use tokio::{
    io::{BufReader, BufWriter},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                tokio::task::spawn(handle_client(stream));
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}

async fn handle_client(stream: TcpStream) -> anyhow::Result<()> {
    let (reader, writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut buf_writer = BufWriter::new(writer);

    loop {
        let request = Request::read(&mut buf_reader).await?;

        let response = match request {
            Request::Ping => Response::Pong,
            Request::Echo(data) => Response::Echo(data),
            Request::UnhandledCommand => {
                Response::Error("BAD_CMD Invalid command received".to_string())
            }
        };

        response.write(&mut buf_writer).await?;
    }
}
