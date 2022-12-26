/*!
# CDTOC: Errors
*/

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
}

impl fmt::Display for TocError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::CDDASampleCount => f.write_str("Invalid CDDA sample count."),
			Self::CDTOCChars => f.write_str("Invalid character(s), expecting only 0-9, A-F, +, and (rarely) X."),
			Self::Checksums => f.write_str("Unable to parse checksums."),
			Self::NoAudio => f.write_str("At least one audio track is required."),
			Self::NoChecksums => f.write_str("No checksums were present."),
			Self::SectorCount(expected, found) => write!(
				f, "Expected {} audio sectors, found {}.",
				expected, found,
			),
			Self::SectorOrder => f.write_str("Sectors are incorrectly ordered or overlap."),
			Self::SectorSize => f.write_str("Sector sizes may not exceed four bytes (u32)."),
			Self::TrackCount => f.write_str("The number of audio tracks must be between 1..=99."),
		}
	}
}

impl Error for TocError {}
