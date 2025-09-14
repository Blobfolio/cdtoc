/*!
# CDTOC: Hex Encoding.

This library does a lot of hex encoding, but it's all small-scale stuff that
won't benefit from the sorts of fancy SIMD tricks third-party crates provide,
so let's skip the deps and do it our selves!
*/

#![expect(clippy::redundant_pub_crate, reason = "Unresolvable.")]

/// # Helper: Hex.
///
/// Our implementation is naive but tedious, so best let macros help with the
/// copy-and-paste. Haha.
macro_rules! hex {
	(@byte 0) => ( b'0' );
	(@byte 1) => ( b'1' );
	(@byte 2) => ( b'2' );
	(@byte 3) => ( b'3' );
	(@byte 4) => ( b'4' );
	(@byte 5) => ( b'5' );
	(@byte 6) => ( b'6' );
	(@byte 7) => ( b'7' );
	(@byte 8) => ( b'8' );
	(@byte 9) => ( b'9' );
	(@byte A) => ( b'A' );
	(@byte B) => ( b'B' );
	(@byte C) => ( b'C' );
	(@byte D) => ( b'D' );
	(@byte E) => ( b'E' );
	(@byte F) => ( b'F' );

	(@byte_low 0) => ( b'0' );
	(@byte_low 1) => ( b'1' );
	(@byte_low 2) => ( b'2' );
	(@byte_low 3) => ( b'3' );
	(@byte_low 4) => ( b'4' );
	(@byte_low 5) => ( b'5' );
	(@byte_low 6) => ( b'6' );
	(@byte_low 7) => ( b'7' );
	(@byte_low 8) => ( b'8' );
	(@byte_low 9) => ( b'9' );
	(@byte_low A) => ( b'a' );
	(@byte_low B) => ( b'b' );
	(@byte_low C) => ( b'c' );
	(@byte_low D) => ( b'd' );
	(@byte_low E) => ( b'e' );
	(@byte_low F) => ( b'f' );

	// Entry point.
	( $( $dec:literal $h1:tt $h2:tt, )+ ) => (
		impl Hex {
			#[inline]
			#[must_use]
			/// # (Upper) Hex Encode `u8`.
			pub(crate) const fn upper_encode_u8(src: u8) -> [u8; 2] {
				match src {
					$( $dec => [hex!(@byte $h1), hex!(@byte $h2)], )+
				}
			}

			#[cfg(any(feature = "accuraterip", feature = "cddb"))]
			#[inline]
			#[must_use]
			/// # (Lower) Hex Encode `u8`.
			pub(crate) const fn lower_encode_u8(src: u8) -> [u8; 2] {
				match src {
					$( $dec => [hex!(@byte_low $h1), hex!(@byte_low $h2)], )+
				}
			}
		}
	);
}

/// # Hex Encoder.
pub(crate) struct Hex;

