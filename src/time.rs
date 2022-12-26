/*!
# CDTOC: Time
*/

use crate::TocError;
use dactyl::{
	NiceElapsed,
	traits::NiceInflection,
};
use std::{
	fmt,
	hash,
	iter::Sum,
	ops::{
		Add,
		AddAssign,
		Sub,
		SubAssign,
		Div,
		DivAssign,
		Mul,
		MulAssign,
	},
	time,
};



const SAMPLES_PER_SECTOR: u64 = 588;
const SECTORS_PER_SECOND: u64 = 75;



#[derive(Debug, Clone, Copy, Default, Ord, PartialOrd)]
/// # (CDDA Sector) Duration.
///
/// This struct holds a non-lossy — at least up to about 7.8 billion years —
/// CD sector duration (seconds + frames) for one or more tracks.
///
/// ## Examples
///
/// ```
/// use cdtoc::Toc;
///
/// let toc = Toc::from_cdtoc("9+96+5766+A284+E600+11FE5+15913+19A98+1E905+240CB+26280").unwrap();
/// let track = toc.audio_track(9).unwrap();
/// let duration = track.duration();
///
/// // The printable format is Dd HH:MM:SS+FF, though the day part is only
/// // present if non-zero.
/// assert_eq!(duration.to_string(), "00:01:55+04");
///
/// // The same as intelligible pieces:
/// assert_eq!(duration.dhmsf(), (0, 0, 1, 55, 4));
///
/// // If that's too many pieces, you can get just the seconds and frames:
/// assert_eq!(duration.seconds_frames(), (115, 4));
/// ```
///
/// The value can also be lossily converted to more familiar formats via
/// [`Duration::to_std_duration_lossy`] or [`Duration::to_f64_lossy`].
///
/// Durations can also be combined every which way, for example:
///
/// ```
/// use cdtoc::{Toc, Duration};
///
/// let toc = Toc::from_cdtoc("9+96+5766+A284+E600+11FE5+15913+19A98+1E905+240CB+26280").unwrap();
/// let duration: Duration = toc.audio_tracks()
///     .map(|t| t.duration())
///     .sum();
/// assert_eq!(duration.to_string(), "00:34:41+63");
/// ```
pub struct Duration(pub(crate) u64);

impl<T> Add<T> for Duration
where u64: From<T> {
	type Output = Self;
	fn add(self, other: T) -> Self { Self(self.0 + u64::from(other)) }
}

impl<T> AddAssign<T> for Duration
where u64: From<T> {
	fn add_assign(&mut self, other: T) { self.0 += u64::from(other); }
}

impl<T> Div<T> for Duration
where u64: From<T> {
	type Output = Self;
	fn div(self, other: T) -> Self {
		let other = u64::from(other);
		if other == 0 { Self(0) }
		else { Self(self.0.wrapping_div(other)) }
	}
}

impl<T> DivAssign<T> for Duration
where u64: From<T> {
	fn div_assign(&mut self, other: T) {
		let other = u64::from(other);
		if other == 0 { self.0 = 0; }
		else { self.0 = self.0.wrapping_div(other) };
	}
}

impl Eq for Duration {}

impl fmt::Display for Duration {
	#[allow(clippy::many_single_char_names)]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let (d, h, m, s, frames) = self.dhmsf();
		if d == 0 {
			write!(f, "{h:02}:{m:02}:{s:02}+{frames:02}")
		}
		else {
			write!(f, "{d}d {h:02}:{m:02}:{s:02}+{frames:02}")
		}
	}
}

impl From<u32> for Duration {
	fn from(src: u32) -> Self { Self(src.into()) }
}

impl From<u64> for Duration {
	fn from(src: u64) -> Self { Self(src) }
}

impl From<usize> for Duration {
	fn from(src: usize) -> Self { Self(src as u64) }
}

impl From<Duration> for u64 {
	fn from(src: Duration) -> Self { src.0 }
}

impl hash::Hash for Duration {
	fn hash<H: hash::Hasher>(&self, state: &mut H) { state.write_u64(self.0); }
}

