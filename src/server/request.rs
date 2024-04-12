use crate::chandata::ChannelData;
use crate::error::*;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::Duration;
use util::Conn;

// Request contains all the state needed to process a single incoming datagram
pub struct Request {
    // Current Request State
    pub conn: Arc<dyn Conn + Send + Sync>,
    pub relay_socket: Arc<dyn Conn + Send + Sync>,
    pub peer: SocketAddr,
    pub src_addr: SocketAddr,
    pub buff: Vec<u8>,

    // User Configuration
    pub channel_bind_timeout: Duration,
}

impl Request {
    // handle_request processes the give Request
    pub async fn handle_request(&mut self) -> Result<()> {
        // log::debug!(
        //     "received {} bytes of udp from {} on {}",
        //     self.buff.len(),
        //     self.src_addr,
        //     self.conn.local_addr().await?
        // );

        if ChannelData::is_channel_data(&self.buff) {
            self.handle_data_packet().await
        } else {
            Ok(())
        }
    }

    async fn handle_data_packet(&mut self) -> Result<()> {
        // log::debug!("received DataPacket from {}", self.src_addr);
        let mut c = ChannelData {
            raw: self.buff.clone(),
            ..Default::default()
        };
        c.decode()?;
        self.handle_channel_data(&c).await
    }

    pub(crate) async fn handle_channel_data(&mut self, c: &ChannelData) -> Result<()> {
        // log::debug!("received ChannelData from {}, peer={}", self.src_addr, self.peer);

        let l = self.relay_socket.send_to(&c.raw, self.peer).await?;
        if l != c.raw.len() {
            Err(Error::ErrShortWrite)
        } else {
            Ok(())
        }
    }
}
