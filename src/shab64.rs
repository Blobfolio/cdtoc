/*!
# CDTOC: Sha1/Base64
*/

use crate::TocError;
use sha1::{
	Digest,
	Sha1,
};
use std::fmt;



#[cfg_attr(docsrs, doc(cfg(feature = "sha1")))]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
/// # Sha1/Base64.
///
/// This struct holds ID data for MusicBrainz and CTDB consisting of a binary
/// sha1 hash encoded with an almost-but-not-quite standard base64 alphabet.
///
/// String formatting is deferred until `ShaB64::to_string` or
/// [`ShaB64::pretty_print`] are called, allowing for a slightly smaller and
/// `copy`-friendly footprint.
pub struct ShaB64([u8; 20]);

impl fmt::Display for ShaB64 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.pretty_print())
	}
}

impl From<Sha1> for ShaB64 {
	fn from(src: Sha1) -> Self { Self(<[u8; 20]>::from(src.finalize())) }
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

	#[allow(unsafe_code)]
	#[must_use]
	/// # Pretty Print.
	///
	/// Return the value has a human-readable string, exactly like `ShaB64::to_string`,
	/// but slightly faster. The result will always be 28-characters in length.
	pub fn pretty_print(&self) -> String {
		let mut out = Vec::with_capacity(28);

		// Handle all the nice 3-byte chunks en masse.
		for chunk in self.0.chunks_exact(3) {
			out.push(base64_encode(chunk[0] >> 2));
			out.push(base64_encode((chunk[0] & 0b0000_0011) << 4 | chunk[1] >> 4));
			out.push(base64_encode((chunk[1] & 0b0000_1111) << 2 | chunk[2] >> 6));
			out.push(base64_encode(chunk[2] & 0b0011_1111));
		}

		// Handle the remainder manually.
		out.push(base64_encode(self.0[18] >> 2));
		out.push(base64_encode((self.0[18] & 0b0000_0011) << 4 | self.0[19] >> 4));
		out.push(base64_encode((self.0[19] & 0b0000_1111) << 2));

		// And add one byte for padding.
		out.push(b'-');

		// Safety: our alphabet is ASCII.
		unsafe { String::from_utf8_unchecked(out) }
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
