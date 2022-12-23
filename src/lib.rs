/*!
# CDTOC

[![docs.rs](https://img.shields.io/docsrs/cdtoc.svg?style=flat-square&label=docs.rs)](https://docs.rs/cdtoc/)
[![changelog](https://img.shields.io/crates/v/cdtoc.svg?style=flat-square&label=changelog&color=9b59b6)](https://github.com/Blobfolio/cdtoc/blob/master/CHANGELOG.md)<br>
[![crates.io](https://img.shields.io/crates/v/cdtoc.svg?style=flat-square&label=crates.io)](https://crates.io/crates/cdtoc)
[![ci](https://img.shields.io/github/workflow/status/Blobfolio/cdtoc/Build.svg?style=flat-square&label=ci)](https://github.com/Blobfolio/cdtoc/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/cdtoc/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/cdtoc)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)
[![contributions welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&label=contributions)](https://github.com/Blobfolio/cdtoc/issues)



CDTOC is a simple Rust library for parsing and working with audio CD tables of contents, namely in the form of [CDTOC-style](https://forum.dbpoweramp.com/showthread.php?16705-FLAC-amp-Ogg-Vorbis-Storage-of-CDTOC&s=3ca0c65ee58fc45489103bb1c39bfac0&p=76686&viewfull=1#post76686) metadata tag values.

By default it can also generate disc IDs for services like [AccurateRip](http://accuraterip.com/), [CDDB](https://en.wikipedia.org/wiki/CDDB), [CUETools Database](http://cue.tools/wiki/CUETools_Database), and [MusicBrainz](https://musicbrainz.org/).

Each of these helpers are feature-gated — `accuraterip`, `cddb`, `ctdb`, and `musicbrainz` respectively — allowing you to selectively skip the overhead for anything you don't care about.

If you disable all of them, this crate is dependency-free.



## Examples

```
use cdtoc::Toc;

// From a CDTOC string.
let toc1 = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();

// From the raw parts.
let toc2 = Toc::from_parts(
    vec![150, 11563, 25174, 45863],
    None,
    55370,
).unwrap();

// Either way gets you to the same place.
assert_eq!(toc1, toc2);

// You can also get a CDTOC-style string back at any time:
assert_eq!(toc1.to_string(), "4+96+2D2B+6256+B327+D84A");
```



## Installation

Add `cdtoc` to your `dependencies` in `Cargo.toml`, like:

```ignore,toml
[dependencies]
cdtoc = "0.1.*"
```

The disc ID helpers require additional dependencies, so if you aren't using them, be sure to disable the default features (and add back any you _do_ want) to skip the overhead.

```ignore,toml
[dependencies.cdtoc]
version = "0.1.*"
default-features = false
```
*/

#![deny(unsafe_code)]

#![warn(
	clippy::filetype_is_file,
	clippy::integer_division,
	clippy::needless_borrow,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::suboptimal_flops,
	clippy::unneeded_field_pattern,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	non_ascii_idents,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
)]

#![allow(
	clippy::doc_markdown,
	clippy::module_name_repetitions,
)]

#![cfg_attr(feature = "docsrs", feature(doc_cfg))]



mod error;
#[cfg(feature = "accuraterip")] mod accuraterip;
#[cfg(feature = "cddb")] mod cddb;
#[cfg(feature = "ctdb")] mod ctdb;
#[cfg(feature = "musicbrainz")] mod musicbrainz;

pub use error::TocError;
#[cfg(feature = "accuraterip")] pub use accuraterip::AccurateRip;
#[cfg(feature = "cddb")] pub use cddb::Cddb;

use std::fmt;



