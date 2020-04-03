use std::convert::TryFrom;

pub use super::{ContactEventKey, ContactEventNumber, Error, ReportAuthorizationKey};

/// Describes the intended type of the contents of a memo field.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum MemoType {
    /// The CoEpi symptom self-report format, version 1 (TBD)
    CoEpiV1 = 0,
    /// The CovidWatch test data format, version 1 (TBD)
    CovidWatchV1 = 1,
    /// Reserved for future use.
    Reserved = 0xff,
}

/// A report of potential exposure.
#[derive(Clone, Debug)]
pub struct Report {
    pub(crate) rvk: ed25519_zebra::PublicKeyBytes,
    pub(crate) cek_bytes: [u8; 32],
    pub(crate) j_1: u16,
    pub(crate) j_2: u16,
    pub(crate) memo_type: MemoType,
    pub(crate) memo_data: Vec<u8>,
}

impl Report {
    /// Get the type of the memo field.
    pub fn memo_type(&self) -> MemoType {
        self.memo_type
    }

    /// Get the memo data.
    pub fn memo_data(&self) -> &[u8] {
        &self.memo_data
    }

    /// Return an iterator over all contact event numbers included in the report.
    pub fn contact_event_numbers(&self) -> impl Iterator<Item = ContactEventNumber> {
        let mut cek = ContactEventKey {
            index: self.j_1,
            rvk: self.rvk,
            cek_bytes: self.cek_bytes,
        };

        (self.j_1..self.j_2).map(move |_| {
            let cen = cek.contact_event_number();
            cek = cek
                .ratchet()
                .expect("we do not ratchet past j_2 <= u16::MAX");
            cen
        })
    }
}

impl ReportAuthorizationKey {
    /// Create a report of potential exposure.
    ///
    /// # Inputs
    ///
    /// - `memo_type`, `memo_data`: the type and data for the report's memo field.
    /// - `j_1`: the ratchet index of the first contact event number in the report.
    /// - `j_2`: the ratchet index of the last contact event number other users should check.
    ///
    /// # Notes
    ///
    /// Creating a report reveals *all* contact event numbers subsequent to
    /// `j_1`, not just up to `j_2`, which is included for convenience.
    ///
    /// The `memo_data` must be less than 256 bytes.
    ///
    /// Reports are unlinkable from each other **only up to the memo field**. In
    /// other words, adding the same high-entropy data to the memo fields of
    /// multiple reports will cause them to be linkable.
    pub fn create_report(
        &self,
        memo_type: MemoType,
        memo_data: Vec<u8>,
        j_1: u16,
        j_2: u16,
    ) -> Result<SignedReport, Error> {
        // Recompute cek_{j_1}. This requires recomputing j_1 hashes, but
        // creating reports is done infrequently and it means we don't force the
        // caller to have saved all intermediate hashes.
        let mut cek = self.initial_cek();
        for _ in 0..j_1 {
            cek = cek.ratchet().expect(
                "cek ratchet must be Some because we don't ratchet more than u16::MAX times",
            );
        }

        let report = Report {
            rvk: ed25519_zebra::PublicKeyBytes::from(&self.rak),
            cek_bytes: cek.cek_bytes,
            j_1,
            j_2,
            memo_type,
            memo_data,
        };

        use std::io::Cursor;
        let mut report_bytes = Vec::with_capacity(report.size_hint());
        report.write(Cursor::new(&mut report_bytes))?;
        let sig = self.rak.sign(&report_bytes);

        Ok(SignedReport { report, sig })
    }
}

/// A signed exposure report, whose source integrity can be verified to produce a `Report`.
#[derive(Clone, Debug)]
pub struct SignedReport {
    report: Report,
    sig: ed25519_zebra::Signature,
}

impl SignedReport {
    /// Verify the source integrity of this report, producing `Ok(Report)` if successful.
    pub fn verify(self) -> Result<Report, Error> {
        use std::io::Cursor;
        let mut report_bytes = Vec::with_capacity(self.report.size_hint());
        self.report.write(Cursor::new(&mut report_bytes))?;

        match ed25519_zebra::PublicKey::try_from(self.report.rvk)
            .and_then(|pk| pk.verify(&self.sig, &report_bytes))
        {
            Ok(_) => Ok(self.report),
            Err(_) => Err(Error::ReportVerificationFailed),
        }
    }
}
