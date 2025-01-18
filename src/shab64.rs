/*!
# CDTOC: Sha1/Base64
*/

use crate::TocError;
use sha1::{
	Digest,
	Sha1,
};
use std::{
	fmt,
	str::FromStr,
};



#[cfg_attr(docsrs, doc(cfg(feature = "sha1")))]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
/// # Sha1/Base64.
///
/// This struct holds ID data for MusicBrainz and CTDB consisting of a binary
/// sha1 hash encoded with an almost-but-not-quite standard base64 alphabet.
///
/// String formatting is deferred until [`fmt::Display`], allowing for a
/// slightly smaller and `Copy`-friendly footprint.
///
/// If you already have a stringified copy and want to get back to a `ShaB64`,
/// you can use [`ShaB64::decode`] or its `FromStr` or `TryFrom<&str>` impls.
pub struct ShaB64([u8; 20]);

impl fmt::Display for ShaB64 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// The output will always be 28-bytes, ending with a dash.
		let mut buf = [b'-'; 28];

		// For all but the last chunk, it's a simple 3:4 ratio.
		for (raw, dst) in self.0.chunks_exact(3).zip(buf.chunks_exact_mut(4)) {
			dst[0] = base64_encode(raw[0] >> 2);
			dst[1] = base64_encode((raw[0] & 0b0000_0011) << 4 | raw[1] >> 4);
			dst[2] = base64_encode((raw[1] & 0b0000_1111) << 2 | raw[2] >> 6);
			dst[3] = base64_encode(raw[2] & 0b0011_1111);
		}

		// The last byte (27) is always padding, but the three before it still
		// need figuring.
		buf[24] = base64_encode(self.0[18] >> 2);
		buf[25] = base64_encode((self.0[18] & 0b0000_0011) << 4 | self.0[19] >> 4);
		buf[26] = base64_encode((self.0[19] & 0b0000_1111) << 2);

		std::str::from_utf8(buf.as_slice())
			.map_err(|_| fmt::Error)
			.and_then(|s| f.pad(s))
	}
}

impl From<Sha1> for ShaB64 {
	#[inline]
	fn from(src: Sha1) -> Self { Self(<[u8; 20]>::from(src.finalize())) }
}

impl FromStr for ShaB64 {
	type Err = TocError;
	#[inline]
	fn from_str(src: &str) -> Result<Self, Self::Err> { Self::decode(src) }
}

impl TryFrom<&str> for ShaB64 {
	type Error = TocError;
	#[inline]
	fn try_from(src: &str) -> Result<Self, Self::Error> { Self::decode(src) }
}

impl ShaB64 {
	/// # Decode.
	///
	/// Convert a string ID back into a [`ShaB64`] instance.
	///
	/// ## Errors
	///
	/// This will return an error if decoding fails.
	pub fn decode<S>(src: S) -> Result<Self, TocError>
	where S: AsRef<str> {
		let src = src.as_ref().as_bytes();
		if src.len() == 28 && src[27] == b'-' {
			let mut out = [0_u8; 20];

			// Handle all the nice four-byte chunks en masse.
			for (i, chunk) in out.chunks_exact_mut(3).zip(src.chunks_exact(4)) {
				let a = base64_decode(chunk[0])?;
				let b = base64_decode(chunk[1])?;
				let c = base64_decode(chunk[2])?;
				let d = base64_decode(chunk[3])?;
				i.copy_from_slice(&[
					(a & 0b0011_1111) << 2 | b >> 4,
					(b & 0b0000_1111) << 4 | c >> 2,
					(c & 0b0000_0011) << 6 | d & 0b0011_1111,
				]);
			}

			// Handle the remainder manually.
			let a = base64_decode(src[24])?;
			let b = base64_decode(src[25])?;
			let c = base64_decode(src[26])?;
			out[18] = (a & 0b0011_1111) << 2 | b >> 4;
			out[19] = (b & 0b0000_1111) << 4 | c >> 2;

			// Done!
			Ok(Self(out))
		}
		else { Err(TocError::ShaB64Decode) }
	}

	#[inline]
	/// # Push to String.
	///
	/// Unpack and write `self` onto the end of a string without the use of
	/// any intermediary buffers.
	pub(crate) fn push_to_string(&self, out: &mut String) {
		// For all but the last chunk, it's a simple 3:4 ratio.
		for chunk in self.0.chunks_exact(3) {
			out.push(base64_encode(chunk[0] >> 2) as char);
			out.push(base64_encode((chunk[0] & 0b0000_0011) << 4 | chunk[1] >> 4) as char);
			out.push(base64_encode((chunk[1] & 0b0000_1111) << 2 | chunk[2] >> 6) as char);
			out.push(base64_encode(chunk[2] & 0b0011_1111) as char);
		}

		// The last byte (27) is always padding, but the three before it still
		// need figuring.
		out.push(base64_encode(self.0[18] >> 2) as char);
		out.push(base64_encode((self.0[18] & 0b0000_0011) << 4 | self.0[19] >> 4) as char);
		out.push(base64_encode((self.0[19] & 0b0000_1111) << 2) as char);

		// And add one byte for padding.
		out.push('-');
	}
}



/// # Base64 Encode.
///
/// The alphabet used here is mostly standard, except the last two slots have
/// `.` and `_` instead of `+` and `/`.
const fn base64_encode(byte: u8) -> u8 {
	debug_assert!(byte < 64, "BUG: base64 encoding byte is not 6-bit!");
	match byte {
		0..=25 => byte + 65,
		26..=51 => byte + 71,
		52..=61 => byte - 4,
		62 => b'.',
		63 => b'_',
		_ => unreachable!(), // We control the inputs.
	}
}

/// # Base64 Decode.
const fn base64_decode(byte: u8) -> Result<u8, TocError> {
	match byte {
		b'A'..=b'Z' => Ok(byte - 65),
		b'a'..=b'z' => Ok(byte - 71),
		b'0'..=b'9' => Ok(byte + 4),
		b'.' => Ok(62),
		b'_' => Ok(63),
		_ => Err(TocError::ShaB64Decode),
	}
}
