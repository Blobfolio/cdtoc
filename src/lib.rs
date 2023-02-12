/*!
# CDTOC

[![docs.rs](https://img.shields.io/docsrs/cdtoc.svg?style=flat-square&label=docs.rs)](https://docs.rs/cdtoc/)
[![changelog](https://img.shields.io/crates/v/cdtoc.svg?style=flat-square&label=changelog&color=9b59b6)](https://github.com/Blobfolio/cdtoc/blob/master/CHANGELOG.md)<br>
[![crates.io](https://img.shields.io/crates/v/cdtoc.svg?style=flat-square&label=crates.io)](https://crates.io/crates/cdtoc)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/cdtoc/ci.yaml?label=ci&style=flat-square)](https://github.com/Blobfolio/cdtoc/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/cdtoc/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/cdtoc)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)
[![contributions welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&label=contributions)](https://github.com/Blobfolio/cdtoc/issues)



CDTOC is a simple Rust library for parsing and working with audio CD tables of contents, namely in the form of [CDTOC-style](https://forum.dbpoweramp.com/showthread.php?16705-FLAC-amp-Ogg-Vorbis-Storage-of-CDTOC&s=3ca0c65ee58fc45489103bb1c39bfac0&p=76686&viewfull=1#post76686) metadata values.

By default it can also generate disc IDs for services like [AccurateRip](http://accuraterip.com/), [CDDB](https://en.wikipedia.org/wiki/CDDB), [CUETools Database](http://cue.tools/wiki/CUETools_Database), and [MusicBrainz](https://musicbrainz.org/), but you can disable the corresponding crate feature(s) — `accuraterip`, `cddb`, `ctdb`, and `musicbrainz` respectively — to shrink the dependency tree if you don't need that functionality.



## Work In Progress

This library is a work-in-progress. If you notice any bugs, please open an [issue](https://github.com/Blobfolio/cdtoc/issues)!



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



## De/Serialization

The optional `serde` crate feature can be enabled to expose de/serialization implementations for this library's types:

| Type | Format | Notes |
| ---- | ------ | ----- |
| [`AccurateRip`] | `String` | |
| [`Cddb`] | `String` | |
| [`Duration`] | `u64` | |
| [`ShaB64`] | `String` | MusicBrainz and CTDB IDs. |
| [`Toc`] | `String` | |
| [`Track`] | `Map` | |
| [`TrackPosition`] | `String` | |



## Installation

Add `cdtoc` to your `dependencies` in `Cargo.toml`, like:

```ignore,toml
[dependencies]
cdtoc = "0.1.*"
```

The disc ID helpers require additional dependencies, so if you aren't using them, be sure to disable the default features (adding back any you _do_ want) to skip the overhead.

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

#![cfg_attr(docsrs, feature(doc_cfg))]



mod error;
mod time;
mod track;
#[cfg(feature = "accuraterip")] mod accuraterip;
#[cfg(feature = "cddb")] mod cddb;
#[cfg(feature = "ctdb")] mod ctdb;
#[cfg(feature = "musicbrainz")] mod musicbrainz;
#[cfg(feature = "serde")] mod serde;
#[cfg(all(feature = "sha1", feature = "base64"))] mod shab64;

pub use error::TocError;
pub use time::Duration;
pub use track::{
	Track,
	Tracks,
	TrackPosition,
};
#[cfg(feature = "accuraterip")] pub use accuraterip::AccurateRip;
#[cfg(feature = "cddb")] pub use cddb::Cddb;
#[cfg(all(feature = "sha1", feature = "base64"))] pub use shab64::ShaB64;

use std::fmt;
use trimothy::TrimSlice;



/// # Powers of 16.
const BASE16: [u32; 8] = [1, 16, 256, 4096, 65_536, 1_048_576, 16_777_216, 268_435_456];

/// # Not Hex Placeholder Value.
const NIL: u8 = u8::MAX;

/// # Hex Decoding Table.
///
/// The `faster-hex` crate uses this table too, but doesn't expost it publicly,
/// unfortunately.
static UNHEX: [u8; 256] = [
	NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL,
	NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL,
	NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, NIL, NIL, NIL,
	NIL, NIL, NIL, NIL, 10, 11, 12, 13, 14, 15, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL,
	NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, 10, 11, 12, 13,
	14, 15, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL,
	NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL,
	NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL,
	NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL,
	NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL,
	NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL,
	NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL,
	NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL, NIL,
	NIL, NIL, NIL,
];

#[cfg(any(feature = "musicbrainz", feature = "ctdb"))]
/// # Lotsa Zeroes.
///
/// MusicBrainz and CTDB take a sha1 hash of 100 hex-encoded tracks, most of
/// which, most of the time, are just zero-padding. Slicing what we need out of
/// a prebuilt static is much faster than pushing zeroes on-the-fly.
static ZEROES: [u8; 792] = [b'0'; 792];



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
	#[cfg(not(feature = "faster-hex"))]
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

	#[cfg(feature = "faster-hex")]
	#[allow(unsafe_code, clippy::cast_possible_truncation)]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use trimothy::TrimSliceMatches;

		let mut out = Vec::with_capacity(128);
		let mut buf = [b'0'; 8];

		// Audio track count.
		let audio_len = self.audio.len() as u8;
		faster_hex::hex_encode_fallback(&[audio_len], &mut buf[..2]);
		if 16 <= audio_len { out.push(buf[0]); }
		out.push(buf[1]);

		macro_rules! push {
			($v:expr) => (
				faster_hex::hex_encode_fallback($v.to_be_bytes().as_slice(), &mut buf);
				out.push(b'+');
				out.extend_from_slice(buf.trim_start_matches(|b| b == b'0'));
			);
		}

		// The sectors.
		for v in &self.audio { push!(v); }

		// And finally some combination of data and leadout.
		match self.kind {
			TocKind::Audio => { push!(self.leadout); },
			TocKind::CDExtra => {
				push!(self.data);
				push!(self.leadout);
			},
			TocKind::DataFirst => {
				push!(self.leadout);

				// Handle this manually since there's the weird X marker.
				faster_hex::hex_encode_fallback(self.data.to_be_bytes().as_slice(), &mut buf);
				out.push(b'+');
				out.push(b'X');
				out.extend_from_slice(buf.trim_start_matches(|b| b == b'0'));
			},
		}

		out.make_ascii_uppercase();
		// Safety: this is all ASCII.
		f.write_str(unsafe { std::str::from_utf8_unchecked(&out) })
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
	/// sectors, the leadin is less than `150`, or the sectors are ordered
	/// incorrectly.
	pub fn from_cdtoc<S>(src: S) -> Result<Self, TocError>
	where S: AsRef<str> {
		let (audio, data, leadout) = parse_cdtoc_metadata(src.as_ref().as_bytes())?;
		Self::from_parts(audio, data, leadout)
	}

	/// # From Durations.
	///
	/// This will attempt to create an audio-only [`Toc`] from the track
	/// durations. (Needless to say, this will only work if all tracks are
	/// present and in the right order!)
	///
	/// If you happen to know the disc's true leadin offset you can specify it,
	/// otherwise the "industry default" value of `150` will be assumed.
	///
	/// To create a mixed-mode [`Toc`] from scratch, use [`Toc::from_parts`]
	/// instead so you can specify the location of the data session.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::{Toc, Duration};
	///
	/// let toc = Toc::from_durations(
	///     [
	///         Duration::from(46650_u64),
	///         Duration::from(41702_u64),
	///         Duration::from(30295_u64),
	///         Duration::from(37700_u64),
	///         Duration::from(40050_u64),
	///         Duration::from(53985_u64),
	///         Duration::from(37163_u64),
	///         Duration::from(59902_u64),
	///     ],
	///     None,
	/// ).unwrap();
	/// assert_eq!(
	///     toc.to_string(),
	///     "8+96+B6D0+159B6+1D00D+26351+2FFC3+3D2A4+463CF+54DCD",
	/// );
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if the track count is outside `1..=99`, the
	/// leadin is less than 150, or the sectors overflow `u32`.
	pub fn from_durations<I>(src: I, leadin: Option<u32>) -> Result<Self, TocError>
	where I: IntoIterator<Item=Duration> {
		let mut last: u32 = leadin.unwrap_or(150);
		let mut audio: Vec<u32> = vec![last];
		for d in src {
			let next = u32::try_from(d.sectors())
				.ok()
				.and_then(|n| last.checked_add(n))
				.ok_or(TocError::SectorSize)?;
			audio.push(next);
			last = next;
		}

		let leadout = audio.remove(audio.len() - 1);
		Self::from_parts(audio, None, leadout)
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
	///
	/// // Sanity matters; the leadin, for example, can't be less than 150.
	/// assert!(Toc::from_parts(
	///     vec![0, 10525],
	///     None,
	///     15000,
	/// ).is_err());
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if the audio track count is outside `1..=99`,
	/// the leadin is less than `150`, or the sectors are in the wrong order.
	pub fn from_parts(audio: Vec<u32>, data: Option<u32>, leadout: u32)
	-> Result<Self, TocError> {
		// Check length.
		let audio_len = audio.len();
		if 0 == audio_len { return Err(TocError::NoAudio); }
		if 99 < audio_len { return Err(TocError::TrackCount); }

		// Audio leadin must be at least 150.
		if audio[0] < 150 { return Err(TocError::LeadinSize); }

		// Audio is out of order?
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

	/// # Set Audio Leadin.
	///
	/// Set the audio leadin, nudging all entries up or down accordingly (
	/// including data and leadout).
	///
	/// Note: this method cannot be used for data-first mixed-mode CDs.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::{Toc, TocKind};
	///
	/// let mut toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.audio_leadin(), 150);
	///
	/// // Bump it up to 182.
	/// assert!(toc.set_audio_leadin(182).is_ok());
	/// assert_eq!(toc.audio_leadin(), 182);
	/// assert_eq!(
	///     toc.to_string(),
	///     "4+B6+2D4B+6276+B347+D86A",
	/// );
	///
	/// // Back down to 150.
	/// assert!(toc.set_audio_leadin(150).is_ok());
	/// assert_eq!(toc.audio_leadin(), 150);
	/// assert_eq!(
	///     toc.to_string(),
	///     "4+96+2D2B+6256+B327+D84A",
	/// );
	///
	/// // For CD-Extra, the data track will get nudged too.
	/// toc = Toc::from_cdtoc("3+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.kind(), TocKind::CDExtra);
	/// assert_eq!(toc.audio_leadin(), 150);
	/// assert_eq!(toc.data_sector(), Some(45863));
	///
	/// assert!(toc.set_audio_leadin(182).is_ok());
	/// assert_eq!(toc.audio_leadin(), 182);
	/// assert_eq!(toc.data_sector(), Some(45895));
	///
	/// // And back again.
	/// assert!(toc.set_audio_leadin(150).is_ok());
	/// assert_eq!(toc.audio_leadin(), 150);
	/// assert_eq!(toc.data_sector(), Some(45863));
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if the leadin is less than `150`, the CD
	/// format is data-first, or the nudging causes the sectors to overflow
	/// `u32`.
	pub fn set_audio_leadin(&mut self, leadin: u32) -> Result<(), TocError> {
		use std::cmp::Ordering;

		if leadin < 150 { Err(TocError::LeadinSize) }
		else if matches!(self.kind, TocKind::DataFirst) {
			Err(TocError::Format(TocKind::DataFirst))
		}
		else {
			let current = self.audio_leadin();
			match leadin.cmp(&current) {
				// Nudge downward.
				Ordering::Less => {
					let diff = current - leadin;
					for v in &mut self.audio { *v -= diff; }
					if self.has_data() { self.data -= diff; }
					self.leadout -= diff;
				},
				// Nudge upward.
				Ordering::Greater => {
					let diff = leadin - current;
					for v in &mut self.audio {
						*v = v.checked_add(diff).ok_or(TocError::SectorSize)?;
					}
					if self.has_data() {
						self.data = self.data.checked_add(diff)
							.ok_or(TocError::SectorSize)?;
					}
					self.leadout = self.leadout.checked_add(diff)
						.ok_or(TocError::SectorSize)?;
				},
				// Noop.
				Ordering::Equal => {},
			}

			Ok(())
		}
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
	///
	/// ## Examples
	///
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

	#[allow(clippy::cast_possible_truncation)]
	#[must_use]
	/// # Audio Track.
	///
	/// Return the details of a given audio track on the disc, or `None` if the
	/// track number is out of range.
	pub fn audio_track(&self, num: usize) -> Option<Track> {
		let len = self.audio_len();
		if num == 0 || len < num { None }
		else {
			let from = self.audio[num - 1];
			let to =
				if num < len { self.audio[num] }
				else { self.audio_leadout() };

			Some(Track {
				num: num as u8,
				pos: TrackPosition::from((num, len)),
				from,
				to,
			})
		}
	}

	#[must_use]
	/// # Audio Tracks.
	///
	/// Return an iterator of [`Track`] details covering the whole album.
	pub fn audio_tracks(&self) -> Tracks<'_> {
		Tracks::new(&self.audio, self.audio_leadout())
	}

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

	#[must_use]
	/// # Duration.
	///
	/// Return the total duration of all audio tracks.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::{Duration, Toc};
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(
	///     toc.duration(),
	///     toc.audio_tracks().map(|t| t.duration()).sum(),
	/// );
	/// ```
	pub fn duration(&self) -> Duration {
		Duration::from(self.audio_leadout() - self.audio_leadin())
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

impl fmt::Display for TocKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl TocKind {
	#[must_use]
	/// # As Str.
	///
	/// Return the value as a string slice.
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::Audio => "audio-only",
			Self::CDExtra => "CD-Extra",
			Self::DataFirst => "data+audio",
		}
	}

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



#[allow(clippy::cast_lossless)]
#[inline]
// Decode One Digit.
fn decode1(src: u8) -> Option<u8> {
	let out = UNHEX[src as usize];
	if out == NIL { None }
	else { Some(out) }
}

/// # Hex Decode u32.
///
/// This is a slightly more performant implementation of [`u8::from_str_radix`].
/// (`faster-hex` doesn't really help at this tiny scale.)
fn hex_decode_u8(src: &[u8]) -> Option<u8> {
	match src.len() {
		1 => decode1(src[0]),
		2 => {
			let a = decode1(src[0])?;
			let b = decode1(src[1])?;
			Some(16 * a + b)
		},
		_ => None,
	}
}

/// # Hex Decode u32.
///
/// This is a slightly more performant implementation of [`u32::from_str_radix`].
/// (`faster-hex` doesn't really help at this tiny scale.)
fn hex_decode_u32(src: &[u8]) -> Option<u32> {
	if (1..=8).contains(&src.len()) {
		let mut out: u32 = 0;
		for (k, byte) in BASE16.into_iter().zip(src.iter().copied().rev()) {
			let digit = u32::from(decode1(byte)?);
			out += digit * k;
		}

		Some(out)
	}
	else { None }
}

/// # Parse CDTOC Metadata.
///
/// This parses the audio track count and sector positions from a CDTOC-style
/// metadata tag value. It will return a parsing error if the formatting is
/// grossly wrong, but will not validate the sanity of the count/parts.
fn parse_cdtoc_metadata(src: &[u8]) -> Result<(Vec<u32>, Option<u32>, u32), TocError> {
	let src = src.trim();
	let mut split = src.split(|b| b'+'.eq(b));

	// The number of audio tracks comes first.
	let audio_len = split.next()
		.and_then(hex_decode_u8)
		.ok_or(TocError::TrackCount)?;

	// We should have starting positions for just as many tracks.
	let sectors: Vec<u32> = split
		.by_ref()
		.take(usize::from(audio_len))
		.map(hex_decode_u32)
		.collect::<Option<Vec<u32>>>()
		.ok_or(TocError::SectorSize)?;

	// Make sure we actually do.
	let sectors_len = sectors.len();
	if 0 == sectors_len { return Err(TocError::NoAudio); }
	if sectors_len != usize::from(audio_len) {
		return Err(TocError::SectorCount(audio_len, sectors_len));
	}

	// There should be at least one more entry to mark the audio leadout.
	let last1 = split.next()
		.ok_or(TocError::SectorCount(audio_len, sectors_len - 1))?;
	let last1 = hex_decode_u32(last1).ok_or(TocError::SectorSize)?;

	// If there is yet another entry, we've got a mixed-mode disc.
	if let Some(last2) = split.next() {
		// Unlike the other values, this entry might have an x-prefix to denote
		// a non-standard data-first position.
		let last2 = hex_decode_u32(last2)
			.or_else(||
				last2.strip_prefix(b"X").or_else(|| last2.strip_prefix(b"x"))
					.and_then(hex_decode_u32)
			)
			.ok_or(TocError::SectorSize)?;

		// That should be that!
		let remaining = split.count();
		if remaining == 0 {
			// "last1" is data, "last2" is leadout.
			if last1 < last2 {
				Ok((sectors, Some(last1), last2))
			}
			// "last2" is data, "last1" is leadout.
			else {
				Ok((sectors, Some(last2), last1))
			}
		}
		// Too many sectors!
		else {
			Err(TocError::SectorCount(audio_len, sectors_len + remaining))
		}
	}
	// A typical audio-only CD.
	else { Ok((sectors, None, last1)) }
}



#[cfg(test)]
mod tests {
	use super::*;
	use brunch as _;
	use serde_json as _;

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

		// Let's also quickly test that a long TOC works gets the audio track
		// count right.
		let toc = Toc::from_cdtoc("20+96+33BA+5B5E+6C74+7C96+91EE+A9A3+B1AC+BEFC+D2E6+E944+103AC+11426+14B58+174E2+1A9F7+1C794+1F675+21AB9+24090+277DD+2A783+2D508+2DEAA+2F348+31F20+37419+3A463+3DC2F+4064B+43337+4675B+4A7C0")
			.expect("Long TOC failed.");
		assert_eq!(toc.audio_len(), 32);
		assert_eq!(
			toc.to_string(),
			"20+96+33BA+5B5E+6C74+7C96+91EE+A9A3+B1AC+BEFC+D2E6+E944+103AC+11426+14B58+174E2+1A9F7+1C794+1F675+21AB9+24090+277DD+2A783+2D508+2DEAA+2F348+31F20+37419+3A463+3DC2F+4064B+43337+4675B+4A7C0"
		);

		// And one more with a hexish track count.
		let toc = Toc::from_cdtoc("10+96+2B4E+4C51+6B3C+9E08+CD43+FC99+13A55+164B8+191C9+1C0FF+1F613+21B5A+23F70+27A4A+2C20D+2FC65").unwrap();
		assert_eq!(toc.audio_len(), 16);
		assert_eq!(
			toc.to_string(),
			"10+96+2B4E+4C51+6B3C+9E08+CD43+FC99+13A55+164B8+191C9+1C0FF+1F613+21B5A+23F70+27A4A+2C20D+2FC65"
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

	#[test]
	fn t_unhex8() {
		for i in 0..=u8::MAX {
			assert_eq!(hex_decode_u8(format!("{:x}", i).as_bytes()), Some(i));
			assert_eq!(hex_decode_u8(format!("{:X}", i).as_bytes()), Some(i));
			assert_eq!(hex_decode_u8(format!("{:02x}", i).as_bytes()), Some(i));
			assert_eq!(hex_decode_u8(format!("{:02X}", i).as_bytes()), Some(i));
		}
	}

	#[test]
	#[ignore = "(very long-running)"]
	fn t_unhexu32() {
		for i in 0..=u32::MAX {
			assert_eq!(hex_decode_u32(format!("{:x}", i).as_bytes()), Some(i));
			assert_eq!(hex_decode_u32(format!("{:X}", i).as_bytes()), Some(i));
			assert_eq!(hex_decode_u32(format!("{:08x}", i).as_bytes()), Some(i));
			assert_eq!(hex_decode_u32(format!("{:08X}", i).as_bytes()), Some(i));
		}
	}
}