#[derive(Debug, Clone, Eq, Hash, PartialEq)]
/// # CDTOC.
///
/// This struct holds a CD's parsed table of contents.
///
/// You can initialize it using a [CDTOC-style](https://forum.dbpoweramp.com/showthread.php?16705-FLAC-amp-Ogg-Vorbis-Storage-of-CDTOC&s=3ca0c65ee58fc45489103bb1c39bfac0&p=76686&viewfull=1#post76686) metadata value
/// via [`Toc::from_cdtoc`] or manually with [`Toc::from_parts`].
///
/// Once parsed, you can obtain things like the [number of audio tracks](Toc::audio_len),
/// their [sector positions](Toc::audio_sectors), information about the [session(s)](Toc::kind)
/// and so on.
///
/// Many online databases derive their unique disc IDs using tables of content
/// too. [`Toc`] can give you the following, provided the corresponding crate
/// feature(s) are enabled:
///
/// | Service | Feature | Method |
/// | ------- | ------- | ------ |
/// | [AccurateRip](http://accuraterip.com/) | `accuraterip` | [`Toc::accuraterip_id`] |
/// | [CDDB](https://en.wikipedia.org/wiki/CDDB) | `cddb` | [`Toc::cddb_id`] |
/// | [CUETools Database](http://cue.tools/wiki/CUETools_Database) | `ctdb` | [`Toc::ctdb_id`] |
/// | [MusicBrainz](https://musicbrainz.org/) | `musicbrainz` | [`Toc::musicbrainz_id`] |
///
/// If you don't care about any of those, import this crate with
/// `default-features = false` to skip the overhead.
///
/// ## Examples
///
/// ```
/// use cdtoc::Toc;
///
/// // From a CDTOC string.
/// let toc1 = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
///
/// // From the raw parts.
/// let toc2 = Toc::from_parts(
///     vec![150, 11563, 25174, 45863],
///     None,
///     55370,
/// ).unwrap();
///
/// // Either way gets you to the same place.
/// assert_eq!(toc1, toc2);
///
/// // You can also get a CDTOC-style string back at any time:
/// assert_eq!(toc1.to_string(), "4+96+2D2B+6256+B327+D84A");
/// ```
pub struct Toc {
	kind: TocKind,
	audio: Vec<u32>,
	data: u32,
	leadout: u32,
}

impl fmt::Display for Toc {
	#[allow(unsafe_code)]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Start with the track count.
		write!(f, "{:X}", self.audio.len())?;

		// Then the audio sectors.
		for v in &self.audio { write!(f, "+{v:X}")?; }

		// And finally some combination of data and leadout.
		match self.kind {
			TocKind::Audio => write!(f, "+{:X}", self.leadout),
			TocKind::CDExtra => write!(f, "+{:X}+{:X}", self.data, self.leadout),
			TocKind::DataFirst => write!(f, "+{:X}+X{:X}", self.leadout, self.data),
		}
	}
}

impl Toc {
	/// # From CDTOC Metadata Tag.
	///
	/// Instantiate a new [`Toc`] from a CDTOC metadata tag value, of the
	/// format described [here](https://forum.dbpoweramp.com/showthread.php?16705-FLAC-amp-Ogg-Vorbis-Storage-of-CDTOC&s=3ca0c65ee58fc45489103bb1c39bfac0&p=76686&viewfull=1#post76686).
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if the tag value is improperly formatted, the
	/// audio track count is outside `1..=99`, there are too many or too few
	/// sectors, or the sectors are ordered incorrectly.
	pub fn from_cdtoc<S>(src: S) -> Result<Self, TocError>
	where S: AsRef<str> {
		let (audio, data, leadout) = parse_cdtoc_metadata(src.as_ref())?;
		Self::from_parts(audio, data, leadout)
	}

