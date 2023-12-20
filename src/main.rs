use std::{
    env,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::Arc,
};

use redis_starter_rust::{
    database::Database, error::MiniRedisError, rdb::Rdb, request::Request, response::Response,
};
use tokio::{
    fs,
    io::{BufReader, BufWriter},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    let dir = parse_cli_dir().unwrap_or_else(env::temp_dir);
    let dbfilename = parse_cli_dbfilename().unwrap_or_else(|| PathBuf::from("dump.rdb"));

    // Create DBs
    let config = Arc::new(Database::new());
    config.set(b"dir", dir.as_os_str().as_bytes()).await;
    config
        .set(b"dbfilename", dbfilename.as_os_str().as_bytes())
        .await;

    let database = Arc::new(Database::new());

    // Apply CLI args
    env::set_current_dir(&dir).expect("Fail to set current dir");
    if dbfilename.exists() {
        let rdb = read_rdb(&dbfilename).await.expect("Fail to read .rdb file");
        for (key, value) in rdb.values {
            database.set(key.as_slice(), value.as_slice()).await;
        }
        for (key, value) in rdb.expiry {
            database.expire_at_millis(key.as_slice(), value).await;
        }
    }

    // Startup server
    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .expect("Fail to start TCP server");

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
    let value = env::args().nth(index + 1)?;
    value.parse().ok()
}

fn parse_cli_dbfilename() -> Option<PathBuf> {
    let index = env::args().position(|x| x == "--dbfilename")?;
    let value = env::args().nth(index + 1)?;
    value.parse().ok()
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
            Request::Get(key) => match db.get(key).await {
                Some(data) => Response::Content(data),
                None => Response::NoContent,
            },
            Request::Set(key, value) => {
                db.set(key, value).await;
                Response::Ok
            }
            Request::SetExpire(key, value, ms_delta) => {
                db.set(key.clone(), value).await;
                db.expire_in_millis(key, ms_delta).await;
                Response::Ok
            }
            Request::Keys => {
                let keys = db.keys().await;
                Response::KeyMatches(keys)
            }
            Request::ConfigGet(key) => match config.get(key.clone()).await {
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

async fn read_rdb<P: AsRef<Path>>(path: P) -> Result<Rdb, MiniRedisError> {
    let file = fs::File::open(path).await?;
    let mut reader = BufReader::new(file);
    Rdb::read(&mut reader).await
}
