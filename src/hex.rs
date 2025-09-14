/*!
# CDTOC: Hex Encoding.

For this particular library, simpler/purpose-built hex-encoding routines
perform better than fancier ones leveraging SIMD-type optimizations, etc.

Easy enough since we only need to convert `u8` and `u32`…
*/

#![expect(clippy::redundant_pub_crate, reason = "Unresolvable.")]
#![expect(clippy::inline_always, reason = "For performance.")]



#[cfg(feature = "accuraterip")]
#[inline(always)]
#[must_use]
/// # Hex Encode `u8` (lowercase).
pub(crate) const fn lower_encode_u8(src: u8) -> [u8; 2] {
	[lower_hex_digit(src >> 4), lower_hex_digit(src & 0x0f)]
}

#[cfg(feature = "cddb")]
#[inline(always)]
#[must_use]
/// # Hex Encode `u32` (lowercase).
pub(crate) const fn lower_encode_u32(src: u32) -> [u8; 8] {
	let src = src.to_be_bytes();
	[
		lower_hex_digit(src[0] >> 4),
		lower_hex_digit(src[0] & 0x0f),
		lower_hex_digit(src[1] >> 4),
		lower_hex_digit(src[1] & 0x0f),
		lower_hex_digit(src[2] >> 4),
		lower_hex_digit(src[2] & 0x0f),
		lower_hex_digit(src[3] >> 4),
		lower_hex_digit(src[3] & 0x0f),
	]
}

#[inline(always)]
#[must_use]
/// # Hex Encode `u8` (UPPERCASE).
pub(crate) const fn upper_encode_u8(src: u8) -> [u8; 2] {
	[upper_hex_digit(src >> 4), upper_hex_digit(src & 0x0f)]
}

#[inline(always)]
#[must_use]
/// # Hex Encode `u32` (UPPERCASE).
pub(crate) const fn upper_encode_u32(src: u32) -> [u8; 8] {
	let src = src.to_be_bytes();
	[
		upper_hex_digit(src[0] >> 4),
		upper_hex_digit(src[0] & 0x0f),
		upper_hex_digit(src[1] >> 4),
		upper_hex_digit(src[1] & 0x0f),
		upper_hex_digit(src[2] >> 4),
		upper_hex_digit(src[2] & 0x0f),
		upper_hex_digit(src[3] >> 4),
		upper_hex_digit(src[3] & 0x0f),
	]
}



#[cfg(any(feature = "cddb", feature = "accuraterip"))]
#[inline]
#[must_use]
/// # To Hex Digit (lowercase).
const fn lower_hex_digit(digit: u8) -> u8 {
	match digit {
		 0 => b'0',
		 1 => b'1',
		 2 => b'2',
		 3 => b'3',
		 4 => b'4',
		 5 => b'5',
		 6 => b'6',
		 7 => b'7',
		 8 => b'8',
		 9 => b'9',
		10 => b'a',
		11 => b'b',
		12 => b'c',
		13 => b'd',
		14 => b'e',
		 _ => b'f',
	}
}

#[inline]
#[must_use]
/// # To Hex Digit (UPPERCASE).
const fn upper_hex_digit(digit: u8) -> u8 {
	match digit {
		 0 => b'0',
		 1 => b'1',
		 2 => b'2',
		 3 => b'3',
		 4 => b'4',
		 5 => b'5',
		 6 => b'6',
		 7 => b'7',
		 8 => b'8',
		 9 => b'9',
		10 => b'A',
		11 => b'B',
		12 => b'C',
		13 => b'D',
		14 => b'E',
		 _ => b'F',
	}
}