	/// # From Parts.
	///
	/// Instantiate a new [`Toc`] by manually specifying the (starting) sectors
	/// for each audio track, data track (if any), and the leadout.
	///
	/// If a data track is supplied, it must fall between the last audio track
	/// and leadout, or come before either.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_parts(
	///     vec![150, 11563, 25174, 45863],
	///     None,
	///     55370,
	/// ).unwrap();
	///
	/// assert_eq!(toc.to_string(), "4+96+2D2B+6256+B327+D84A");
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if the audio track count is outside `1..=99`
	/// or the sectors are in the wrong order.
	pub fn from_parts(audio: Vec<u32>, data: Option<u32>, leadout: u32)
	-> Result<Self, TocError> {
		if audio.is_empty() { return Err(TocError::NoAudio); }

		// Audio is out of order?
		let audio_len = audio.len();
		if
			(1 < audio_len && audio.windows(2).any(|pair| pair[1] <= pair[0])) ||
			leadout <= audio[audio_len - 1]
		{
			return Err(TocError::SectorOrder);
		}

		// Figure out the kind and validate the data sector.
		let kind =
			if let Some(d) = data {
				if d < audio[0] { TocKind::DataFirst }
				else if audio[audio_len - 1] < d && d < leadout {
					TocKind::CDExtra
				}
				else { return Err(TocError::SectorOrder); }
			}
			else { TocKind::Audio };

		Ok(Self { kind, audio, data: data.unwrap_or_default(), leadout })
	}

	/// # Set Media Kind.
	///
	/// This method can be used to override the table of content's derived
	/// media format.
	///
	/// This is weird, but might come in handy if you need to correct a not-
	/// quite-right CDTOC metadata tag value, such as one that accidentally
	/// included the data session in its leading track count or ordered the
	/// sectors of a data-audio CD sequentially.
	/// ```
	/// use cdtoc::{Toc, TocKind};
	///
	/// // This will be interpreted as audio-only.
	/// let mut toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	///
	/// // If the track count was wrong and it is really a mixed-mode CD-Extra
	/// // disc, this will fix it right up:
	/// assert!(toc.set_kind(TocKind::CDExtra).is_ok());
	/// assert_eq!(
	///     toc.to_string(),
	///     "3+96+2D2B+6256+B327+D84A",
	/// );
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if there aren't enough sectors or tracks for
	/// the new kind.
	pub fn set_kind(&mut self, kind: TocKind) -> Result<(), TocError> {
		match (self.kind, kind) {
			// The last "audio" track is really data.
			(TocKind::Audio, TocKind::CDExtra) => {
				let len = self.audio.len();
				if len == 1 { return Err(TocError::NoAudio); }
				self.data = self.audio.remove(len - 1);
			},
			// The first "audio" track is really data.
			(TocKind::Audio, TocKind::DataFirst) => {
				if self.audio.len() == 1 { return Err(TocError::NoAudio); }
				self.data = self.audio.remove(0);
			},
			// The "data" track is the really the last audio track.
			(TocKind::CDExtra, TocKind::Audio) => {
				self.audio.push(self.data);
				self.data = 0;
			},
			// The "data" track is the really the last audio track.
			(TocKind::DataFirst, TocKind::Audio) => {
				self.audio.insert(0, self.data);
				self.data = 0;
			},
			// Data should come first, not last.
			(TocKind::CDExtra, TocKind::DataFirst) => {
				// Move the old track to the end of the audio list and replace
				// with the first.
				self.audio.push(self.data);
				self.data = self.audio.remove(0);
			},
			// Data should come last, not first.
			(TocKind::DataFirst, TocKind::CDExtra) => {
				// Move the old track to the front of the audio list and
				// replace with the last.
				self.audio.insert(0, self.data);
				self.data = self.audio.remove(self.audio.len() - 1);
			},
			// Noop.
			_ => return Ok(()),
		}

		self.kind = kind;
		Ok(())
	}
}

impl Toc {
	#[must_use]
	/// # Audio Leadin.
	///
	/// Return the leadin of the audio session, sometimes called the "offset".
	/// In practice, this is just where the first audio track begins.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.audio_leadin(), 150);
	/// ```
	pub fn audio_leadin(&self) -> u32 { self.audio[0] }

	#[must_use]
	/// # Audio Leadout.
	///
	/// Return the leadout for the audio session. This is usually the same as
	/// [`Toc::leadout`], but for CD-Extra discs, the audio leadout is actually
	/// the start of the data, minus a gap of `11_400`.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.audio_leadout(), 55370);
	/// ```
	pub const fn audio_leadout(&self) -> u32 {
		if matches!(self.kind, TocKind::CDExtra) {
			self.data.saturating_sub(11_400)
		}
		else { self.leadout }
	}

