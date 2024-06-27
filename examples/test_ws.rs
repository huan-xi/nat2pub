use std::env;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use btnat::reuse_socket::make_socket;


#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8081));
    let socket = make_socket(addr)?;

    /*  let url = Url::parse("tcp://www.qq.com:80")?;
      let addr = url.socket_addrs(|| None)?
          .into_iter()
          .next()
          .ok_or(anyhow::anyhow!("Failed to resolve address"))?;*/

    // let addr:SocketAddr = "qq.com:80".parse()?;
    let addr = "qq.com:80".to_socket_addrs()?
        .next()
        .ok_or(anyhow::anyhow!("Failed to resolve address"))?;
    let mut tcp = socket.connect(addr).await?;
    let mut buf = [0u8; 1024];
    loop {
        let start_time = std::time::Instant::now();
        while let Ok(n) = tcp.read(&mut buf).await {
            if n == 0 {
                break;
            }
        };
        let elapsed = start_time.elapsed();
        info!("elapsed: {:?}", elapsed);
        info!("断开重连");
    }
    Ok(())
}