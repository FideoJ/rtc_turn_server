use crate::error::*;

// ChannelData represents The ChannelData Message.
#[derive(Default, Debug)]
pub struct ChannelData {
    pub number: u64,
    pub raw: Vec<u8>,
}

impl ChannelData {
    // Reset resets Length, Data and Raw length.
    pub fn reset(&mut self) {
        self.raw.clear();
    }

    // Encode encodes ChannelData Message to Raw.
    pub fn encode(&mut self) {}

    // Decode decodes The ChannelData Message from Raw.
    pub fn decode(&mut self) -> Result<()> {
        Ok(())
    }

    // is_channel_data returns true if buf looks like the ChannelData Message.
    pub fn is_channel_data(buf: &[u8]) -> bool {
        true
    }
}
