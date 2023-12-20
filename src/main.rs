use std::{env, os::unix::ffi::OsStrExt, path::PathBuf, sync::Arc};

use redis_starter_rust::{database::Database, request::Request, response::Response};
use tokio::{
    io::{BufReader, BufWriter},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    let dir = parse_cli_dir().unwrap_or_else(|| env::temp_dir());
    let dbfilename = parse_cli_dbfilename().unwrap_or_else(|| "dump.rdb".to_string());

    let config = Arc::new(Database::new());
    config.set(b"dir", dir.as_os_str().as_bytes()).await;
    config.set(b"dbfilename", dbfilename.as_bytes()).await;

    let database = Arc::new(Database::new());
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                tokio::task::spawn(handle_client(stream, database.clone(), config.clone()));
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}

fn parse_cli_dir() -> Option<PathBuf> {
    let index = env::args().position(|x| x == "--dir")?;
    let addr = env::args().nth(index + 1)?;
    addr.parse().ok()
}

fn parse_cli_dbfilename() -> Option<String> {
    let index = env::args().position(|x| x == "--dbfilename")?;
    let addr = env::args().nth(index + 1)?;
    Some(addr)
}

async fn handle_client(
    stream: TcpStream,
    db: Arc<Database>,
    config: Arc<Database>,
) -> anyhow::Result<()> {
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
            Request::SetExpire(key, value, ms_delta) => {
                db.set(&key, &value).await;
                db.expire_in(&key, ms_delta).await;
                Response::Ok
            }
            Request::ConfigGet(key) => match config.get(&key).await {
                Some(value) => Response::ConfigGet(key, value),
                None => Response::NoContent,
            },
            Request::UnhandledCommand => {
                Response::Error("BAD_CMD Invalid command received".to_string())
            }
        };

        response.write(&mut buf_writer).await?;
    }
}
