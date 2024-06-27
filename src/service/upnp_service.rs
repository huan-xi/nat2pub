// #![feature(const_socketaddr)]

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddrV4, TcpStream},
    time::Duration,
};
use std::net::SocketAddr;

use anyhow::{Error, Result};
use igd::{aio::search_gateway, AddPortError::PortInUse, Gateway, SearchOptions};
use log::{error, info};
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time;

#[derive(Error, Debug)]
pub enum UpnpError {
    #[error("upnp not support ipv6")]
    Ipv6,
}

pub struct UpnpOptions {
    pub name: String,
    pub inter_port: u16,
    pub duration: u32,
    pub sleep_seconds: u32,
    pub protocol: igd::PortMappingProtocol,
    pub broadcast_address: Option<SocketAddr>,
}


pub struct UpnpService {
    options: UpnpOptions,
    external_port: RwLock<Option<u16>>,
}

impl UpnpService {
    pub fn new(options: UpnpOptions) -> Self {
        Self {
            options,
            external_port: RwLock::new(None),
        }
    }

    pub async fn set_external_port(&self, port: u16) {
        let mut p = self.external_port.write().await;
        let old = p.clone();
        p.replace(port);

        if old.is_none() || old.unwrap() != port {
            if let Err(e) = self.add_port(port).await {
                error!("upnp error: {:?}", e);
            };
        };
    }

    pub async fn add_port(&self, port: u16) -> Result<(SocketAddrV4, Ipv4Addr)> {
        let mut options: SearchOptions = Default::default();
        if let Some(addr) = self.options.broadcast_address {
            options.broadcast_address = addr;
        }
        let gateway = search_gateway(options).await?;
        info!("upnp gateway {:?}", gateway);
        let gateway_addr = gateway.addr;
        let stream = TcpStream::connect(gateway_addr)?;
        let addr = stream.local_addr()?;
        //获取网关下的本地ip
        let ip = addr.ip();
        drop(stream);
        // let port = self.options.port;
        let name = self.options.name.as_str();
        let duration = self.options.duration;
        if let IpAddr::V4(ip) = ip {
            let mut retry = true;
            loop {
                //添加端口
                return match gateway
                    .add_port(
                        igd::PortMappingProtocol::TCP,
                        //外部端口
                        port,
                        SocketAddrV4::new(ip, self.options.inter_port),
                        duration,
                        name,
                    )
                    .await
                {
                    Err(err) => {
                        if let PortInUse = err {
                            if retry {
                                retry = false;
                                match gateway
                                    .remove_port(igd::PortMappingProtocol::TCP, port)
                                    .await
                                {
                                    Err(err) => {
                                        info!("upnp remove port {} error {}", port, err);
                                    }
                                    Ok(_) => {
                                        continue;
                                    }
                                }
                            }
                        }
                        //info!("upnp {} > {}", gateway_addr, err);
                        Err(err.into())
                    }
                    Ok(_) => {
                        Ok((gateway_addr, ip))
                    }
                };
            }
        } else {
            return Err(UpnpError::Ipv6.into());
        }
    }


    pub async fn start(&self) -> anyhow::Result<()> {
        let mut local_ip = Ipv4Addr::UNSPECIFIED;
        let sleep_seconds = self.options.sleep_seconds;
        let mut pre_gateway = SocketAddrV4::new(local_ip, 0);
        let duration = sleep_seconds + 60;
        let mut interval = time::interval(Duration::from_secs(duration.into()));
        loop {
            if let Some(port) = self.external_port.read().await.clone() {
                match self.add_port(port).await {
                    Ok((gateway, ip)) => {
                        if ip != local_ip || gateway != pre_gateway {
                            local_ip = ip;
                            pre_gateway = gateway;
                            info!("upnp add port {} > {}", gateway, ip);
                            // watch.ok(SocketAddrV4::new(ip, port), gateway);
                        }
                    }
                    Err(err) => {
                        local_ip = Ipv4Addr::UNSPECIFIED;
                        error!("upnp error: {:?}", err);
                    }
                }
            }

            interval.tick().await;
        }


        Ok(())
    }
}


