//! Reference implementation of **Temporary Contact Numbers**, a detcntralized,
//! privacy-first contact tracing protocol developed by [CoEpi] and [CovidWatch].
//! No personally-identifiable information is required by the
//! protocol, and although it is compatible with a trusted health authority, it
//! does not require one. Users' devices send short-range broadcasts over
//! Bluetooth to nearby devices. Later, a user who develops symptoms or tests
//! positive can report their status to their contacts with minimal loss of
//! privacy. Users who do not send reports reveal no information. Different
//! applications using the TCN protocol can interoperate, and the protocol can be
//! used with either verified test results or for self-reported symptoms via an
//! extensible report memo field.
//!
//! For more information on the protocol, see the [README].
//!
//! [CoEpi]: https://www.coepi.org/
//! [CovidWatch]: https://www.covid-watch.org/
//! [README]: https://github.com/Co-Epi/TCN/blob/main/README.md
//!
//! # Example
//!
//! ```
//! use tcn::*;
//! // Generate a report authorization key.  This key represents the capability
//! // to publish a report about a collection of derived temporary contact numbers.
//! let rak = ReportAuthorizationKey::new(rand::thread_rng());
//!
//! // Compute a list of temporary contact numbers by ratcheting the report's initial
//! // temporary contact key.
//! let mut tck = rak.initial_temporary_contact_key();
//! let mut tcns = Vec::new();
//! for _ in 0..100 {
//!     tcns.push(tck.temporary_contact_number());
//!     tck = tck.ratchet().unwrap();
//! }
//!
//! // Prepare a report about a subset of the temporary contact numbers.
//! let signed_report = rak
//!     .create_report(
//!         MemoType::CoEpiV1,        // The memo type
//!         b"symptom data".to_vec(), // The memo data
//!         20,                       // Index of the first TCN to disclose
//!         100,                      // Index of the last TCN to check
//!     )
//!     .expect("Report creation can only fail if the memo data is too long");
//!
//! // Verify the source integrity of the report...
//! let report = signed_report
//!     .verify()
//!     .expect("Valid reports should verify correctly");
//!
//! // ...allowing the disclosed TCNs to be recomputed.
//! let recomputed_tcns = report.temporary_contact_numbers().collect::<Vec<_>>();
//!
//! // Check that the recomputed TCNs match the originals.
//! assert_eq!(&recomputed_tcns[..], &tcns[20..100]);
//! ```

#![deny(missing_docs)]

mod error;
mod keys;
mod report;
mod serialize;

pub use error::Error;
pub use keys::{TemporaryContactKey, TemporaryContactNumber, ReportAuthorizationKey};
pub use report::{MemoType, Report, SignedReport};
