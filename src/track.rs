/*!
# CDTOC: Track
*/

use crate::Duration;
use std::ops::Range;



#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
/// # Audio Track.
///
/// This struct holds the details for an audio track, allowing you to fetch
/// things like duration, sector positioning, etc.
///
/// It is the return value of [`Toc::audio_track`](crate::Toc::audio_track).
pub struct Track {
	pub(super) num: u8,
	pub(super) pos: TrackPosition,
	pub(super) from: u32,
	pub(super) to: u32,
}

impl Track {
	#[must_use]
	/// # Byte Size.
	///
	/// Return the equivalent RAW PCM byte size for this track.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("9+96+5766+A284+E600+11FE5+15913+19A98+1E905+240CB+26280").unwrap();
	/// let track = toc.audio_track(9).unwrap();
	/// assert_eq!(
	///     track.bytes(),
	///     20_295_408,
	/// );
	/// ```
	pub const fn bytes(self) -> u64 { self.sectors() as u64 * 2352 }

	#[must_use]
	/// # Duration.
	///
	/// Return the track duration (seconds + frames).
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// let track = toc.audio_track(1).unwrap();
	/// assert_eq!(track.duration().to_string(), "00:02:32+13");
	/// ```
	pub const fn duration(&self) -> Duration { Duration(self.sectors() as u64) }

	#[must_use]
	/// # MSF.
	///
	/// Return the (beginning) MSF — minutes, seconds, and frames — of the
	/// track.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// let track = toc.audio_track(2).unwrap();
	/// assert_eq!(track.msf(), (2, 34, 13));
	/// ```
	pub const fn msf(&self) -> (u32, u8, u8) { lba_to_msf(self.from) }

	#[must_use]
	/// # MSF (Normalized).
	///
	/// Return the (beginning) MSF — minutes, seconds, and frames — of the
	/// track, _without_ the mandatory 150-sector CD lead-in.
	///
	/// In other words, this value will always be two seconds less than
	/// [`Track::msf`].
	///
	/// Most applications expect a normalized MSF, so this is probably the
	/// version you'll want to use.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// let track = toc.audio_track(2).unwrap();
	/// assert_eq!(track.msf(), (2, 34, 13));
	/// assert_eq!(track.msf_normalized(), (2, 32, 13));
	/// ```
	pub const fn msf_normalized(&self) -> (u32, u8, u8) { lba_to_msf(self.from - 150) }

	#[must_use]
	/// # Number.
	///
	/// Return the track number.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.audio_track(2).unwrap().number(), 2_u8);
	/// ```
	pub const fn number(&self) -> u8 { self.num }

	#[must_use]
	/// # Disc Position.
	///
	/// Return whether or not this track appears first, last, or somewhere in
	/// between on the disc.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::{Toc, TrackPosition};
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.audio_track(1).unwrap().position(), TrackPosition::First);
	/// assert_eq!(toc.audio_track(2).unwrap().position(), TrackPosition::Middle);
	/// assert_eq!(toc.audio_track(3).unwrap().position(), TrackPosition::Middle);
	/// assert_eq!(toc.audio_track(4).unwrap().position(), TrackPosition::Last);
	/// ```
	pub const fn position(&self) -> TrackPosition { self.pos }

	#[must_use]
	/// # Total Samples.
	///
	/// Return the total number of samples.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("9+96+5766+A284+E600+11FE5+15913+19A98+1E905+240CB+26280").unwrap();
	/// let track = toc.audio_track(9).unwrap();
	/// assert_eq!(
	///     track.samples(),
	///     5_073_852,
	/// );
	/// ```
	pub const fn samples(self) -> u64 { self.duration().samples() }

	#[must_use]
	/// # Sector Size.
	///
	/// Return the number of sectors occupied by this track.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// let track = toc.audio_track(1).unwrap();
	/// assert_eq!(track.sectors(), 11_413_u32);
	/// ```
	pub const fn sectors(&self) -> u32 { self.to - self.from }

	#[must_use]
	/// # Sector Range.
	///
	/// Return the range of sectors — `start..end` — occupied by this track.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// let track = toc.audio_track(1).unwrap();
	/// assert_eq!(track.sector_range(), 150..11_563);
	///
	/// // If you just want the length, sectors() can get that more
	/// // directly, but it works out the same either way:
	/// assert_eq!(track.sector_range().len(), track.sectors() as usize);
	/// ```
	pub const fn sector_range(&self) -> Range<u32> { self.from..self.to }

	#[must_use]
	/// # Normalized Sector Range.
	///
	/// Return the range of sectors — `start..end` — occupied by this track,
	/// _without_ the mandatory 150-sector CD lead-in.
	///
	/// In other words, the start and end will be 150 less than the range
	/// reported by [`Track::sector_range`].
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// let track = toc.audio_track(1).unwrap();
	/// assert_eq!(track.sector_range(), 150..11_563);
	/// assert_eq!(track.sector_range_normalized(), 0..11_413);
	/// ```
	pub const fn sector_range_normalized(&self) -> Range<u32> {
		self.from - 150..self.to - 150
	}
}



