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
	/// # Invalid characters.
	CDTOCChars,

	/// # No Audio.
	NoAudio,

	/// # Invalid sector count.
	SectorCount(u8, usize),

	/// # Sector Ordering.
	SectorOrder,

	/// # Sector Size.
	SectorSize,

	/// # Track Count.
	TrackCount,

	/// # Track Position (Out of Range).
	TrackPosition,
}

impl fmt::Display for TocError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::CDTOCChars => f.write_str("Invalid character(s), expecting only 0-9, A-F, +, and (rarely) X."),
			Self::NoAudio => f.write_str("At least one audio track is required."),
			Self::SectorCount(expected, found) => write!(
				f, "Expected {} audio sectors, found {}.",
				expected, found,
			),
			Self::SectorOrder => f.write_str("Sectors are incorrectly ordered or overlap."),
			Self::SectorSize => f.write_str("Sector sizes may not exceed four bytes (u32)."),
			Self::TrackCount => f.write_str("The number of audio tracks must be between 1..=99."),
			Self::TrackPosition => f.write_str("The track is out of range."),
		}
	}
}

impl Error for TocError {}
