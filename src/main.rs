use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
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

async fn handle_client(mut stream: TcpStream) -> anyhow::Result<()> {
    loop {
        let mut buf = vec![0_u8; 10];
        stream.read_buf(&mut buf).await?;

        stream.write_all(b"+PONG\r\n").await?;
        stream.flush().await?;
    }
}
