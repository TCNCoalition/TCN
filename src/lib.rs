use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};

const H_CEK_DOMAIN_SEP: &[u8; 5] = b"H_CEK";
const H_CEN_DOMAIN_SEP: &[u8; 5] = b"H_CEN";

pub struct ReportAuthorizationKey {
    // We don't store rvk explicitly because it's cached inside the SecretKey.
    rak: ed25519_zebra::SecretKey,
}

impl ReportAuthorizationKey {
    pub fn new<R: RngCore + CryptoRng>(rng: R) -> ReportAuthorizationKey {
        ReportAuthorizationKey {
            rak: ed25519_zebra::SecretKey::new(rng),
        }
    }

    pub fn initial_cek(&self) -> ContactEventKey {
        let rvk = ed25519_zebra::PublicKeyBytes::from(&self.rak);

        let cek_0 = {
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

        ContactEventKey {
            index: 0,
            rvk,
            cek: cek_0,
        }
    }
}

pub struct ContactEventKey {
    index: u16,
    rvk: ed25519_zebra::PublicKeyBytes,
    cek: [u8; 32],
}

impl ContactEventKey {
    pub fn index(&self) -> u16 {
        self.index
    }

    pub fn ratchet(self) -> Option<ContactEventKey> {
        let ContactEventKey { index, rvk, cek } = self;
        if let Some(next_index) = index.checked_add(1) {
            let next_cek = {
                let mut bytes = [0; 32];
                bytes.copy_from_slice(
                    &Sha256::default()
                        .chain(H_CEK_DOMAIN_SEP)
                        .chain(&cek)
                        .result()[..],
                );
                bytes
            };
            Some(ContactEventKey {
                rvk,
                index: next_index,
                cek: next_cek,
            })
        } else {
            None
        }
    }
}

pub struct ContactEventNumber(pub [u8; 16]);

impl<'a> From<&'a ContactEventKey> for ContactEventNumber {
    fn from(key: &'a ContactEventKey) -> ContactEventNumber {
        let mut bytes = [0; 16];
        bytes.copy_from_slice(
            &Sha256::default()
                .chain(H_CEN_DOMAIN_SEP)
                .chain(&key.index.to_le_bytes()[..])
                .chain(&key.cek)
                .result()[..],
        );
        ContactEventNumber(bytes)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
