use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};

pub use super::Error;

const H_CEK_DOMAIN_SEP: &[u8; 5] = b"H_CEK";
const H_CEN_DOMAIN_SEP: &[u8; 5] = b"H_CEN";

/// Authorizes publication of a report of potential exposure.
#[derive(Copy, Clone, Debug)]
pub struct ReportAuthorizationKey {
    // We don't store rvk explicitly because it's cached inside the SecretKey.
    pub(crate) rak: ed25519_zebra::SecretKey,
}

impl ReportAuthorizationKey {
    /// Initialize a new report authorization key from a random number generator.
    pub fn new<R: RngCore + CryptoRng>(rng: R) -> ReportAuthorizationKey {
        ReportAuthorizationKey {
            rak: ed25519_zebra::SecretKey::new(rng),
        }
    }

    /// Compute the initial contact event key derived from this report authorization key.
    pub fn initial_contact_event_key(&self) -> ContactEventKey {
        let rvk = ed25519_zebra::PublicKeyBytes::from(&self.rak);

        let cek_bytes = {
            // There's a bit of an awkward dance to get digests into fixed-size
            // arrays because Rust doesn't have const generics (yet).
            let mut bytes = [0; 32];
            bytes.copy_from_slice(
                &Sha256::default()
                    .chain(H_CEK_DOMAIN_SEP)
                    .chain(&self.rak)
                    .result()[..],
            );
            bytes
        };

        // Immediately ratchet cek_0 to return cek_1.
        ContactEventKey {
            index: 0,
            rvk,
            cek_bytes,
        }
        .ratchet()
        .expect("0 < u16::MAX")
    }
}

/// A pseudorandom 128-bit value broadcast to nearby devices over Bluetooth.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ContactEventNumber(pub [u8; 16]);

/// A ratcheting key used to derive contact event numbers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ContactEventKey {
    pub(crate) index: u16,
    pub(crate) rvk: ed25519_zebra::PublicKeyBytes,
    pub(crate) cek_bytes: [u8; 32],
}

impl ContactEventKey {
    /// The current ratchet index.
    pub fn index(&self) -> u16 {
        self.index
    }

    /// Compute the contact event number derived from this key.
    pub fn contact_event_number(&self) -> ContactEventNumber {
        let mut bytes = [0; 16];
        bytes.copy_from_slice(
            &Sha256::default()
                .chain(H_CEN_DOMAIN_SEP)
                .chain(&self.index.to_le_bytes()[..])
                .chain(&self.cek_bytes)
                .result()[..16],
        );
        ContactEventNumber(bytes)
    }

    /// Ratchet the key forward, producing a new key for a new contact event
    /// number.
    ///
    /// # Returns
    /// - `Some(new_key)` if the current ratchet index is less than `u16::MAX`;
    /// - `None` if the current ratchet index is `u16::MAX`, signaling that the
    /// report authorization key should be rotated.
    pub fn ratchet(self) -> Option<ContactEventKey> {
        let ContactEventKey {
            index,
            rvk,
            cek_bytes,
        } = self;

        if let Some(next_index) = index.checked_add(1) {
            let next_cek_bytes = {
                let mut bytes = [0; 32];
                bytes.copy_from_slice(
                    &Sha256::default()
                        .chain(H_CEK_DOMAIN_SEP)
                        .chain(&rvk)
                        .chain(&cek_bytes)
                        .result()[..],
                );
                bytes
            };
            Some(ContactEventKey {
                rvk,
                index: next_index,
                cek_bytes: next_cek_bytes,
            })
        } else {
            None
        }
    }
}