#[derive(Debug)]
/// # Audio Tracks Iterator.
///
/// This is an iterator of [`Track`] details for a given [`Toc`](crate::Toc).
///
/// It is the return value of [`Toc::audio_tracks`](crate::Toc::audio_tracks).
pub struct Tracks<'a> {
	tracks: &'a [u32],
	leadout: u32,
	pos: usize,
}

impl Iterator for Tracks<'_> {
	type Item = Track;

	#[allow(clippy::cast_possible_truncation)]
	fn next(&mut self) -> Option<Self::Item> {
		let len = self.tracks.len();
		if len <= self.pos { return None; }

		let num = (self.pos + 1) as u8;
		let pos = TrackPosition::from((self.pos + 1, len));
		let from = self.tracks[self.pos];
		let to =
			if self.pos + 1 < len { self.tracks[self.pos + 1] }
			else { self.leadout };

		self.pos += 1;
		Some(Track { num, pos, from, to })
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.len();
		(len, Some(len))
	}
}

impl ExactSizeIterator for Tracks<'_> {
	fn len(&self) -> usize {
		self.tracks.len().saturating_sub(self.pos)
	}
}

impl<'a> Tracks<'a> {
	/// # New.
	pub(super) const fn new(tracks: &'a [u32], leadout: u32) -> Self {
		Self { tracks, leadout, pos: 0 }
	}
}





#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
/// # Track Position.
///
/// This enum is used to differentiate between first, middle, and final track
/// positions within the context of a given table of contents.
///
/// Variants of this type are returned by [`Track::position`].
pub enum TrackPosition {
	/// # Invalid.
	///
	/// This is used for track numbers that are out of range, including pre-gap
	/// hidden tracks (#0).
	Invalid,

	/// # The First Track.
	First,

	/// # Somewhere in the Middle.
	Middle,

	/// # The Last Track.
	Last,

	/// # The Only Track.
	Only,
}

macro_rules! pos_tuple {
	($($ty:ty),+) => ($(
		impl From<($ty, $ty)> for TrackPosition {
			fn from(src: ($ty, $ty)) -> Self {
				if src.0 == 0 || src.1 < src.0 { Self::Invalid }
				else if src.0 == 1 {
					if src.1 == 1 { Self::Only }
					else { Self::First }
				}
				else if src.0 == src.1 { Self::Last }
				else { Self::Middle }
			}
		}
	)+);
}

pos_tuple!(u8, u16, u32, u64, usize);

impl TrackPosition {
	#[must_use]
	/// # Is Valid?
	///
	/// Returns `true` if the position is anything other than [`TrackPosition::Invalid`].
	pub const fn is_valid(self) -> bool { ! matches!(self, Self::Invalid) }

	#[must_use]
	/// # Is First?
	///
	/// This returns `true` if the track appears at spot #1 on the disc.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::TrackPosition;
	///
	/// // Yep!
	/// assert!(TrackPosition::First.is_first());
	/// assert!(TrackPosition::Only.is_first());
	///
	/// // Nope!
	/// assert!(! TrackPosition::Middle.is_first());
	/// assert!(! TrackPosition::Last.is_first());
	/// ```
	pub const fn is_first(self) -> bool { matches!(self, Self::First | Self::Only) }

	#[must_use]
	/// # Is Last?
	///
	/// This returns `true` if the track appears at the end of the disc.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::TrackPosition;
	///
	/// // Yep!
	/// assert!(TrackPosition::Last.is_last());
	/// assert!(TrackPosition::Only.is_last());
	///
	/// // Nope!
	/// assert!(! TrackPosition::First.is_last());
	/// assert!(! TrackPosition::Middle.is_last());
	/// ```
	pub const fn is_last(self) -> bool { matches!(self, Self::Last | Self::Only) }

	#[must_use]
	/// # As Str.
	///
	/// Return the equivalent string slice for the variant.
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::Invalid => "Invalid",
			Self::First => "First",
			Self::Middle => "Middle",
			Self::Last => "Last",
			Self::Only => "Only",
		}
	}
}



#[allow(clippy::integer_division, clippy::cast_possible_truncation)]
/// # LBA to MSF.
///
/// Convert a logical block address (sectors) to minutes, seconds, and frames.
const fn lba_to_msf(sectors: u32) -> (u32, u8, u8) {
	// 75 sectors per second.
	let mut s = sectors / 75;
	let f = sectors - s * 75;

	// 60 seconds per minute.
	let m = s / 60;
	s -= m * 60;

	(m, s as u8, f as u8)
}
