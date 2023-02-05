/*!
# CDTOC: Sha1/Base64
*/

use base64::{
	Engine,
	prelude::BASE64_STANDARD,
};
use crate::TocError;
use sha1::{
	Digest,
	Sha1,
};
use std::fmt;



#[cfg_attr(docsrs, doc(cfg(all(feature = "base64", feature = "sha1"))))]
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
	where S: Into<String> {
		let mut src = src.into().into_bytes();
		if src.len() == 28 && src.is_ascii() {
			// Safety: the string is ASCII, as are the substitutions.
			for b in &mut src {
				match *b {
					b'.' => { *b = b'+'; },
					b'_' => { *b = b'/'; },
					b'-' => { *b = b'='; },
					_ => {},
				}
			}

			// This should be exactly 20 bytes, but the base64 crate doesn't
			// think so so we'll use a vec and figure it out later.
			let mut out = Vec::with_capacity(20);
			BASE64_STANDARD.decode_vec(src, &mut out)
				.map_err(|_| TocError::ShaB64Decode)?;

			// Return if good.
			<[u8; 20]>::try_from(out)
				.map(Self)
				.map_err(|_| TocError::ShaB64Decode)
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
		let mut out = String::with_capacity(28);
		BASE64_STANDARD.encode_string(self.0.as_slice(), &mut out);

		// Safety: the string is ASCII, as are the substitutions.
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
}
