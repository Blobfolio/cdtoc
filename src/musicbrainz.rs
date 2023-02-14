/*!
# CDTOC: MusicBrainz
*/

use crate::{
	ShaB64,
	Toc,
};



const CHUNK_SIZE: usize = 4;



impl Toc {
	#[allow(clippy::cast_possible_truncation, clippy::missing_panics_doc)]
	#[cfg_attr(docsrs, doc(cfg(feature = "musicbrainz")))]
	#[must_use]
	/// # MusicBrainz ID.
	///
	/// This returns the [MusicBrainz](https://musicbrainz.org/) ID
	/// corresponding to the table of contents.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(
	///     toc.musicbrainz_id().to_string(),
	///     "nljDXdC8B_pDwbdY1vZJvdrAZI4-",
	/// );
	/// ```
	pub fn musicbrainz_id(&self) -> ShaB64 {
		use sha1::Digest;
		let mut sha = sha1::Sha1::new();
		let mut src = [b'0'; CHUNK_SIZE * 4]; // Four raw u32s.
		let mut dst: [u8; CHUNK_SIZE * 8] = [
			b'0', b'1', b'0', b'0', b'0', b'0', b'0', b'0',
			b'0', b'0', b'0', b'0', b'0', b'0', b'0', b'0',
			b'0', b'0', b'0', b'0', b'0', b'0', b'0', b'0',
			b'0', b'0', b'0', b'0', b'0', b'0', b'0', b'0',
		]; // Four hexed u32s.

		// Start with "01", the audio track count, and leadout.
		faster_hex::hex_encode_fallback(&[self.audio_len() as u8], &mut dst[2..4]);
		faster_hex::hex_encode_fallback(self.audio_leadout().to_be_bytes().as_slice(), &mut dst[4..12]);
		dst[2..12].make_ascii_uppercase();
		sha.update(&dst[..12]);

		// Process the sector positions in batches of four to leverage SSE hex
		// optimizations.
		let sectors = self.audio_sectors();
		let len = sectors.len();
		let rem = len % CHUNK_SIZE;
		for v in sectors.chunks_exact(CHUNK_SIZE) {
			// Copy the values to the source buffer.
			for (s_chunk, v) in src.chunks_exact_mut(4).zip(v) {
				s_chunk.copy_from_slice(v.to_be_bytes().as_slice());
			}

			// Encode and hash, en masse.
			faster_hex::hex_encode(src.as_slice(), &mut dst).unwrap();
			dst.make_ascii_uppercase();
			sha.update(dst.as_slice());
		}

		// Handle the remaining sectors, if any,
		if rem != 0 {
			// Copy the values to the source buffer.
			for (s_chunk, v) in src.chunks_exact_mut(4).zip(sectors[len - rem..].iter()) {
				s_chunk.copy_from_slice(v.to_be_bytes().as_slice());
			}

			// Encode and hash, en masse.
			let src_to = rem * 4;
			let dst2 = &mut dst[..src_to * 2];
			faster_hex::hex_encode_fallback(&src[..src_to], dst2);
			dst2.make_ascii_uppercase();
			sha.update(dst2);
		}

		// Pad with zeroes.
		let padding = 99 - sectors.len();
		if padding != 0 { sha.update(&crate::ZEROES[..padding * 8]); }

		// Run it through base64 and we're done!
		ShaB64::from(sha)
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_musicbrainz() {
		for (t, id) in [
			(
				"18+B6+3CE3+7C6F+B2BD+E47F+1121C+15865+175E0+1AED9+1E159+20BF9+235FC+259EF+2826E+29B62+2ED67+311B1+3396B+36ACB+3916B+3BB75+3D60A+40AA6+422FE+48B68+4E4CB",
				"eLuEIkHsua.iJpetabxqYM9SIbk-",
			),
			(
				"D+96+3B5D+78E3+B441+EC83+134F4+17225+1A801+1EA5C+23B5B+27CEF+2B58B+2F974+35D56+514C8",
				"ucgpiD84p.2iBxO4j3hdjSjhtnw-",
			),
			(
				"4+96+2D2B+6256+B327+D84A",
				"nljDXdC8B_pDwbdY1vZJvdrAZI4-",
			),
			(
				"10+B6+5352+62AC+99D6+E218+12AC0+135E7+142E9+178B0+19D22+1B0D0+1E7FA+22882+247DB+27074+2A1BD+2C0FB",
				"PQ02DnwdDaxgWEFSpAzI_IVBL3o-",
			),
			(
				"15+247E+2BEC+4AF4+7368+9704+B794+E271+110D0+12B7A+145C1+16CAF+195CF+1B40F+1F04A+21380+2362D+2589D+2793D+2A760+2DA32+300E1+32B46",
				"JTsyXbyn9DUbppDWELj5o5CiFaI-",
			),
			(
				"63+96+12D9+5546+A8A2+CAAA+128BF+17194+171DF+1722A+17275+172C0+1730B+17356+173A1+173EC+17437+17482+174CD+17518+17563+175AE+175F9+17644+1768F+176DA+17725+17770+177BB+17806+17851+1789C+178E7+17932+1797D+179C8+17A13+17A5E+17AA9+17AF4+17B3F+17B8A+17BD5+17C20+17C6B+17CB6+17D01+17D4C+17D97+17DE2+17E2D+17E78+17EC3+17F0E+17F59+17FA4+17FEF+1803A+18085+180D0+1811B+18166+181B1+181FC+18247+18292+182DD+18328+18373+183BE+18409+18454+1849F+184EA+18535+18580+185CB+18616+18661+186AC+186F7+18742+1878D+187D8+18823+1886E+188B9+18904+1894F+1899A+189E5+18A30+18A7B+18AC6+18B11+18B5C+18BA7+18BF2+18C38+1ECDC+246E9",
				"efFU9TD0IyDF3iME6KlK.rZJEaw-",
			),
		] {
			let toc = Toc::from_cdtoc(t).expect("Invalid TOC");
			let mb_id = toc.musicbrainz_id();
			assert_eq!(mb_id.to_string(), id);

			// Test decoding three ways.
			assert_eq!(ShaB64::decode(id), Ok(mb_id));
			assert_eq!(ShaB64::try_from(id), Ok(mb_id));
			assert_eq!(id.parse::<ShaB64>(), Ok(mb_id));
		}
	}
}
