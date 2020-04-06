use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};

pub use super::Error;

const H_TCK_DOMAIN_SEP: &[u8; 5] = b"H_TCK";
const H_TCN_DOMAIN_SEP: &[u8; 5] = b"H_TCN";

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

    /// Compute the initial temporary contact key derived from this report authorization key.
    pub fn initial_temporary_contact_key(&self) -> TemporaryContactKey {
        let rvk = ed25519_zebra::PublicKeyBytes::from(&self.rak);

        let tck_bytes = {
            // There's a bit of an awkward dance to get digests into fixed-size
            // arrays because Rust doesn't have const generics (yet).
            let mut bytes = [0; 32];
            bytes.copy_from_slice(
                &Sha256::default()
                    .chain(H_TCK_DOMAIN_SEP)
                    .chain(&self.rak)
                    .result()[..],
            );
            bytes
        };

        TemporaryContactKey {
            index: 0,
            rvk,
            tck_bytes,
        }
    }
}

/// A pseudorandom 128-bit value broadcast to nearby devices over Bluetooth.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TemporaryContactNumber(pub [u8; 16]);

/// A ratcheting key used to derive temporary contact numbers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TemporaryContactKey {
    pub(crate) index: u16,
    pub(crate) rvk: ed25519_zebra::PublicKeyBytes,
    pub(crate) tck_bytes: [u8; 32],
}

impl TemporaryContactKey {
    /// The current ratchet index.
    pub fn index(&self) -> u16 {
        self.index
    }

    /// Compute the temporary contact number derived from this key.
    pub fn temporary_contact_number(&self) -> TemporaryContactNumber {
        let mut bytes = [0; 16];
        bytes.copy_from_slice(
            &Sha256::default()
                .chain(H_TCN_DOMAIN_SEP)
                .chain(&self.index.to_le_bytes()[..])
                .chain(&self.tck_bytes)
                .result()[..16],
        );
        TemporaryContactNumber(bytes)
    }

    /// Ratchet the key forward, producing a new key for a new temporary contact
    /// number.
    ///
    /// # Returns
    /// - `Some(new_key)` if the current ratchet index is less than `u16::MAX`;
    /// - `None` if the current ratchet index is `u16::MAX`, signaling that the
    ///   report authorization key should be rotated.
    pub fn ratchet(self) -> Option<TemporaryContactKey> {
        let TemporaryContactKey {
            index,
            rvk,
            tck_bytes,
        } = self;

        if let Some(next_index) = index.checked_add(1) {
            let next_tck_bytes = {
                let mut bytes = [0; 32];
                bytes.copy_from_slice(
                    &Sha256::default()
                        .chain(H_TCK_DOMAIN_SEP)
                        .chain(&tck_bytes)
                        .result()[..],
                );
                bytes
            };
            Some(TemporaryContactKey {
                rvk,
                index: next_index,
                tck_bytes: next_tck_bytes,
            })
        } else {
            None
        }
    }
}
