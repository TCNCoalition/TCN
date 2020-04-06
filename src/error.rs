use thiserror::Error;

/// Errors related to the TCN protocol.
#[derive(Error, Debug)]
pub enum Error {
    /// An unknown memo type was encountered while parsing a report.
    #[error("Unknown memo type {0}")]
    UnknownMemoType(u8),
    /// An underlying I/O error occurred while parsing data.
    #[error("I/O error {0}")]
    Io(#[from] std::io::Error),
    /// An oversized memo field was supplied when creating a report.
    #[error("Oversize memo field: {0} bytes")]
    OversizeMemo(usize),
    /// A report failed the source integrity check.
    #[error("Report verification failed")]
    ReportVerificationFailed,
}
