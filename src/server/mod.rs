pub mod config;
mod request;

use crate::error::*;
use config::*;
use request::*;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{watch, Mutex};
use tokio::time::Duration;
use util::Conn;

const INBOUND_MTU: usize = 1500;

/// Server is an instance of the TURN Server
pub struct Server {
    channel_bind_timeout: Duration,
    shutdown_tx: Mutex<Option<watch::Sender<bool>>>,
}

impl Server {
    pub async fn new(config: ServerConfig) -> Result<Self> {
        config.validate()?;
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let s = Server {
            channel_bind_timeout: config.channel_bind_timeout,
            shutdown_tx: Mutex::new(Some(shutdown_tx)),
        };

        for (i, conn) in config.conns.into_iter().enumerate() {
            let relay_addr = format!("9.218.99.18:{}", 9801 + i);
            println!("{}", relay_addr);
            let relay_socket =
                Arc::new(UdpSocket::bind(relay_addr).await?);
            let shutdown_rx = shutdown_rx.clone();

            tokio::spawn(async move {
                Server::read_loop(
                    conn,
                    relay_socket,
                    config.peer,
                    s.channel_bind_timeout,
                    shutdown_rx,
                )
                .await;
            });
        }

        Ok(s)
    }

    async fn read_loop(
        conn: Arc<dyn Conn + Send + Sync>,
        relay_socket: Arc<dyn Conn + Send + Sync>,
        peer: SocketAddr,
        channel_bind_timeout: Duration,
        mut shutdown_rx: watch::Receiver<bool>,
    ) {
        let mut buf = vec![0u8; INBOUND_MTU];

        loop {
            let (n, addr) = tokio::select! {
                v = conn.recv_from(&mut buf) => {
                    match v {
                        Ok(v) => v,
                        Err(err) => {
                            log::debug!("exit read loop on error: {}", err);
                            break;
                        }
                    }
                },
                did_change = shutdown_rx.changed() => {
                    if did_change.is_err() || *shutdown_rx.borrow() {
                        // if did_change.is_err, sender was dropped, or if
                        // bool is set to true, that means we're shutting down.
                        break
                    } else {
                        continue;
                    }
                }
            };

            let mut r = Request {
                conn: Arc::clone(&conn),
                relay_socket: Arc::clone(&relay_socket),
                peer,
                src_addr: addr,
                buff: buf[..n].to_vec(),
                channel_bind_timeout,
            };

            if let Err(err) = r.handle_request().await {
                log::error!("error when handling datagram: {}", err);
            }
        }
        let _ = conn.close().await;
    }

    /// Close stops the TURN Server. It cleans up any associated state and closes all connections it is managing
    pub async fn close(&self) -> Result<()> {
        let mut shutdown_tx = self.shutdown_tx.lock().await;
        if let Some(tx) = shutdown_tx.take() {
            // errors if there are no receivers, but that's irrelevant.
            let _ = tx.send(true);
            // wait for all receivers to drop/close.
            tx.closed().await;
        }

        Ok(())
    }
}