	#[must_use]
	/// # Number of Audio Tracks.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.audio_len(), 4);
	/// ```
	pub fn audio_len(&self) -> usize { self.audio.len() }

	#[must_use]
	/// # Audio Sectors.
	///
	/// Return the starting positions of each audio track.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.audio_sectors(), &[150, 11563, 25174, 45863]);
	/// ```
	pub fn audio_sectors(&self) -> &[u32] { &self.audio }

	#[must_use]
	/// # Data Sector.
	///
	/// Return the starting position of the data track, if any.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// // No data here.
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.data_sector(), None);
	///
	/// // This CD-Extra has data, though!
	/// let toc = Toc::from_cdtoc("3+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.data_sector(), Some(45_863));
	/// ```
	pub const fn data_sector(&self) -> Option<u32> {
		if self.kind.has_data() { Some(self.data) }
		else { None }
	}

	#[must_use]
	/// # Has Data?
	///
	/// This returns `true` for mixed-mode CDs and `false` for audio-only ones.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.has_data(), false);
	///
	/// let toc = Toc::from_cdtoc("3+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.has_data(), true);
	/// ```
	pub const fn has_data(&self) -> bool { self.kind.has_data() }

	#[must_use]
	/// # CD Format.
	///
	/// This returns the [`TocKind`] corresponding to the table of contents,
	/// useful if you want to know whether or not the disc has a data session,
	/// and where it is in relation to the audio session.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::{Toc, TocKind};
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.kind(), TocKind::Audio);
	///
	/// let toc = Toc::from_cdtoc("3+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.kind(), TocKind::CDExtra);
	///
	/// let toc = Toc::from_cdtoc("3+2D2B+6256+B327+D84A+X96").unwrap();
	/// assert_eq!(toc.kind(), TocKind::DataFirst);
	/// ```
	pub const fn kind(&self) -> TocKind { self.kind }

	#[must_use]
	/// # Absolute Leadin.
	///
	/// Return the offset of the first track (no matter the session type).
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.leadin(), 150);
	/// ```
	pub fn leadin(&self) -> u32 {
		if matches!(self.kind, TocKind::DataFirst) { self.data }
		else { self.audio[0] }
	}

	#[must_use]
	/// # Absolute Leadout.
	///
	/// Return the disc leadout, regardless of whether it marks the end of the
	/// audio or data session.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.leadout(), 55_370);
	/// ```
	pub const fn leadout(&self) -> u32 { self.leadout }

	/// # Track Position.
	///
	/// This lets you know if a given track number for this disc would come
	/// first, last, fall somwhere in the middle, or stand alone (i.e. track
	/// one of one).
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::{Toc, TrackPosition};
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert!(toc.track_position(0).is_err());
	/// assert_eq!(toc.track_position(1), Ok(TrackPosition::First));
	/// assert_eq!(toc.track_position(2), Ok(TrackPosition::Middle));
	/// assert_eq!(toc.track_position(3), Ok(TrackPosition::Middle));
	/// assert_eq!(toc.track_position(4), Ok(TrackPosition::Last));
	/// assert!(toc.track_position(5).is_err());
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if the track number is out of range for the
	/// table of contents.
	pub fn track_position(&self, track: usize) -> Result<TrackPosition, TocError> {
		TrackPosition::try_from((track, self.audio_len()))
	}
}



#[derive(Debug, Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// # CD Format.
///
/// This enum is used to differentiate between audio-only and mixed-mode discs
/// because that ultimately determines the formatting of CDTOC metadata values
/// and various derived third-party IDs.
pub enum TocKind {
	#[default]
	/// # Audio-Only.
	Audio,

	/// # Mixed w/ Trailing Data Session.
	CDExtra,

