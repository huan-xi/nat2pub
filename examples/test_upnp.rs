use std::env;
use std::net::SocketAddr;
use igd::aio::search_gateway;
use igd::SearchOptions;
use log::info;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let mut options: SearchOptions = Default::default();
    let addr: SocketAddr = "192.168.68.1:1900".parse()?;
    options.broadcast_address = addr;
    let gateway = search_gateway(options).await?;
    info!("upnp gateway {:?}", gateway);

    Ok(())
}