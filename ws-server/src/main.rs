use std::env;
use std::net::SocketAddr;
use std::num::ParseIntError;
use futures_util::SinkExt;
use hyper::Server;
use hyper::service::{make_service_fn, service_fn};
use websocket_codec::{Message};
use ws_server::{AsyncClient, server_upgrade};

async fn on_client(mut client: AsyncClient) {
    let _ = client.send(Message::text("Hello, world!")).await;
    let _ = client.send(Message::close(None)).await;
}

#[derive(Debug)]
pub enum Error {
    Hyper(hyper::Error),
    ParseInt(ParseIntError),
    // WebSocket(websocket_lite::Error),
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::Hyper(e)
    }
}
impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Error::ParseInt(e)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let port = env::args().nth(1)
        .unwrap_or_else(|| "9001".to_owned()).parse()?;
    let addr: SocketAddr = ([0, 0, 0, 0], port).into();
    println!("Listening on: {}", addr);
    let make_service =
        make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(|req| server_upgrade(req, on_client))) });

    Server::bind(&addr).serve(make_service).await?;
    Ok(())
}