impl PartialEq for Duration {
	fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl<T> Mul<T> for Duration
where u64: From<T> {
	type Output = Self;
	fn mul(self, other: T) -> Self { Self(self.0 * u64::from(other)) }
}

impl<T> MulAssign<T> for Duration
where u64: From<T> {
	fn mul_assign(&mut self, other: T) { self.0 *= u64::from(other); }
}

impl<T> Sub<T> for Duration
where u64: From<T> {
	type Output = Self;
	fn sub(self, other: T) -> Self { Self(self.0.saturating_sub(u64::from(other))) }
}

impl<T> SubAssign<T> for Duration
where u64: From<T> {
	fn sub_assign(&mut self, other: T) { self.0 = self.0.saturating_sub(u64::from(other)); }
}

impl Sum for Duration {
	fn sum<I>(iter: I) -> Self
	where I: Iterator<Item = Self> {
		iter.fold(Self::default(), |a, b| a + b)
	}
}

impl Duration {
	/// # From CDDA Samples.
	///
	/// Derive the duration from the total number of CDDA  — 16-bit stereo @
	/// 44.1 kHz — samples.
	///
	/// For tracks with non-CDDA bit depths, channel counts, sample rates, or
	/// sample totals, use [`Duration::from_samples`] instead.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Duration;
	///
	/// let duration = Duration::from_cdda_samples(5_073_852).unwrap();
	/// assert_eq!(
	///     duration.to_string(),
	///     "00:01:55+04",
	/// );
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if the sample count is not evenly divisible
	/// by `588`, the number of samples-per-sector for a standard audio CD.
	pub const fn from_cdda_samples(total_samples: u64) -> Result<Self, TocError> {
		let out = total_samples.wrapping_div(SAMPLES_PER_SECTOR);
		if total_samples % SAMPLES_PER_SECTOR == 0 { Ok(Self(out)) }
		else { Err(TocError::CDDASampleCount) }
	}

	#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
	#[must_use]
	/// # From Samples (Rescaled).
	///
	/// Derive the equivalent CDDA duration for a track with an arbitrary
	/// sample rate (i.e. not 44.1 kHz) or sample count.
	///
	/// This operation is potentially lossy and may result in a duration that
	/// is off by ±1 frame.
	///
	/// For standard CDDA tracks, use [`Duration::from_cdda_samples`] instead.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Duration;
	///
	/// let duration = Duration::from_samples(96_000, 17_271_098);
	/// assert_eq!(
	///     duration.to_string(),
	///     "00:02:59+68",
	/// );
	/// ```
	pub fn from_samples(sample_rate: u32,  total_samples: u64) -> Self {
		if sample_rate == 0 || total_samples == 0 { Self::default() }
		else {
			let sample_rate = u64::from(sample_rate);
			let (s, rem) = dactyl::div_mod(total_samples, sample_rate);
			if rem == 0 { Self(s * SECTORS_PER_SECOND) }
			else {
				let f = dactyl::int_div_float(rem * 75, sample_rate)
					.map_or(0, |f| f.trunc() as u64);
				Self(s * SECTORS_PER_SECOND + f)
			}
		}
	}
}

impl Duration {
	#[allow(clippy::many_single_char_names, clippy::cast_possible_truncation)]
	#[must_use]
	/// # Days, Hours, Minutes, Seconds, Frames.
	///
	/// Carve up the duration into a quintuple of days, hours, minutes,
	/// seconds, and frames.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("9+96+5766+A284+E600+11FE5+15913+19A98+1E905+240CB+26280").unwrap();
	/// let track = toc.audio_track(9).unwrap();
	/// assert_eq!(
	///     track.duration().dhmsf(),
	///     (0, 0, 1, 55, 4),
	/// );
	/// ```
	pub const fn dhmsf(self) -> (u64, u8, u8, u8, u8) {
		let (s, f) = self.seconds_frames();
		if s <= 4_294_967_295 {
			let (d, h, m, s) = NiceElapsed::dhms(s as u32);
			(d as u64, h, m, s, f)
		}
		else {
			let d = s.wrapping_div(86_400);
			let [h, m, s] = NiceElapsed::hms((s - d * 86_400) as u32);
			(d, h, m, s, f)
		}
	}

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
	///     track.duration().samples(),
	///     5_073_852,
	/// );
	/// ```
	pub const fn samples(self) -> u64 { self.0 * SAMPLES_PER_SECTOR }

	#[must_use]
	/// # Seconds + Frames.
	///
	/// Return the duration as a tuple containing the total number of seconds
	/// and remaining frames (some fraction of a second).
	///
	/// Audio CDs have 75 frames per second, so the frame portion will always
	/// be in the range of `0..75`.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("9+96+5766+A284+E600+11FE5+15913+19A98+1E905+240CB+26280").unwrap();
	/// let track = toc.audio_track(9).unwrap();
	/// assert_eq!(
	///     track.duration().seconds_frames(),
	///     (115, 4),
	/// );
	/// ```
	pub const fn seconds_frames(self) -> (u64, u8) {
		(self.0.wrapping_div(SECTORS_PER_SECOND), (self.0 % SECTORS_PER_SECOND) as u8)
	}

