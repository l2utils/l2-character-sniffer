//! Lineage 2 Packet Cryptography utilities.

use blowfish::Blowfish;
use byteorder::LE;
use cipher::{BlockDecrypt, KeyInit};

/// Handles dynamic packet decryption for Lineage 2 game clients/servers.
#[derive(Clone)]
pub struct L2Cryptor {
    key: [u8; 16],
    initialized: bool,
}

impl Default for L2Cryptor {
    fn default() -> Self {
        Self::new()
    }
}

impl L2Cryptor {
    /// Creates a new uninitialized cryptor.
    pub fn new() -> Self {
        Self {
            key: [0u8; 16],
            initialized: false,
        }
    }

    /// Sets the dynamic 8 or 16-byte key received in the `KeyPacket` (initial packet exchange).
    pub fn set_key(&mut self, initial_key: &[u8]) {
        let len = initial_key.len().min(16);
        self.key[..len].copy_from_slice(&initial_key[..len]);
        self.initialized = true;
    }

    /// Returns whether the cryptor is initialized with a key.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Decrypts packet payload in-place using the current dynamic key.
    pub fn decrypt(&mut self, raw_data: &mut [u8]) -> Result<(), String> {
        if !self.initialized {
            return Ok(());
        }

        // L2 XOR / Blowfish stream decryption
        let mut temp = 0u32;
        let mut key_offset = 0usize;

        for chunk in raw_data.chunks_mut(4) {
            if chunk.len() == 4 {
                let current = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let key_part = u32::from_le_bytes([
                    self.key[key_offset % 16],
                    self.key[(key_offset + 1) % 16],
                    self.key[(key_offset + 2) % 16],
                    self.key[(key_offset + 3) % 16],
                ]);

                let decrypted = current ^ key_part ^ temp;
                temp = current;
                chunk.copy_from_slice(&decrypted.to_le_bytes());
                key_offset += 4;
            }
        }

        // Advance the dynamic key
        self.advance_key(raw_data.len() as u32);
        Ok(())
    }

    fn advance_key(&mut self, size: u32) {
        let mut old = u32::from_le_bytes([self.key[8], self.key[9], self.key[10], self.key[11]]);
        old = old.wrapping_add(size);
        self.key[8..12].copy_from_slice(&old.to_le_bytes());
    }

    /// Decrypts a blowfish encrypted block (used for login and initial handshake).
    pub fn decrypt_blowfish_block(&self, block: &mut [u8; 8], key: &[u8]) -> Result<(), String> {
        let cipher = Blowfish::<LE>::new_from_slice(key)
            .map_err(|e| format!("Failed to create Blowfish cipher: {e}"))?;
        let block_generic = cipher::generic_array::GenericArray::from_mut_slice(block);
        cipher.decrypt_block(block_generic);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cryptor_init_and_decrypt() {
        let mut cryptor = L2Cryptor::new();
        assert!(!cryptor.is_initialized());

        let key = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        cryptor.set_key(&key);
        assert!(cryptor.is_initialized());

        let mut data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let original = data;
        cryptor.decrypt(&mut data).unwrap();
        assert_ne!(data, original);
    }
}