	/// # Mixed w/ Leading Data Session.
	///
	/// This would only be possible with a weird homebrew CD-R; retail CDs
	/// place their data sessions at the end.
	DataFirst,
}

impl TocKind {
	#[must_use]
	/// # Has Data?
	///
	/// Returns `true` if the format is mixed-mode.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::TocKind;
	///
	/// // Yep!
	/// assert!(TocKind::CDExtra.has_data());
	/// assert!(TocKind::DataFirst.has_data());
	///
	/// // Nope!
	/// assert!(! TocKind::Audio.has_data());
	/// ```
	pub const fn has_data(self) -> bool {
		matches!(self, Self::CDExtra | Self::DataFirst)
	}
}



#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
/// # Track Position.
///
/// This enum is used to differentiate between first, middle, and final track
/// positions within the context of a given table of contents.
///
/// Variants of this type are returned by [`Toc::track_position`].
pub enum TrackPosition {
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
		impl TryFrom<($ty, $ty)> for TrackPosition {
			type Error = TocError;

			fn try_from(src: ($ty, $ty)) -> Result<Self, Self::Error> {
				if src.0 == 0 || src.1 < src.0 { Err(TocError::TrackPosition) }
				else if src.0 == 1 {
					if src.1 == 1 { Ok(Self::Only) }
					else { Ok(Self::First) }
				}
				else if src.0 == src.1 { Ok(Self::Last) }
				else { Ok(Self::Middle) }
			}
		}
	)+);
}
pos_tuple!(u8, u16, u32, u64, usize);

impl TrackPosition {
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
}



#[cfg(feature = "base64")]
#[allow(unsafe_code)]
/// # Base64 Encode.
///
/// Encode the slice with base64 and apply a few character substitutions.
fn base64_encode(src: &[u8]) -> String {
	let mut out = base64::encode(src);
	for b in unsafe { out.as_mut_vec() } {
		match *b {
			b'+' => { *b = b'.'; },
			b'/' => { *b = b'_'; },
			b'=' => { *b = b'-'; },
			_ => {},
		}
	}
	out
}

#[cfg(feature = "faster-hex")]
#[allow(unsafe_code)]
/// # HEX Encode u32.
///
/// This convenience wrapper uses faster-hex to encode a u32 to a buffer.
fn hex_u32(src: u32, buf: &mut [u8], upper: bool) {
	faster_hex::hex_encode(&src.to_be_bytes(), buf).unwrap();
	if upper { buf.make_ascii_uppercase(); }
}

#[allow(clippy::cast_possible_truncation)]
/// # Parse CDTOC Metadata.
///
/// This parses the audio track count and sector positions from a CDTOC-style
/// metadata tag value. It will return a parsing error if the formatting is
/// grossly wrong, but will not validate the sanity of the count/parts.
fn parse_cdtoc_metadata(mut src: &str) -> Result<(Vec<u32>, Option<u32>, u32), TocError> {
	// There shouldn't be anything other than HEX, + delimiters, and maybe an X
	// on the last part.
	src = src.trim();
	if ! src.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' | b'+' | b'x' | b'X')) {
		return Err(TocError::CDTOCChars);
	}

	// Split on plus.
	let mut split = src.split('+');

	// The number of audio tracks comes first.
	let audio_len = split.by_ref()
		.next()
		.and_then(|n| u8::from_str_radix(n, 16).ok())
		.map(usize::from)
		.ok_or(TocError::TrackCount)?;

	// Everything else should be a starting sector.
	let mut sectors: Vec<u32> = split
		.map(|n|
			u32::from_str_radix(n.trim_start_matches(|c| c == 'x' || c == 'X'), 16)
				.map_err(|_| TocError::SectorSize)
		)
		.collect::<Result<Vec<u32>, TocError>>()?;

	// Pop the last part, which is either leadout or data.
	let mut leadout = sectors.pop().ok_or(TocError::NoAudio)?;

	// Audio-only.
	let sectors_len = sectors.len();
	if sectors_len == audio_len { Ok((sectors, None, leadout)) }
	// Mixed-mode.
	else if sectors_len == audio_len + 1 {
		// Pop the next-last part, which again is either data or leadout.
		let mut data = sectors.pop().ok_or(TocError::NoAudio)?;
		if leadout < data { std::mem::swap(&mut leadout, &mut data); }
		Ok((sectors, Some(data), leadout))
	}
	// Incorrect sector count.
	else { Err(TocError::SectorCount(audio_len as u8, sectors.len())) }
}



