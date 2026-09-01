//! Frame packet codec for framing raw TCP byte streams into discrete L2 packet frames.

use bytes::{Buf, BytesMut};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("Packet length too small: {0}")]
    TooSmall(usize),
    #[error("Packet exceeds maximum allowed size ({actual} > {max})")]
    TooLarge { actual: usize, max: usize },
}

pub struct L2FrameCodec {
    max_frame_size: usize,
}

impl Default for L2FrameCodec {
    fn default() -> Self {
        Self::new(65535)
    }
}

impl L2FrameCodec {
    pub fn new(max_frame_size: usize) -> Self {
        Self { max_frame_size }
    }

    /// Attempts to extract the next packet payload from the incoming buffer.
    /// In Lineage 2, each packet begins with a 2-byte little-endian length that includes the 2 length bytes.
    pub fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Vec<u8>>, FrameError> {
        if src.len() < 2 {
            return Ok(None);
        }

        let length = u16::from_le_bytes([src[0], src[1]]) as usize;

        if length < 2 {
            return Err(FrameError::TooSmall(length));
        }

        if length > self.max_frame_size {
            return Err(FrameError::TooLarge {
                actual: length,
                max: self.max_frame_size,
            });
        }

        if src.len() < length {
            // Wait for remaining packet data
            return Ok(None);
        }

        // Consume header
        src.advance(2);
        let payload_len = length - 2;
        let payload = src.split_to(payload_len).to_vec();

        Ok(Some(payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec_frame_extraction() {
        let mut codec = L2FrameCodec::default();
        let mut buffer = BytesMut::new();

        // Frame: len = 6 (2 bytes header + 4 bytes payload [0x04, 0x01, 0x02, 0x03])
        buffer.extend_from_slice(&[0x06, 0x00, 0x04, 0x01, 0x02, 0x03]);

        let frame = codec.decode(&mut buffer).unwrap();
        assert!(frame.is_some());
        assert_eq!(frame.unwrap(), vec![0x04, 0x01, 0x02, 0x03]);
        assert!(buffer.is_empty());
    }
}
