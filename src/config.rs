/// 服务类型
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Service {
    /// 本地端口,范围,或者范围随机
    pub local_port: Option<String>,
    /// upnp 地址
    pub upnp_addr: Option<String>,
    /// 默认1900
    pub upnp_port: Option<u16>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// 服务
    pub services: Vec<Service>,
    ///stun 服务器,用于发现端口
    pub stun_server: Vec<String>,
    /// 保持连接服务器
    pub keep_alive_server: Vec<String>,
}