#[cfg(test)]
mod tests {
	use super::*;
	use brunch as _;

	const CDTOC_AUDIO: &str = "B+96+5DEF+A0F2+F809+1529F+1ACB3+20CBC+24E14+2AF17+2F4EA+35BDD+3B96D";
	const CDTOC_EXTRA: &str = "A+96+3757+696D+C64F+10A13+14DA2+19E88+1DBAA+213A4+2784E+2D7AF+36F11";
	const CDTOC_DATA_AUDIO: &str = "A+3757+696D+C64F+10A13+14DA2+19E88+1DBAA+213A4+2784E+2D7AF+36F11+X96";

	#[test]
	/// # Test Audio-Only Parsing.
	fn t_audio() {
		let toc = Toc::from_cdtoc(CDTOC_AUDIO).expect("Unable to parse CDTOC_AUDIO.");
		let sectors = vec![
			150,
			24047,
			41202,
			63497,
			86687,
			109747,
			134332,
			151060,
			175895,
			193770,
			220125,
		];
		assert_eq!(toc.audio_len(), 11);
		assert_eq!(toc.audio_sectors(), &sectors);
		assert_eq!(toc.data_sector(), None);
		assert_eq!(toc.has_data(), false);
		assert_eq!(toc.kind(), TocKind::Audio);
		assert_eq!(toc.audio_leadin(), 150);
		assert_eq!(toc.audio_leadout(), 244077);
		assert_eq!(toc.leadin(), 150);
		assert_eq!(toc.leadout(), 244077);
		assert_eq!(toc.to_string(), CDTOC_AUDIO);

		// This should match when built with the equivalent parts.
		assert_eq!(
			Toc::from_parts(sectors, None, 244077),
			Ok(toc),
		);
	}

	#[test]
	/// # Test CD-Extra Parsing.
	fn t_extra() {
		let toc = Toc::from_cdtoc(CDTOC_EXTRA).expect("Unable to parse CDTOC_EXTRA.");
		let sectors = vec![
			150,
			14167,
			26989,
			50767,
			68115,
			85410,
			106120,
			121770,
			136100,
			161870,
		];
		assert_eq!(toc.audio_len(), 10);
		assert_eq!(toc.audio_sectors(), &sectors);
		assert_eq!(toc.data_sector(), Some(186287));
		assert_eq!(toc.has_data(), true);
		assert_eq!(toc.kind(), TocKind::CDExtra);
		assert_eq!(toc.audio_leadin(), 150);
		assert_eq!(toc.audio_leadout(), 174887);
		assert_eq!(toc.leadin(), 150);
		assert_eq!(toc.leadout(), 225041);
		assert_eq!(toc.to_string(), CDTOC_EXTRA);

		// This should match when built with the equivalent parts.
		assert_eq!(
			Toc::from_parts(sectors, Some(186287), 225041),
			Ok(toc),
		);
	}

	#[test]
	/// # Test Data-First Parsing.
	fn t_data_first() {
		let toc = Toc::from_cdtoc(CDTOC_DATA_AUDIO)
			.expect("Unable to parse CDTOC_DATA_AUDIO.");
		let sectors = vec![
			14167,
			26989,
			50767,
			68115,
			85410,
			106120,
			121770,
			136100,
			161870,
			186287,
		];
		assert_eq!(toc.audio_len(), 10);
		assert_eq!(toc.audio_sectors(), &sectors);
		assert_eq!(toc.data_sector(), Some(150));
		assert_eq!(toc.has_data(), true);
		assert_eq!(toc.kind(), TocKind::DataFirst);
		assert_eq!(toc.audio_leadin(), 14167);
		assert_eq!(toc.audio_leadout(), 225041);
		assert_eq!(toc.leadin(), 150);
		assert_eq!(toc.leadout(), 225041);
		assert_eq!(toc.to_string(), CDTOC_DATA_AUDIO);

		// This should match when built with the equivalent parts.
		assert_eq!(
			Toc::from_parts(sectors, Some(150), 225041),
			Ok(toc),
		);
	}