	#[must_use]
	/// # Number of Sectors.
	///
	/// Return the total number of sectors.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("9+96+5766+A284+E600+11FE5+15913+19A98+1E905+240CB+26280").unwrap();
	/// let track = toc.audio_track(9).unwrap();
	/// assert_eq!(
	///     track.duration().sectors(),
	///     8629,
	/// );
	/// ```
	pub const fn sectors(self) -> u64 { self.0 }

	#[allow(clippy::cast_precision_loss)]
	#[must_use]
	/// # To `f64` (Lossy).
	///
	/// Return the duration as a float (seconds.subseconds).
	///
	/// Given that 75ths don't always make the cleanest of fractions, there
	/// will likely be some loss in precision.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("9+96+5766+A284+E600+11FE5+15913+19A98+1E905+240CB+26280").unwrap();
	/// let track = toc.audio_track(9).unwrap();
	/// assert_eq!(
	///     track.duration().to_f64_lossy(),
	///     115.05333333333333,
	/// );
	/// ```
	pub fn to_f64_lossy(self) -> f64 {
		// Most durations will probably fit within `u32`, which converts
		// cleanly.
		if self.0 <= 4_294_967_295 { self.0 as f64 / 75.0 }
		// Otherwise let's try to do it in parts and hope for the best.
		else {
			let (s, f) = self.seconds_frames();
			s as f64 + f64::from(f) / 75.0
		}
	}

	#[must_use]
	/// # To [`std::time::Duration`] (Lossy).
	///
	/// Return the value as a "normal" [`std::time::Duration`].
	///
	/// Note that the `std` struct only counts time down to the nanosecond, so
	/// this value might be off by a few frames.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("9+96+5766+A284+E600+11FE5+15913+19A98+1E905+240CB+26280").unwrap();
	/// let track = toc.audio_track(9).unwrap();
	/// assert_eq!(
	///     track.duration().to_std_duration_lossy().as_nanos(),
	///     115_053_333_333,
	/// );
	/// ```
	pub fn to_std_duration_lossy(self) -> time::Duration {
		// There are 1_000_000_000 nanoseconds per 75 sectors. Reducing this to
		// 40_000_000:3 leaves less chance of temporary overflow.
		self.0.checked_mul(40_000_000)
			.map_or_else(
				|| {
					let (s, f) = self.seconds_frames();
					time::Duration::from_secs(s) +
					time::Duration::from_nanos((u64::from(f) * 40_000_000).wrapping_div(3))
				},
				|n| time::Duration::from_nanos(n.wrapping_div(3)),
			)
	}

	#[allow(clippy::many_single_char_names)]
	#[must_use]
	/// # To String Pretty.
	///
	/// Return a string reprsentation of the non-zero parts with English
	/// labels, separated Oxford-comma-style.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::{Toc, Duration};
	///
	/// let toc = Toc::from_cdtoc("9+96+5766+A284+E600+11FE5+15913+19A98+1E905+240CB+26280").unwrap();
	/// let track = toc.audio_track(9).unwrap();
	/// assert_eq!(
	///     track.duration().to_string_pretty(),
	///     "1 minute, 55 seconds, and 4 frames",
	/// );
	///
	/// // Empty durations look like this:
	/// assert_eq!(
	///     Duration::default().to_string_pretty(),
	///     "0 seconds",
	/// );
	/// ```
	pub fn to_string_pretty(self) -> String {
		let (d, h, m, s, f) = self.dhmsf();
		let mut parts: Vec<String> = Vec::new();
		if d != 0 { parts.push(d.nice_inflect("day", "days")); }
		for (num, single, plural) in [
			(h, "hour", "hours"),
			(m, "minute", "minutes"),
			(s, "second", "seconds"),
			(f, "frame", "frames"),
		] {
			if num != 0 { parts.push(num.nice_inflect(single, plural)); }
		}

		match parts.len() {
			0 => "0 seconds".to_owned(),
			1 => parts.remove(0),
			2 => parts.join(" and "),
			n => {
				let last = parts.remove(n - 1);
				let mut out = parts.join(", ");
				out.push_str(", and ");
				out.push_str(&last);
				out
			},
		}
	}
}
