/*!
# CDTOC: AccurateRip
*/

use crate::{
	Cddb,
	Toc,
};
use std::fmt;



#[cfg_attr(feature = "docsrs", doc(cfg(feature = "accuraterip")))]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
/// # AccurateRip ID.
///
/// This struct holds an [AccurateRip](http://accuraterip.com/) ID.
///
/// Values of this type are returned by [`Toc::accuraterip_id`].
///
/// ## Examples
///
/// ```
/// use cdtoc::Toc;
///
/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
/// let ar_id = toc.accuraterip_id();
///
/// // Usually you'll want this value as a string:
/// assert_eq!(
///     ar_id.to_string(),
///     "004-0002189a-00087f33-1f02e004",
/// );
///
/// // But you can also get a binary version matching the format of the
/// // checksum bin files:
/// assert_eq!(
///     <[u8; 13]>::from(ar_id),
///     [4, 154, 24, 2, 0, 51, 127, 8, 0, 4, 224, 2, 31],
/// );
/// ```
pub struct AccurateRip([u8; 13]);

impl AsRef<[u8]> for AccurateRip {
	fn as_ref(&self) -> &[u8] { &self.0 }
}

impl From<AccurateRip> for [u8; 13] {
	fn from(src: AccurateRip) -> Self { src.0 }
}

impl fmt::Display for AccurateRip {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let b = u32::from_le_bytes([self.0[1], self.0[2], self.0[3], self.0[4]]);
		let c = u32::from_le_bytes([self.0[5], self.0[6], self.0[7], self.0[8]]);
		let d = u32::from_le_bytes([self.0[9], self.0[10], self.0[11], self.0[12]]);

		write!(f, "{:03}-{b:08x}-{c:08x}-{d:08x}", self.0[0])
	}
}

impl From<&Toc> for AccurateRip {
	#[allow(clippy::cast_possible_truncation)]
	fn from(src: &Toc) -> Self {
		let mut b: u32 = 0;
		let mut c: u32 = 0;

		let mut idx = 1;
		for v in src.audio_sectors() {
			let off = v.saturating_sub(150);
			b += off;
			c += off.max(1) * idx;
			idx += 1;
		}

		// Add in the last part.
		let leadout = src.leadout().saturating_sub(150);

		let b = (b + leadout).to_le_bytes();
		let c = (c + leadout.max(1) * idx).to_le_bytes();
		let d = u32::from(src.cddb_id()).to_le_bytes();

		Self([
			src.audio_len() as u8,
			b[0], b[1], b[2], b[3],
			c[0], c[1], c[2], c[3],
			d[0], d[1], d[2], d[3],
		])
	}
}

impl AccurateRip {
	#[must_use]
	/// # Number of Audio Tracks.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// // From Toc.
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(toc.audio_len(), 4_usize);
	///
	/// // From AccurateRip.
	/// let disc_id = toc.accuraterip_id();
	/// assert_eq!(disc_id.audio_len(), 4_u8);
	/// ```
	pub const fn audio_len(&self) -> u8 { self.0[0] }

	#[cfg_attr(feature = "docsrs", doc(cfg(feature = "accuraterip")))]
	#[must_use]
	/// # AccurateRip Checksum URL.
	///
	/// This returns the URL where you can download the v1 and v2 checksums for
	/// the disc, provided it is actually _in_ the AccurateRip database. (If it
	/// isn't, their server will return a `404`.)
	///
	/// You can also get this directly via [`Toc::accuraterip_checksum_url`].
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// let ar_id = toc.accuraterip_id();
	/// assert_eq!(
	///     ar_id.checksum_url(),
	///     "http://www.accuraterip.com/accuraterip/a/9/8/dBAR-004-0002189a-00087f33-1f02e004.bin",
	/// );
	/// ```
	pub fn checksum_url(&self) -> String {
		let disc_id = self.to_string();
		[
			"http://www.accuraterip.com/accuraterip/",
			&disc_id[11..12],
			"/",
			&disc_id[10..11],
			"/",
			&disc_id[9..10],
			"/dBAR-",
			&disc_id,
			".bin",
		].concat()
	}

