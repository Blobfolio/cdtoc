/*!
# CDTOC: Errors
*/

use crate::TocKind;
use std::{
	error::Error,
	fmt,
};



#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// # Error Type.
pub enum TocError {
	/// # CDDA Sample Rate.
	///
	/// The total number of samples for a given audio track on a CD must be
	/// evenly divisible by `588`, the number of samples per sector.
	CDDASampleCount,

	/// # Invalid characters.
	///
	/// CDTOC metadata tags comprise HEX-encoded decimals separated by `+`
	/// signs. The only other character allowed is an `X`, used to indicate a
	/// leading data session.
	CDTOCChars,

	/// # Invalid Checksum File.
	///
	/// This is a catch-all error used when an AccurateRip or CTDB checksum
	/// manifest contains some sort of logical error (i.e. preventing it being
	/// parsed).
	Checksums,

	/// # Invalid Format For Operation.
	///
	/// This is a catch-all error used when a given disc format is incompatible
	/// with the operation, such as [`TocKind::DataFirst`] w/ [`Toc::set_audio_leadin`](crate::Toc::set_audio_leadin).
	Format(TocKind),

	/// # Leadin Too Small.
	///
	/// Audio CDs require a leadin of at least `150`.
	LeadinSize,

	/// # No Audio.
	///
	/// At least one audio track is required for a table of contents.
	NoAudio,

	/// # No Checksums.
	///
	/// This error is used when an AccurateRip or CTDB checksum manifest yields
	/// no valid checksums.
	NoChecksums,

	/// # Invalid sector count.
	///
	/// The stated number of audio tracks should match the number of sectors
	/// provided (once data and leadout values have been separated).
	SectorCount(u8, usize),

	/// # Sector Ordering.
	///
	/// Audio CD sectors must be sequentially ordered and non-overlapping, and
	/// the data session, if any, must come either immediately before or after
	/// the audio set. The leadout must be larger than every other sector.
	SectorOrder,

	/// # Sector Size.
	///
	/// Sector values cannot exceed [`u32::MAX`].
	SectorSize,

	/// # Track Count.
	///
	/// Audio CDs support a maximum of 99 tracks.
	TrackCount,

	#[cfg(feature = "accuraterip")]
	/// # AccurateRip Decode.
	AccurateRipDecode,

	#[cfg(feature = "accuraterip")]
	/// # Drive Offset Decode.
	DriveOffsetDecode,

	#[cfg(feature = "accuraterip")]
	/// # No Drive Offsets.
	NoDriveOffsets,

	#[cfg(feature = "cddb")]
	/// # CDDB Decode.
	CddbDecode,

	#[cfg(feature = "sha1")]
	/// # SHA1/Base64 Decode.
	ShaB64Decode,
}

impl fmt::Display for TocError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Self::CDDASampleCount => "Invalid CDDA sample count.",
			Self::CDTOCChars => "Invalid character(s), expecting only 0-9, A-F, +, and (rarely) X.",
			Self::Checksums => "Unable to parse checksums.",
			Self::Format(kind) => return write!(f, "This operation can't be applied to {kind} discs."),
			Self::LeadinSize => "Leadin must be at least 150.",
			Self::NoAudio => "At least one audio track is required.",
			Self::NoChecksums => "No checksums were present.",
			Self::SectorCount(expected, found) => return write!(f, "Expected {expected} audio sectors, found {found}."),
			Self::SectorOrder => "Sectors are incorrectly ordered or overlap.",
			Self::SectorSize => "Sector sizes may not exceed four bytes (u32).",
			Self::TrackCount => "The number of audio tracks must be between 1..=99.",

			#[cfg(feature = "accuraterip")] Self::AccurateRipDecode => "Invalid AccurateRip ID string.",
			#[cfg(feature = "accuraterip")] Self::DriveOffsetDecode => "Unable to parse drive offsets.",
			#[cfg(feature = "accuraterip")] Self::NoDriveOffsets => "No drive offsets were found.",

			#[cfg(feature = "cddb")] Self::CddbDecode => "Invalid CDDB ID string.",
			#[cfg(feature = "sha1")] Self::ShaB64Decode => "Invalid sha/base64 ID string.",
		})
	}
}

impl Error for TocError {}
