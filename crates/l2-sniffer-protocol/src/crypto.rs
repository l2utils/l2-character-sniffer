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

    /// Sets the dynamic 8 or 16-byte key received in the `KeyPacket` / `VersionCheck` (opcode 0x2E).
    pub fn set_key(&mut self, initial_key: &[u8]) {
        if initial_key.len() >= 8 {
            self.key[..8].copy_from_slice(&initial_key[..8]);
            // Standard Lineage 2 static key tail constant
            self.key[8..16].copy_from_slice(&[0xc8, 0x27, 0x93, 0x01, 0xa1, 0x6c, 0x31, 0x97]);
            self.initialized = true;
        } else if initial_key.len() == 16 {
            self.key.copy_from_slice(initial_key);
            self.initialized = true;
        }
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

        let mut temp = 0u8;
        for (i, byte) in raw_data.iter_mut().enumerate() {
            let temp2 = *byte;
            *byte = temp2 ^ self.key[i & 15] ^ temp;
            temp = temp2;
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
        let key_seed = [0x34, 0x67, 0xc4, 0x56, 0xe8, 0x76, 0x22, 0xd9];
        cryptor.set_key(&key_seed);

        let mut data = vec![
            0xca, 0xa0, 0x67, 0x31, 0xd8, 0xba, 0x98, 0x41, 0x89, 0xae, 0xc3, 0x3d, 0x63, 0xf0,
        ];
        cryptor.decrypt(&mut data).unwrap();
        assert_eq!(data[0], 0xfe);
    }
}
