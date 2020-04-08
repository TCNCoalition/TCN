use std::{
    convert::{TryFrom, TryInto},
    io::{self, Read},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use super::{Error, MemoType, Report, ReportAuthorizationKey, SignedReport, TemporaryContactKey};

/// Some convenience methods to add to Read.
trait ReadExt: io::Read + Sized {
    /// Convenience method to read a `[u8; 32]`.
    #[inline]
    fn read_32_bytes(&mut self) -> io::Result<[u8; 32]> {
        let mut bytes = [0; 32];
        self.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    /// Convenience method to read a `[u8; 64]`.
    #[inline]
    fn read_64_bytes(&mut self) -> io::Result<[u8; 64]> {
        let mut bytes = [0; 64];
        self.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    /// Convenience method to read a short vector with a 1-byte length tag.
    #[inline]
    fn read_compact_vec(&mut self) -> io::Result<Vec<u8>> {
        let len = self.read_u8()? as usize;
        let mut bytes = Vec::with_capacity(len);
        self.take(len as u64).read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}

impl<R: io::Read> ReadExt for R {}

impl TryFrom<u8> for MemoType {
    type Error = Error;

    fn try_from(t: u8) -> Result<MemoType, Self::Error> {
        match t {
            0 => Ok(MemoType::CoEpiV1),
            1 => Ok(MemoType::CovidWatchV1),
            t => Err(Error::UnknownMemoType(t)),
        }
    }
}

impl Report {
    /// Compute the size of the serialization, to allow preallocations.
    pub(crate) fn size_hint(&self) -> usize {
        32 + 32 + 2 + 2 + 1 + 1 + self.memo_data.len()
    }

    /// Try to read a `Report` from a generic `io::Read`er.
    pub fn read<R: std::io::Read>(mut reader: R) -> Result<Report, Error> {
        let report = Report {
            rvk: reader.read_32_bytes()?.into(),
            tck_bytes: reader.read_32_bytes()?,
            j_1: reader.read_u16::<LittleEndian>()?,
            j_2: reader.read_u16::<LittleEndian>()?,
            memo_type: reader.read_u8()?.try_into()?,
            memo_data: reader.read_compact_vec()?,
        };

        // Invariant: j_1 > 0
        if report.j_1 > 0 {
            Ok(report)
        } else {
            Err(Error::InvalidReportIndex)
        }
    }

    /// Try to write a `Report` into a generic `io::Write`er.
    ///
    /// This method fails only when the memo data is too long or in the event of
    /// an underlying I/O error.
    pub fn write<W: io::Write>(&self, mut writer: W) -> Result<(), Error> {
        let memo_len = u8::try_from(self.memo_data.len())
            .map_err(|_| Error::OversizeMemo(self.memo_data.len()))?;
        writer.write_all(&<[u8; 32]>::from(self.rvk))?;
        writer.write_all(&self.tck_bytes)?;
        writer.write_u16::<LittleEndian>(self.j_1)?;
        writer.write_u16::<LittleEndian>(self.j_2)?;
        writer.write_u8(self.memo_type as u8)?;
        writer.write_u8(memo_len)?;
        writer.write_all(&self.memo_data)?;
        Ok(())
    }
}

impl SignedReport {
    /// Try to read a `SignedReport` from a generic `io::Read`er.
    pub fn read<R: io::Read>(mut reader: R) -> Result<SignedReport, Error> {
        Ok(SignedReport {
            report: Report::read(&mut reader)?,
            sig: reader.read_64_bytes()?.into(),
        })
    }

    /// Try to write a `SignedReport` into a generic `io::Write`er.
    pub fn write<W: io::Write>(&self, mut writer: W) -> Result<(), Error> {
        self.report.write(&mut writer)?;
        writer.write_all(&<[u8; 64]>::from(self.sig)[..])?;
        Ok(())
    }
}

impl ReportAuthorizationKey {
    /// Try to read a `ReportAuthorizationKey` from a generic `io::Read`er.
    pub fn read<R: io::Read>(mut reader: R) -> Result<ReportAuthorizationKey, io::Error> {
        Ok(ReportAuthorizationKey {
            rak: reader.read_32_bytes()?.into(),
        })
    }

    /// Try to write a `ReportAuthorizationKey` into a generic `io::Write`er.
    pub fn write<W: io::Write>(&self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&<[u8; 32]>::from(self.rak))
    }
}

impl TemporaryContactKey {
    /// Try to read a `TemporaryContactKey` from a generic `io::Read`er.
    pub fn read<R: io::Read>(mut reader: R) -> Result<TemporaryContactKey, io::Error> {
        Ok(TemporaryContactKey {
            index: reader.read_u16::<LittleEndian>()?,
            rvk: reader.read_32_bytes()?.into(),
            tck_bytes: reader.read_32_bytes()?,
        })
    }

    /// Try to write a `TemporaryContactKey` into a generic `io::Write`er.
    pub fn write<W: io::Write>(&self, mut writer: W) -> Result<(), io::Error> {
        writer.write_u16::<LittleEndian>(self.index)?;
        writer.write_all(&<[u8; 32]>::from(self.rvk))?;
        writer.write_all(&self.tck_bytes)?;
        Ok(())
    }
}
