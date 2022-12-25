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
	/// # CDDASample Rate.
	CDDASampleCount,

	/// # Invalid characters.
	CDTOCChars,

	/// # Invalid Checksum File.
	Checksums,

	/// # No Audio.
	NoAudio,

	/// # No Checksums.
	NoChecksums,

	/// # Invalid sector count.
	SectorCount(u8, usize),

	/// # Sector Ordering.
	SectorOrder,

	/// # Sector Size.
	SectorSize,

	/// # Track Count.
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
