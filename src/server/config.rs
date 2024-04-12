use crate::error::*;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::Duration;
use util::Conn;

// ServerConfig configures the Pion TURN Server
pub struct ServerConfig {
    // conns are a list of all the turn listeners
    // Each listener can have custom behavior around the creation of Relays
    pub conns: Vec<Arc<dyn Conn + Send + Sync>>,

    // the static peer address
    pub peer: SocketAddr,

    // channel_bind_timeout sets the lifetime of channel binding.
    pub channel_bind_timeout: Duration,
}

impl ServerConfig {
    pub fn validate(&self) -> Result<()> {
        if self.conns.is_empty() {
            Err(Error::ErrNoAvailableConns)
        } else {
            Ok(())
        }
    }
}
