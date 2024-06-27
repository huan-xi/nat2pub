use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::sync::Arc;
use log::{error, info};
use stun_format::Attr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time;
use crate::config;
use crate::reuse_socket::make_socket;
use crate::service::upnp_service::{UpnpOptions, UpnpService};

pub struct NatService {
    service_config: config::Service,
    /// 本机监听地址
    local_addr: SocketAddr,
    /// upnp 服务
    upnp_service: Arc<UpnpService>,
    keep_alive_server: String,
}

/// 启动stun 服务,监听ip变化
impl NatService {
    pub fn new(service_config: config::Service) -> anyhow::Result<Self> {
        let port = 28082;
        let local_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port));
        let broadcast_address = if let Some(addr) = service_config.upnp_addr.clone() {
            let addr: Ipv4Addr = addr.parse()?;
            Some(SocketAddr::V4(SocketAddrV4::new(addr, service_config.upnp_port.clone().unwrap_or(1900))))
        } else {
            None
        };

        let opts = UpnpOptions {
            name: "nat2pub".to_string(),
            inter_port: port,
            duration: 140,
            sleep_seconds: 120,
            protocol: igd::PortMappingProtocol::TCP,
            broadcast_address,
        };


        Ok(NatService {
            service_config,
            local_addr,
            upnp_service: Arc::new(UpnpService::new(opts)),
            keep_alive_server: "".to_string(),
        })
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let loc_addr = self.local_addr;
        // 启动upnp
        let upnp_service = self.upnp_service.clone();
        tokio::spawn(async move {
            if let Err(e) = upnp_service.start().await {
                error!("upnp error: {:?}", e);
            }
        });


        //获取公网端口
        let upnp_service = self.upnp_service.clone();
        tokio::spawn(async move {
            match self.stun_connect(upnp_service).await {
                Ok(Some(addr)) => {
                    match addr {
                        stun_format::SocketAddr::V4(ip, port) => {
                            info!("stun_get_port: {}.{}.{}.{} {}", ip[0], ip[1], ip[2], ip[3], port);
                            // upnp_service.set_external_port(port).await;
                            // upnp_service.set_external_port(28082).await;
                        }
                        stun_format::SocketAddr::V6(_, _) => {}
                    }
                }
                _ => {
                    error!("stun_get_port error");
                }
            };
            error!("stun disconnect");
        });


        //监听本地端口
        let listener = make_socket(loc_addr)?.listen(1024).unwrap();
        info!("Listening on: {}", loc_addr);
        loop {
            let conn = listener.accept().await;
            tokio::spawn(async move {
                match conn {
                    Ok((mut c, addr)) => {
                        println!("Accept: {:?}", addr);
                        let mut buf = [0u8; 28];
                        while let Ok(n) = c.read(&mut buf).await {
                            //收到数据
                            if n == 0 {
                                break;
                            };
                            let str = String::from_utf8_lossy(&buf[..n]);
                            println!("Received: {:?}", str);
                        }
                    }
                    Err(_) => {}
                }
            });
        }
        Ok(())
    }
    /// 启动upnp 服务
    async fn start_upnp_service() {}

    /// 获取公网端口
    async fn stun_connect(&self, upnp_service: Arc<UpnpService>) -> anyhow::Result<Option<stun_format::SocketAddr>> {
        let time = std::time::Instant::now();
        let socket = make_socket(self.local_addr)?;
        let stun_server: SocketAddr = "101.43.169.183:3478".parse().unwrap();
        let mut stream = socket.connect(stun_server).await?;
        info!("Connected to {}", stream.peer_addr().unwrap());
        info!("Local addr: {}", stream.local_addr().unwrap());

        let (mut reader, mut writer) = stream.split();
        let mut buf = [0u8; 28];
        let mut msg = stun_format::MsgBuilder::from(buf.as_mut_slice());
        msg.typ(stun_format::MsgType::BindingRequest).unwrap();
        msg.tid(1).unwrap();
        writer.write(msg.as_bytes()).await?;


        let mut buffer = [0; 1024];
        let mut mapped_addr = None;

        while let Ok(n) = reader.read(&mut buffer).await {
            // let bytes = buffer[..n].to_vec();
            let msg = stun_format::Msg::from(&buffer[..n]);
            for addr in msg.attrs_iter() {
                match addr {
                    Attr::MappedAddress(addr) => {
                        info!("MappedAddress: {:?}", addr);
                        mapped_addr = Some(addr);
                    }
                    Attr::ChangedAddress(addr) => {
                        info!("ChangedAddress: {:?}", addr);
                    }
                    Attr::XorMappedAddress(addr) => {
                        info!("XorMappedAddress: {:?}", addr);
                        mapped_addr = Some(addr);
                        upnp_service.set_external_port(28082).await;
                    }
                    _ => {}
                }
            }
        }
        info!("stun 服务器断开 ,总连接时长{:?}", time.elapsed());

        Ok(mapped_addr)
    }
}


async fn keep_alive(local_addr: SocketAddr, target: &str) {
    loop {
        info!("connect {target} to keep_alive");
        let res = keep_alive0(local_addr, target).await;
        match res {
            Ok(_) => {
                info!("keep_alive ok");
            }
            Err(e) => {
                error!("{target} disconnect error: {e}");
            }
        }
        time::sleep(time::Duration::from_secs(1)).await;
    }
}

async fn keep_alive0(local_addr: SocketAddr, target: &str) -> anyhow::Result<()> {
    let addr = target.to_socket_addrs()?
        .next()
        .ok_or(anyhow::anyhow!("Failed to resolve address"))?;

    let mut buf = [0u8; 1024];
    loop {
        let socket = make_socket(local_addr)?;
        let mut tcp = socket.connect(addr).await?;
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