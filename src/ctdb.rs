/*!
# CDTOC: CUETools Database
*/

use crate::{
	ShaB64,
	Toc,
	TocError,
	TocKind,
};
use dactyl::traits::HexToUnsigned;
use std::collections::BTreeMap;



const CHUNK_SIZE: usize = 4;



impl Toc {
	#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
	#[allow(clippy::missing_panics_doc)]
	#[must_use]
	/// # CUETools Database ID.
	///
	/// This returns the [CUETools Database](http://cue.tools/wiki/CUETools_Database) ID
	/// corresponding to the table of contents.
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(
	///     toc.ctdb_id().to_string(),
	///     "VukMWWItblELRM.CEFpXxw0FlME-",
	/// );
	/// ```
	pub fn ctdb_id(&self) -> ShaB64 {
		use sha1::Digest;
		let mut sha = sha1::Sha1::new();
		let mut src = [b'0'; CHUNK_SIZE * 4]; // Four raw u32s.
		let mut dst = [b'0'; CHUNK_SIZE * 8]; // Four hexed u32s.

		// Split the leadin from the rest of the sectors.
		let [leadin, sectors @ ..] = self.audio_sectors() else { unreachable!() };
		let len = sectors.len();
		let rem = len % CHUNK_SIZE;

		// Process the sector positions in batches of four to leverage SSE hex
		// optimizations.
		for v in sectors.chunks_exact(CHUNK_SIZE) {
			// Copy the values to the source buffer.
			for (s_chunk, v) in src.chunks_exact_mut(4).zip(v.iter().map(|n| n - leadin)) {
				s_chunk.copy_from_slice(v.to_be_bytes().as_slice());
			}

			// Encode and hash, en masse.
			faster_hex::hex_encode(src.as_slice(), &mut dst).unwrap();
			dst.make_ascii_uppercase();
			sha.update(dst.as_slice());
		}

		// Handle the remaining sectors, if any, and the leadout.
		if rem == 0 {
			let dst2 = &mut dst[..8];
			faster_hex::hex_encode_fallback((self.audio_leadout() - leadin).to_be_bytes().as_slice(), dst2);
			dst2.make_ascii_uppercase();
			sha.update(dst2);
		}
		else {
			// Copy the values to the source buffer.
			for (s_chunk, v) in src.chunks_exact_mut(4).zip(
				sectors[len - rem..].iter().map(|n| n - leadin)
					.chain(std::iter::once(self.audio_leadout() - leadin))
			) {
				s_chunk.copy_from_slice(v.to_be_bytes().as_slice());
			}

			// Encode and hash, en masse.
			let src_to = rem * 4 + 4;
			let dst2 = &mut dst[..src_to * 2];
			faster_hex::hex_encode(&src[..src_to], dst2).unwrap();
			dst2.make_ascii_uppercase();
			sha.update(dst2);
		}

		// And padding for a total of 99 tracks.
		let padding = 99 - len;
		if padding != 0 { sha.update(&crate::ZEROES[..padding * 8]); }

		// Run it through base64 and we're done!
		ShaB64::from(sha)
	}

	#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
	#[must_use]
	/// # CUETools Database Checksum URL.
	///
	/// This returns the URL where you can download the checksums for the disc,
	/// provided it is actually _in_ the CTDB. (If it isn't, their server will
	/// return a `404` or empty XML document.)
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	/// assert_eq!(
	///     toc.ctdb_checksum_url(),
	///     "http://db.cuetools.net/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=0:11413:25024:45713:55220",
	/// );
	/// ```
	pub fn ctdb_checksum_url(&self) -> String {
		let mut url = "http://db.cuetools.net/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=".to_string();
		let mut buf = itoa::Buffer::new();

		// Leading data?
		if matches!(self.kind, TocKind::DataFirst) {
			url.push('-');
			url.push_str(buf.format(self.data - 150));
			url.push(':');
		}

		// Each audio track relative to the first.
		for v in &self.audio {
			url.push_str(buf.format(v - 150));
			url.push(':');
		}

		// Trailing data?
		if matches!(self.kind, TocKind::CDExtra) {
			url.push('-');
			url.push_str(buf.format(self.data - 150));
			url.push(':');
		}

		// And the leadout.
		url.push_str(buf.format(self.leadout - 150));

		url
	}

	#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
	/// # Parse Checksums.
	///
	/// This will parse the track checksums from an XML CTDB [lookup](Toc::ctdb_checksum_url).
	///
	/// The return result is a vector — indexed by track number (`n-1`) — of
	/// `checksum => confidence` pairs.
	///
	/// ## Errors
	///
	/// This method uses naive parsing so does not worry about strict XML
	/// validation, but will return an error if other parsing errors are
	/// encountered or no checksums are found.
	pub fn ctdb_parse_checksums(&self, xml: &str) -> Result<Vec<BTreeMap<u32, u16>>, TocError> {
		let audio_len = self.audio_len();
		let mut out: Vec<BTreeMap<u32, u16>> = vec![BTreeMap::default(); audio_len];

		for line in xml.lines() {
			if let Some((confidence, crcs)) = parse_entry(line.trim()) {
				let confidence: u16 = confidence.parse().map_err(|_| TocError::Checksums)?;
				let mut id = 0;
				for chk in crcs.split_ascii_whitespace() {
					let crc = u32::htou(chk.as_bytes()).ok_or(TocError::Checksums)?;
					if crc != 0 {
						let e = out[id].entry(crc).or_insert(0);
						*e = e.saturating_add(confidence);
					}
					id += 1;
				}

				if id != audio_len { return Err(TocError::Checksums); }
			}
		}

		// Consider it okay if we found at least one checksum.
		if out.iter().any(|v| ! v.is_empty()) { Ok(out) }
		else { Err(TocError::NoChecksums) }
	}
}



/// # Parse XML Entry.
///
/// This returns the value subslices corresponding to the "confidence" and
/// "trackcrcs" attributes.
fn parse_entry(line: &str) -> Option<(&str, &str)> {
	if line.starts_with("<entry ") {
		let confidence = parse_attr(line, " confidence=\"")?;
		let crcs = parse_attr(line, " trackcrcs=\"")?;
		Some((confidence, crcs))
	}
	else { None }
}

/// # Parse Entry Value.
///
/// This naively parses an attribute value from a tag, returning the subslice
/// corresponding to its value if non-empty.
///
/// But that's okay; there shouldn't be!
fn parse_attr<'a>(mut line: &'a str, attr: &'static str) -> Option<&'a str> {
	let start = line.find(attr)?;
	line = &line[start + attr.len()..];
	let end = line.find('"')?;

	if 0 < end { Some(line[..end].trim()) }
	else { None }
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_ctdb() {
		for (t, id, lookup) in [
			(
				"18+B6+3CE3+7C6F+B2BD+E47F+1121C+15865+175E0+1AED9+1E159+20BF9+235FC+259EF+2826E+29B62+2ED67+311B1+3396B+36ACB+3916B+3BB75+3D60A+40AA6+422FE+48B68+4E4CB",
				"sBOUSHYC0oLdQZtAEQcmnc3V3Ak-",
				"http://db.cuetools.net/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=32:15437:31705:45607:58345:70022:88015:95562:110147:123075:133987:144742:153945:164312:170700:191697:200987:211157:223797:233685:244447:251252:264720:270952:-297682:320565",
			),
			(
				"D+96+3B5D+78E3+B441+EC83+134F4+17225+1A801+1EA5C+23B5B+27CEF+2B58B+2F974+35D56+514C8",
				"gmEsiU5wvQFA1Nq9YE_posiwgK8-",
				"http://db.cuetools.net/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=0:15047:30797:45995:60397:78942:94607:108395:125382:146117:162905:177397:194782:-220352:332850",
			),
			(
				"4+96+2D2B+6256+B327+D84A",
				"VukMWWItblELRM.CEFpXxw0FlME-",
				"http://db.cuetools.net/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=0:11413:25024:45713:55220",
			),
			(
				"10+B6+5352+62AC+99D6+E218+12AC0+135E7+142E9+178B0+19D22+1B0D0+1E7FA+22882+247DB+27074+2A1BD+2C0FB",
				"iL4EZ56YD5WmG..M4v5qzPG0cFY-",
				"http://db.cuetools.net/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=32:21180:25110:39232:57730:76330:79185:82515:96282:105612:110650:124772:141292:149317:159710:172327:180325",
			),
			(
				"15+247E+2BEC+4AF4+7368+9704+B794+E271+110D0+12B7A+145C1+16CAF+195CF+1B40F+1F04A+21380+2362D+2589D+2793D+2A760+2DA32+300E1+32B46",
				"8geCxI4CSyw_ydvHWGmPQUGF1UE-",
				"http://db.cuetools.net/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=9192:11094:19038:29394:38510:46846:57819:69690:76516:83243:93209:103737:111481:126900:135914:144791:153607:161959:173770:186780:196683:207536",
			),
			(
				"63+96+12D9+5546+A8A2+CAAA+128BF+17194+171DF+1722A+17275+172C0+1730B+17356+173A1+173EC+17437+17482+174CD+17518+17563+175AE+175F9+17644+1768F+176DA+17725+17770+177BB+17806+17851+1789C+178E7+17932+1797D+179C8+17A13+17A5E+17AA9+17AF4+17B3F+17B8A+17BD5+17C20+17C6B+17CB6+17D01+17D4C+17D97+17DE2+17E2D+17E78+17EC3+17F0E+17F59+17FA4+17FEF+1803A+18085+180D0+1811B+18166+181B1+181FC+18247+18292+182DD+18328+18373+183BE+18409+18454+1849F+184EA+18535+18580+185CB+18616+18661+186AC+186F7+18742+1878D+187D8+18823+1886E+188B9+18904+1894F+1899A+189E5+18A30+18A7B+18AC6+18B11+18B5C+18BA7+18BF2+18C38+1ECDC+246E9",
				"okpTZ4Yt2noZkGqbBLte3FfkyVs-",
				"http://db.cuetools.net/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=0:4675:21680:43020:51732:75817:94462:94537:94612:94687:94762:94837:94912:94987:95062:95137:95212:95287:95362:95437:95512:95587:95662:95737:95812:95887:95962:96037:96112:96187:96262:96337:96412:96487:96562:96637:96712:96787:96862:96937:97012:97087:97162:97237:97312:97387:97462:97537:97612:97687:97762:97837:97912:97987:98062:98137:98212:98287:98362:98437:98512:98587:98662:98737:98812:98887:98962:99037:99112:99187:99262:99337:99412:99487:99562:99637:99712:99787:99862:99937:100012:100087:100162:100237:100312:100387:100462:100537:100612:100687:100762:100837:100912:100987:101062:101137:101212:101282:126022:149075",
			),
		] {
			let toc = Toc::from_cdtoc(t).expect("Invalid TOC");
			let ctdb_id = toc.ctdb_id();
			assert_eq!(ctdb_id.to_string(), id);
			assert_eq!(toc.ctdb_checksum_url(), lookup);

			// Test decoding three ways.
			assert_eq!(ShaB64::decode(id), Ok(ctdb_id));
			assert_eq!(ShaB64::try_from(id), Ok(ctdb_id));
			assert_eq!(id.parse::<ShaB64>(), Ok(ctdb_id));
		}
	}
}
