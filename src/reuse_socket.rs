use std::net::SocketAddr;
use tokio::net::TcpSocket;

/// 创建一个端口复用socket
pub fn make_socket(local_addr: SocketAddr) -> anyhow::Result<TcpSocket> {
    let socket = TcpSocket::new_v4()?;
    // allow to reuse the addr both for connect and listen
    #[cfg(target_family = "unix")]
    { socket.set_reuseport(true)?; }
    socket.set_reuseaddr(true)?;
    socket.bind(local_addr).unwrap();
    Ok(socket)
}
