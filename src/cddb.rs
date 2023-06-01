/*!
# CDTOC: CDDB
*/

use crate::{
	Toc,
	TocError,
};
use dactyl::traits::HexToUnsigned;
use std::{
	fmt,
	hash,
	str::FromStr,
};



#[cfg_attr(docsrs, doc(cfg(feature = "cddb")))]
#[derive(Debug, Clone, Copy)]
/// # CDDB ID.
///
/// This struct holds a [CDDB](https://en.wikipedia.org/wiki/CDDB) ID.
///
/// Values of this type are returned by [`Toc::cddb_id`].
///
/// ## Examples
///
/// ```
/// use cdtoc::Toc;
///
/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
/// let cddb_id = toc.cddb_id();
///
/// // Usually you'll want this value as a string:
/// assert_eq!(
///     cddb_id.to_string(),
///     "1f02e004",
/// );
///
/// // But you can also get it as a `u32`:
/// assert_eq!(
///     u32::from(cddb_id),
///     520_282_116,
/// );
/// ```
pub struct Cddb(pub(crate) u32);

impl Eq for Cddb {}

impl fmt::Display for Cddb {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut buf = [b'0'; 8];
		faster_hex::hex_encode_fallback(self.0.to_be_bytes().as_slice(), &mut buf);
		std::str::from_utf8(buf.as_slice())
			.map_err(|_| fmt::Error)
			.and_then(|s| f.write_str(s))
	}
}

impl FromStr for Cddb {
	type Err = TocError;
	#[inline]
	fn from_str(src: &str) -> Result<Self, Self::Err> { Self::decode(src) }
}

impl hash::Hash for Cddb {
	fn hash<H: hash::Hasher>(&self, state: &mut H) { state.write_u32(self.0); }
}

impl PartialEq for Cddb {
	fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl From<Cddb> for u32 {
	#[inline]
	fn from(src: Cddb) -> Self { src.0 }
}

impl From<&Toc> for Cddb {
	#[allow(clippy::cast_possible_truncation)]
	fn from(src: &Toc) -> Self {
		let mut len = src.audio_len();
		let mut a: u32 = 0;

		// Add the audio positions.
		let mut buf = itoa::Buffer::new();
		for v in src.audio_sectors() {
			for b in buf.format(v.wrapping_div(75)).bytes() {
				a += u32::from(b ^ b'0');
			}
		}

		// Add the data position.
		if let Some(v) = src.data_sector() {
			len += 1;
			for b in buf.format(v.wrapping_div(75)).bytes() {
				a += u32::from(b ^ b'0');
			}
		}

		// The three parts we need.
		let a = (a % 255) as u8;
		let b = ((src.leadout().wrapping_div(75) - src.leadin().wrapping_div(75)) as u16).to_be_bytes();
		let c = len as u8;

		// Shove it into a single u32.
		Self(u32::from_be_bytes([
			a,
			b[0], b[1],
			c,
		]))
	}
}

impl TryFrom<&str> for Cddb {
	type Error = TocError;
	#[inline]
	fn try_from(src: &str) -> Result<Self, Self::Error> { Self::decode(src) }
}

impl Cddb {
	/// # Decode.
	///
	/// Convert a CDDB ID string back into a [`Cddb`] instance.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::{Cddb, Toc};
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// let cddb_id = toc.cddb_id();
	/// let cddb_str = cddb_id.to_string();
	/// assert_eq!(cddb_str, "1f02e004");
	/// assert_eq!(Cddb::decode(cddb_str), Ok(cddb_id));
	/// ```
	///
	/// Alternatively, you can use its `FromStr` and `TryFrom<&str>` impls:
	///
	/// ```
	/// use cdtoc::{Cddb, Toc};
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// let cddb_id = toc.cddb_id();
	/// let cddb_str = cddb_id.to_string();
	/// assert_eq!(Cddb::try_from(cddb_str.as_str()), Ok(cddb_id));
	/// assert_eq!(cddb_str.parse::<Cddb>(), Ok(cddb_id));
	/// ```
	///
	/// ## Errors
	///
	/// This will return an error if decoding fails.
	pub fn decode<S>(src: S) -> Result<Self, TocError>
	where S: AsRef<str> {
		let src = src.as_ref().as_bytes();
		u32::htou(src).map(Self).ok_or(TocError::CddbDecode)
	}
}



impl Toc {
	#[cfg_attr(docsrs, doc(cfg(feature = "cddb")))]
	#[must_use]
	/// # CDDB ID.
	///
	/// This returns the [CDDB](https://en.wikipedia.org/wiki/CDDB) ID
	/// corresponding to the table of contents.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// let cddb_id = toc.cddb_id();
	///
	/// // Usually you'll want this value as a string:
	/// assert_eq!(
	///     cddb_id.to_string(),
	///     "1f02e004",
	/// );
	///
	/// // But you can also get it as a `u32`:
	/// assert_eq!(
	///     u32::from(cddb_id),
	///     520_282_116,
	/// );
	/// ```
	pub fn cddb_id(&self) -> Cddb { Cddb::from(self) }
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_cddb() {
		for (t, id) in [
			(
				"D+96+3B5D+78E3+B441+EC83+134F4+17225+1A801+1EA5C+23B5B+27CEF+2B58B+2F974+35D56+514C8",
				"b611560e",
			),
			(
				"4+96+2D2B+6256+B327+D84A",
				"1f02e004",
			),
			(
				"10+B6+5352+62AC+99D6+E218+12AC0+135E7+142E9+178B0+19D22+1B0D0+1E7FA+22882+247DB+27074+2A1BD+2C0FB",
				"d6096410",
			),
			(
				"15+247E+2BEC+4AF4+7368+9704+B794+E271+110D0+12B7A+145C1+16CAF+195CF+1B40F+1F04A+21380+2362D+2589D+2793D+2A760+2DA32+300E1+32B46",
				"100a5515",
			),
			(
				"63+96+12D9+5546+A8A2+CAAA+128BF+17194+171DF+1722A+17275+172C0+1730B+17356+173A1+173EC+17437+17482+174CD+17518+17563+175AE+175F9+17644+1768F+176DA+17725+17770+177BB+17806+17851+1789C+178E7+17932+1797D+179C8+17A13+17A5E+17AA9+17AF4+17B3F+17B8A+17BD5+17C20+17C6B+17CB6+17D01+17D4C+17D97+17DE2+17E2D+17E78+17EC3+17F0E+17F59+17FA4+17FEF+1803A+18085+180D0+1811B+18166+181B1+181FC+18247+18292+182DD+18328+18373+183BE+18409+18454+1849F+184EA+18535+18580+185CB+18616+18661+186AC+186F7+18742+1878D+187D8+18823+1886E+188B9+18904+1894F+1899A+189E5+18A30+18A7B+18AC6+18B11+18B5C+18BA7+18BF2+18C38+1ECDC+246E9",
				"cc07c363",
			),
		] {
			let toc = Toc::from_cdtoc(t).expect("Invalid TOC");
			let cddb_id = toc.cddb_id();
			assert_eq!(cddb_id.to_string(), id);

			// Test decoding three ways.
			assert_eq!(Cddb::decode(id), Ok(cddb_id));
			assert_eq!(Cddb::try_from(id), Ok(cddb_id));
			assert_eq!(id.parse::<Cddb>(), Ok(cddb_id));
		}
	}
}