hex! {
	0   0 0,
	1   0 1,
	2   0 2,
	3   0 3,
	4   0 4,
	5   0 5,
	6   0 6,
	7   0 7,
	8   0 8,
	9   0 9,
	10  0 A,
	11  0 B,
	12  0 C,
	13  0 D,
	14  0 E,
	15  0 F,
	16  1 0,
	17  1 1,
	18  1 2,
	19  1 3,
	20  1 4,
	21  1 5,
	22  1 6,
	23  1 7,
	24  1 8,
	25  1 9,
	26  1 A,
	27  1 B,
	28  1 C,
	29  1 D,
	30  1 E,
	31  1 F,
	32  2 0,
	33  2 1,
	34  2 2,
	35  2 3,
	36  2 4,
	37  2 5,
	38  2 6,
	39  2 7,
	40  2 8,
	41  2 9,
	42  2 A,
	43  2 B,
	44  2 C,
	45  2 D,
	46  2 E,
	47  2 F,
	48  3 0,
	49  3 1,
	50  3 2,
	51  3 3,
	52  3 4,
	53  3 5,
	54  3 6,
	55  3 7,
	56  3 8,
	57  3 9,
	58  3 A,
	59  3 B,
	60  3 C,
	61  3 D,
	62  3 E,
	63  3 F,
	64  4 0,
	65  4 1,
	66  4 2,
	67  4 3,
	68  4 4,
	69  4 5,
	70  4 6,
	71  4 7,
	72  4 8,
	73  4 9,
	74  4 A,
	75  4 B,
	76  4 C,
	77  4 D,
	78  4 E,
	79  4 F,
	80  5 0,
	81  5 1,
	82  5 2,
	83  5 3,
	84  5 4,
	85  5 5,
	86  5 6,
	87  5 7,
	88  5 8,
	89  5 9,
	90  5 A,
	91  5 B,
	92  5 C,
	93  5 D,
	94  5 E,
	95  5 F,
	96  6 0,
	97  6 1,
	98  6 2,
	99  6 3,
	100 6 4,
	101 6 5,
	102 6 6,
	103 6 7,
	104 6 8,
	105 6 9,
	106 6 A,
	107 6 B,
	108 6 C,
	109 6 D,
	110 6 E,
	111 6 F,
	112 7 0,
	113 7 1,
	114 7 2,
	115 7 3,
	116 7 4,
	117 7 5,
	118 7 6,
	119 7 7,
	120 7 8,
	121 7 9,
	122 7 A,
	123 7 B,
	124 7 C,
	125 7 D,
	126 7 E,
	127 7 F,
	128 8 0,
	129 8 1,
	130 8 2,
	131 8 3,
	132 8 4,
	133 8 5,
	134 8 6,
	135 8 7,
	136 8 8,
	137 8 9,
	138 8 A,
	139 8 B,
	140 8 C,
	141 8 D,
	142 8 E,
	143 8 F,
	144 9 0,
	145 9 1,
	146 9 2,
	147 9 3,
	148 9 4,
	149 9 5,
	150 9 6,
	151 9 7,
	152 9 8,
	153 9 9,
	154 9 A,
	155 9 B,
	156 9 C,
	157 9 D,
	158 9 E,
	159 9 F,
	160 A 0,
	161 A 1,
	162 A 2,
	163 A 3,
	164 A 4,
	165 A 5,
	166 A 6,
	167 A 7,
	168 A 8,
	169 A 9,
	170 A A,
	171 A B,
	172 A C,
	173 A D,
	174 A E,
	175 A F,
	176 B 0,
	177 B 1,
	178 B 2,
	179 B 3,
	180 B 4,
	181 B 5,
	182 B 6,
	183 B 7,
	184 B 8,
	185 B 9,
	186 B A,
	187 B B,
	188 B C,
	189 B D,
	190 B E,
	191 B F,
	192 C 0,
	193 C 1,
	194 C 2,
	195 C 3,
	196 C 4,
	197 C 5,
	198 C 6,
	199 C 7,
	200 C 8,
	201 C 9,
	202 C A,
	203 C B,
	204 C C,
	205 C D,
	206 C E,
	207 C F,
	208 D 0,
	209 D 1,
	210 D 2,
	211 D 3,
	212 D 4,
	213 D 5,
	214 D 6,
	215 D 7,
	216 D 8,
	217 D 9,
	218 D A,
	219 D B,
	220 D C,
	221 D D,
	222 D E,
	223 D F,
	224 E 0,
	225 E 1,
	226 E 2,
	227 E 3,
	228 E 4,
	229 E 5,
	230 E 6,
	231 E 7,
	232 E 8,
	233 E 9,
	234 E A,
	235 E B,
	236 E C,
	237 E D,
	238 E E,
	239 E F,
	240 F 0,
	241 F 1,
	242 F 2,
	243 F 3,
	244 F 4,
	245 F 5,
	246 F 6,
	247 F 7,
	248 F 8,
	249 F 9,
	250 F A,
	251 F B,
	252 F C,
	253 F D,
	254 F E,
	255 F F,
}

impl Hex {
	#[cfg(feature = "cddb")]
	#[expect(clippy::many_single_char_names, reason = "For consistency.")]
	#[inline]
	#[must_use]
	/// # (Lower) Hex Encode u32.
	pub(crate) const fn lower_encode_u32(src: u32) -> [u8; 8] {
		let src = src.to_be_bytes();
		let [a, b] = Self::lower_encode_u8(src[0]);
		let [c, d] = Self::lower_encode_u8(src[1]);
		let [e, f] = Self::lower_encode_u8(src[2]);
		let [g, h] = Self::lower_encode_u8(src[3]);

		[a, b, c, d, e, f, g, h]
	}

	#[expect(clippy::many_single_char_names, reason = "For consistency.")]
	#[inline]
	#[must_use]
	/// # (Upper) Hex Encode u32.
	pub(crate) const fn upper_encode_u32(src: u32) -> [u8; 8] {
		let src = src.to_be_bytes();
		let [a, b] = Self::upper_encode_u8(src[0]);
		let [c, d] = Self::upper_encode_u8(src[1]);
		let [e, f] = Self::upper_encode_u8(src[2]);
		let [g, h] = Self::upper_encode_u8(src[3]);

		[a, b, c, d, e, f, g, h]
	}

	#[cfg(any(feature = "ctdb", feature = "musicbrainz"))]
	/// # (Upper) Hex Encode/Write Array.
	pub(crate) const fn upper_array<const S: usize, const D: usize>(
		src: &[u8; S],
		dst: &mut [u8; D],
	) {
		const {
			assert!(
				S != 0 && S * 2 == D,
				"BUG: Destination array must be twice as large as source.",
			);
		}

		let mut idx1 = 0;
		let mut idx2 = 0;
		while idx1 < src.len() {
			[dst[idx2], dst[idx2 + 1]] = Self::upper_encode_u8(src[idx1]);
			idx1 += 1;
			idx2 += 2;
		}
	}
}