	#[cfg_attr(feature = "docsrs", doc(cfg(all(feature = "accuraterip", feature = "cddb"))))]
	#[must_use]
	/// # CDDB ID.
	///
	/// In cases where your application requires both AccurateRip and CDDB IDs,
	/// using this method to obtain the latter is cheaper than calling
	/// [`Toc::cddb_id`].
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// let ar_id = toc.accuraterip_id();
	/// assert_eq!(
	///     ar_id.cddb_id(),
	///     toc.cddb_id(),
	/// );
	/// ```
	pub const fn cddb_id(&self) -> Cddb {
		Cddb(u32::from_le_bytes([
			self.0[9],
			self.0[10],
			self.0[11],
			self.0[12],
		]))
	}
}



impl Toc {
	#[cfg_attr(feature = "docsrs", doc(cfg(feature = "accuraterip")))]
	#[must_use]
	/// # AccurateRip ID.
	///
	/// This returns the [AccurateRip](http://accuraterip.com/) ID
	/// corresponding to the table of contents.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// let ar_id = toc.accuraterip_id();
	///
	/// // Usually you'll want this value as a string:
	/// assert_eq!(
	///     ar_id.to_string(),
	///     "004-0002189a-00087f33-1f02e004",
	/// );
	///
	/// // But you can also get a binary version matching the format of the
	/// // checksum bin files:
	/// assert_eq!(
	///     <[u8; 13]>::from(ar_id),
	///     [4, 154, 24, 2, 0, 51, 127, 8, 0, 4, 224, 2, 31],
	/// );
	/// ```
	pub fn accuraterip_id(&self) -> AccurateRip { AccurateRip::from(self) }

	#[cfg_attr(feature = "docsrs", doc(cfg(feature = "accuraterip")))]
	#[must_use]
	/// # AccurateRip Checksum URL.
	///
	/// This returns the URL where you can download the v1 and v2 checksums for
	/// the disc, provided it is actually _in_ the AccurateRip database. (If it
	/// isn't, their server will return a `404`.)
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(
	///     toc.accuraterip_checksum_url(),
	///     "http://www.accuraterip.com/accuraterip/a/9/8/dBAR-004-0002189a-00087f33-1f02e004.bin",
	/// );
	/// ```
	pub fn accuraterip_checksum_url(&self) -> String {
		self.accuraterip_id().checksum_url()
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_accuraterip() {
		for (t, c) in [
			(
				"D+96+3B5D+78E3+B441+EC83+134F4+17225+1A801+1EA5C+23B5B+27CEF+2B58B+2F974+35D56+514C8",
				"013-001802ed-00f8ee31-b611560e",
			),
			(
				"4+96+2D2B+6256+B327+D84A",
				"004-0002189a-00087f33-1f02e004",
			),
			(
				"10+B6+5352+62AC+99D6+E218+12AC0+135E7+142E9+178B0+19D22+1B0D0+1E7FA+22882+247DB+27074+2A1BD+2C0FB",
				"016-0018be61-012232a8-d6096410",
			),
			(
				"15+247E+2BEC+4AF4+7368+9704+B794+E271+110D0+12B7A+145C1+16CAF+195CF+1B40F+1F04A+21380+2362D+2589D+2793D+2A760+2DA32+300E1+32B46",
				"021-0022250d-020afc1b-100a5515",
			),
			(
				"63+96+12D9+5546+A8A2+CAAA+128BF+17194+171DF+1722A+17275+172C0+1730B+17356+173A1+173EC+17437+17482+174CD+17518+17563+175AE+175F9+17644+1768F+176DA+17725+17770+177BB+17806+17851+1789C+178E7+17932+1797D+179C8+17A13+17A5E+17AA9+17AF4+17B3F+17B8A+17BD5+17C20+17C6B+17CB6+17D01+17D4C+17D97+17DE2+17E2D+17E78+17EC3+17F0E+17F59+17FA4+17FEF+1803A+18085+180D0+1811B+18166+181B1+181FC+18247+18292+182DD+18328+18373+183BE+18409+18454+1849F+184EA+18535+18580+185CB+18616+18661+186AC+186F7+18742+1878D+187D8+18823+1886E+188B9+18904+1894F+1899A+189E5+18A30+18A7B+18AC6+18B11+18B5C+18BA7+18BF2+18C38+1ECDC+246E9",
				"099-00909976-1e2814f1-cc07c363",
			),
		] {
			let toc = Toc::from_cdtoc(t).expect("Invalid TOC");
			assert_eq!(toc.accuraterip_id().to_string(), c);
		}
	}
}