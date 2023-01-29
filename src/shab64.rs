/*!
# CDTOC: Sha1/Base64
*/

use base64::{
	Engine,
	prelude::BASE64_STANDARD,
};
use sha1::{
	Digest,
	Sha1,
};
use std::fmt;



#[cfg_attr(feature = "docsrs", doc(cfg(all(feature = "base64", feature = "sha1"))))]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
/// # Sha1/Base64.
///
/// This struct holds ID data for MusicBrainz and CTDB consisting of a binary
/// sha1 hash encoded with an almost-but-not-quite standard base64 alphabet.
///
/// String formatting is deferred until [`Shab64::to_string`] or
/// [`Shab64::pretty_print`] are called, allowing for a slightly smaller and
/// `copy`-friendly footprint.
pub struct Shab64([u8; 20]);

impl fmt::Display for Shab64 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.pretty_print())
	}
}

impl From<Sha1> for Shab64 {
	fn from(src: Sha1) -> Self { Self(<[u8; 20]>::from(src.finalize())) }
}

impl Shab64 {
	#[cfg_attr(feature = "docsrs", doc(cfg(all(feature = "base64", feature = "sha1"))))]
	#[allow(unsafe_code)]
	#[must_use]
	/// # Pretty Print.
	///
	/// Return the value has a human-readable string, exactly like [`Shab64::to_string`],
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