	#[test]
	/// # Test Metadata Failures.
	fn t_bad() {
		for i in [
			"A+96+3757+696D+C64F+10A13+14DA2+19E88+1DBAA+213A4+2784E+2D7AF+36F11+36F12",
			"A+96+3757+696D+C64F+10A13+14DA2+19E88+1DBAA+213A4+2784E",
			"0+96",
			"A+96+3757+696D+C64F+10A13+14DA2+19E88+2784E+1DBAA+213A4+2D7AF+36F11",
		] {
			assert!(Toc::from_cdtoc(i).is_err());
		}
	}

	#[test]
	/// # Test Kind Conversions.
	fn t_rekind() {
		// Start with audio.
		let mut toc = Toc::from_cdtoc(CDTOC_AUDIO)
			.expect("Unable to parse CDTOC_AUDIO.");

		// To CD-Extra.
		assert!(toc.set_kind(TocKind::CDExtra).is_ok());
		assert_eq!(toc.audio_len(), 10);
		assert_eq!(
			toc.audio_sectors(),
			&[
				150,
				24047,
				41202,
				63497,
				86687,
				109747,
				134332,
				151060,
				175895,
				193770,
			]
		);
		assert_eq!(toc.data_sector(), Some(220125));
		assert_eq!(toc.has_data(), true);
		assert_eq!(toc.kind(), TocKind::CDExtra);
		assert_eq!(toc.audio_leadin(), 150);
		assert_eq!(toc.audio_leadout(), 208725);
		assert_eq!(toc.leadin(), 150);
		assert_eq!(toc.leadout(), 244077);

		// Back again.
		assert!(toc.set_kind(TocKind::Audio).is_ok());
		assert_eq!(Toc::from_cdtoc(CDTOC_AUDIO).unwrap(), toc);

		// To data-audio.
		assert!(toc.set_kind(TocKind::DataFirst).is_ok());
		assert_eq!(toc.audio_len(), 10);
		assert_eq!(
			toc.audio_sectors(),
			&[
				24047,
				41202,
				63497,
				86687,
				109747,
				134332,
				151060,
				175895,
				193770,
				220125,
			]
		);
		assert_eq!(toc.data_sector(), Some(150));
		assert_eq!(toc.has_data(), true);
		assert_eq!(toc.kind(), TocKind::DataFirst);
		assert_eq!(toc.audio_leadin(), 24047);
		assert_eq!(toc.audio_leadout(), 244077);
		assert_eq!(toc.leadin(), 150);
		assert_eq!(toc.leadout(), 244077);

		// Back again.
		assert!(toc.set_kind(TocKind::Audio).is_ok());
		assert_eq!(Toc::from_cdtoc(CDTOC_AUDIO).unwrap(), toc);

		// Now test data-to-other-data conversions.
		toc = Toc::from_cdtoc(CDTOC_EXTRA)
			.expect("Unable to parse CDTOC_EXTRA.");
		let extra = toc.clone();
		let data_audio = Toc::from_cdtoc(CDTOC_DATA_AUDIO)
			.expect("Unable to parse CDTOC_DATA_AUDIO.");

		// To data-audio.
		assert!(toc.set_kind(TocKind::DataFirst).is_ok());
		assert_eq!(toc, data_audio);

		// And back again.
		assert!(toc.set_kind(TocKind::CDExtra).is_ok());
		assert_eq!(toc, extra);
	}
}
