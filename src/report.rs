use std::convert::TryFrom;

pub use super::{Error, ReportAuthorizationKey, TemporaryContactKey, TemporaryContactNumber};

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
    pub(crate) tck_bytes: [u8; 32],
    // Invariant: j_1 > 0.
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

    /// Return an iterator over all temporary contact numbers included in the report.
    pub fn temporary_contact_numbers(&self) -> impl Iterator<Item = TemporaryContactNumber> {
        let mut tck = TemporaryContactKey {
            // Does not underflow as j_1 > 0.
            index: self.j_1 - 1,
            rvk: self.rvk,
            tck_bytes: self.tck_bytes,
        };
        // Ratchet to obtain tck_{j_1}.
        tck = tck.ratchet().expect("j_1 - 1 < j_1 <= u16::MAX");

        (self.j_1..self.j_2).map(move |_| {
            let tcn = tck.temporary_contact_number();
            tck = tck
                .ratchet()
                .expect("we do not ratchet past j_2 <= u16::MAX");
            tcn
        })
    }
}

impl ReportAuthorizationKey {
    /// Create a report of potential exposure.
    ///
    /// # Inputs
    ///
    /// - `memo_type`, `memo_data`: the type and data for the report's memo field.
    /// - `j_1 > 0`: the ratchet index of the first temporary contact number in the report.
    /// - `j_2`: the ratchet index of the last temporary contact number other users should check.
    ///
    /// # Notes
    ///
    /// Creating a report reveals *all* temporary contact numbers subsequent to
    /// `j_1`, not just up to `j_2`, which is included for convenience.
    ///
    /// The `memo_data` must be less than 256 bytes long.
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
        // Ensure that j_1 is at least 1.
        let j_1 = if j_1 == 0 { 1 } else { j_1 };

        // Recompute tck_{j_1-1}. This requires recomputing j_1-1 hashes, but
        // creating reports is done infrequently and it means we don't force the
        // caller to have saved all intermediate hashes.
        let mut tck = self.initial_temporary_contact_key();
        // initial_temporary_contact_key returns tck_1, so begin iteration at 1.
        for _ in 1..(j_1 - 1) {
            tck = tck.ratchet().expect("j_1 - 1 < u16::MAX");
        }

        let report = Report {
            rvk: ed25519_zebra::PublicKeyBytes::from(&self.rak),
            tck_bytes: tck.tck_bytes,
            // Invariant: we have ensured j_1 > 0 above.
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
    pub(crate) report: Report,
    pub(crate) sig: ed25519_zebra::Signature,
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
