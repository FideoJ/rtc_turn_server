mod chandata;
mod error;
mod server;

use server::*;

use net2::UdpBuilder;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::signal;
use tokio::time::Duration;
use util::Conn;
use std::os::unix::io::AsRawFd;

#[tokio::main]
async fn main() -> error::Result<()> {
    env_logger::init();

    let num_cpus = num_cpus::get();
    println!("{}", num_cpus);
    let mut conns: Vec<Arc<dyn Conn + Send + Sync>> = Vec::new();
    for _ in 0..num_cpus {
        let conn = Arc::new(UdpSocket::from_std(
            UdpBuilder::new_v4()?
                .reuse_address(true)?
                .bind("0.0.0.0:13001")?,
        )?);
        println!("fd={}", conn.as_raw_fd());
        conns.push(conn);
    }
    let peer = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9802);

    let server = Server::new(config::ServerConfig {
        conns,
        peer,
        channel_bind_timeout: Duration::from_secs(10),
    })
    .await?;

    println!("Waiting for Ctrl-C...");
    signal::ctrl_c().await.expect("failed to listen for event");
    println!("\nClosing connection now...");
    server.close().await
}
