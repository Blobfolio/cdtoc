/*!
# CDTOC: CUETools Database
*/

use crate::{
	hex,
	ShaB64,
	Toc,
	TocError,
	TocKind,
};
use dactyl::traits::HexToUnsigned;
use std::collections::BTreeMap;



impl Toc {
	#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
	#[must_use]
	/// # CUETools Database ID.
	///
	/// This returns the [CUETools Database](https://cue.tools/wiki/CUETools_Database) ID
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

		// Split the leadin from the rest of the sectors.
		let [leadin, sectors @ ..] = self.audio_sectors() else { unreachable!() };

		// Start with a whole lotta ASCII zeroes.
		let mut buf = crate::TRACK_ZEROES;

		// Overwrite the first few entries with the audio and leadout sectors,
		// relative to the leadin.
		for (k, v) in sectors.iter().copied().chain(std::iter::once(self.audio_leadout())).enumerate() {
			buf[k] = hex::upper_encode_u32(v - leadin);
		}

		// SHA and done!
		let mut sha = sha1::Sha1::new();
		sha.update(buf.as_flattened());
		ShaB64::from(sha)
	}

	#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
	#[must_use]
	/// # CUETools Database URL.
	///
	/// This URL can be visited in a web browser to view the details for the
	/// disc (if it is present in the database).
	///
	/// See also: [`Toc::ctdb_checksum_url`]
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	///
	/// // Note: the CUETools website lacks SSL/TLS support.
	/// assert_eq!(
	///     toc.ctdb_url(),
	///     "https://db.cue.tools/ui/?tocid=VukMWWItblELRM.CEFpXxw0FlME-",
	/// );
	/// ```
	pub fn ctdb_url(&self) -> String {
		let mut out = String::with_capacity(59);
		out.push_str("https://db.cue.tools/ui/?tocid=");
		self.ctdb_id().push_to_string(&mut out);
		out
	}

	#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
	#[must_use]
	/// # CUETools Database Checksum URL.
	///
	/// This URL can be used to fetch XML-formatted checksums and metadata for
	/// the disc (if it is present in the database).
	///
	/// See also: [`Toc::ctdb_url`]
	///
	/// ## Examples
	///
	/// ```
	/// use cdtoc::Toc;
	///
	/// let toc = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();
	///
	/// // Note: the CUETools website lacks SSL/TLS support.
	/// assert_eq!(
	///     toc.ctdb_checksum_url(),
	///     "https://db.cue.tools/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=0:11413:25024:45713:55220",
	/// );
	/// ```
	pub fn ctdb_checksum_url(&self) -> String {
		// We can't efficiently precalculate the exact size, but the next
		// power-of-two isn't too far away, so we might as well start there.
		let mut url = String::with_capacity(128);
		url.push_str("https://db.cue.tools/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=");

		// Digit buffer.
		let mut buf = U32DigitBuffer::DEFAULT;

		// Leading data?
		if matches!(self.kind, TocKind::DataFirst) {
			url.push('-');
			for &c in buf.format(self.data - 150) { url.push(c); }
			url.push(':');
		}

		// Each audio track relative to the first.
		for v in &self.audio {
			for &c in buf.format(v - 150) { url.push(c); }
			url.push(':');
		}

		// Trailing data?
		if matches!(self.kind, TocKind::CDExtra) {
			url.push('-');
			for &c in buf.format(self.data - 150) { url.push(c); }
			url.push(':');
		}

		// And the leadout.
		for &c in buf.format(self.leadout - 150) { url.push(c); }

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



#[derive(Debug, Clone, Copy)]
/// # Digit Buffer.
///
/// This buffer is used by `ctdb_checksum_url` to convert `u32` values to
/// string. It doesn't do anything fancy, but helps reduce writes/allocations.
struct U32DigitBuffer([char; 10]);

impl U32DigitBuffer {
	/// # Default Buffer.
	const DEFAULT: Self = Self(['0'; 10]);

	#[expect(clippy::cast_possible_truncation, reason = "False positive.")]
	/// # Digitize a Number.
	///
	/// Return a slice containing each digit represented as an ASCII `char`.
	const fn format(&mut self, mut num: u32) -> &[char] {
		// Fill the buffer, right to left.
		let mut len = 0;
		while 9 < num {
			len += 1;
			self.0[10 - len] = ((num % 10) as u8 ^ b'0') as char;
			num /= 10;
		}
		len += 1;
		self.0[10 - len] = (num as u8 ^ b'0') as char;

		// Split off and return the relevant part.
		let (_, b) = self.0.split_at(10 - len);
		b
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	#[test]
	fn t_ctdb() {
		for (t, id, lookup) in [
			(
				"18+B6+3CE3+7C6F+B2BD+E47F+1121C+15865+175E0+1AED9+1E159+20BF9+235FC+259EF+2826E+29B62+2ED67+311B1+3396B+36ACB+3916B+3BB75+3D60A+40AA6+422FE+48B68+4E4CB",
				"sBOUSHYC0oLdQZtAEQcmnc3V3Ak-",
				"https://db.cue.tools/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=32:15437:31705:45607:58345:70022:88015:95562:110147:123075:133987:144742:153945:164312:170700:191697:200987:211157:223797:233685:244447:251252:264720:270952:-297682:320565",
			),
			(
				"D+96+3B5D+78E3+B441+EC83+134F4+17225+1A801+1EA5C+23B5B+27CEF+2B58B+2F974+35D56+514C8",
				"gmEsiU5wvQFA1Nq9YE_posiwgK8-",
				"https://db.cue.tools/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=0:15047:30797:45995:60397:78942:94607:108395:125382:146117:162905:177397:194782:-220352:332850",
			),
			(
				"4+96+2D2B+6256+B327+D84A",
				"VukMWWItblELRM.CEFpXxw0FlME-",
				"https://db.cue.tools/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=0:11413:25024:45713:55220",
			),
			(
				"10+B6+5352+62AC+99D6+E218+12AC0+135E7+142E9+178B0+19D22+1B0D0+1E7FA+22882+247DB+27074+2A1BD+2C0FB",
				"iL4EZ56YD5WmG..M4v5qzPG0cFY-",
				"https://db.cue.tools/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=32:21180:25110:39232:57730:76330:79185:82515:96282:105612:110650:124772:141292:149317:159710:172327:180325",
			),
			(
				"15+247E+2BEC+4AF4+7368+9704+B794+E271+110D0+12B7A+145C1+16CAF+195CF+1B40F+1F04A+21380+2362D+2589D+2793D+2A760+2DA32+300E1+32B46",
				"8geCxI4CSyw_ydvHWGmPQUGF1UE-",
				"https://db.cue.tools/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=9192:11094:19038:29394:38510:46846:57819:69690:76516:83243:93209:103737:111481:126900:135914:144791:153607:161959:173770:186780:196683:207536",
			),
			(
				"63+96+12D9+5546+A8A2+CAAA+128BF+17194+171DF+1722A+17275+172C0+1730B+17356+173A1+173EC+17437+17482+174CD+17518+17563+175AE+175F9+17644+1768F+176DA+17725+17770+177BB+17806+17851+1789C+178E7+17932+1797D+179C8+17A13+17A5E+17AA9+17AF4+17B3F+17B8A+17BD5+17C20+17C6B+17CB6+17D01+17D4C+17D97+17DE2+17E2D+17E78+17EC3+17F0E+17F59+17FA4+17FEF+1803A+18085+180D0+1811B+18166+181B1+181FC+18247+18292+182DD+18328+18373+183BE+18409+18454+1849F+184EA+18535+18580+185CB+18616+18661+186AC+186F7+18742+1878D+187D8+18823+1886E+188B9+18904+1894F+1899A+189E5+18A30+18A7B+18AC6+18B11+18B5C+18BA7+18BF2+18C38+1ECDC+246E9",
				"okpTZ4Yt2noZkGqbBLte3FfkyVs-",
				"https://db.cue.tools/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=0:4675:21680:43020:51732:75817:94462:94537:94612:94687:94762:94837:94912:94987:95062:95137:95212:95287:95362:95437:95512:95587:95662:95737:95812:95887:95962:96037:96112:96187:96262:96337:96412:96487:96562:96637:96712:96787:96862:96937:97012:97087:97162:97237:97312:97387:97462:97537:97612:97687:97762:97837:97912:97987:98062:98137:98212:98287:98362:98437:98512:98587:98662:98737:98812:98887:98962:99037:99112:99187:99262:99337:99412:99487:99562:99637:99712:99787:99862:99937:100012:100087:100162:100237:100312:100387:100462:100537:100612:100687:100762:100837:100912:100987:101062:101137:101212:101282:126022:149075",
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

	#[test]
	fn t_digits() {
		let mut buf = U32DigitBuffer::DEFAULT;
		assert_eq!(buf.format(0), &['0']);
		assert_eq!(buf.format(10), &['1', '0']);
		assert_eq!(buf.format(432), &['4', '3', '2']);
		assert_eq!(buf.format(50_000), &['5', '0', '0', '0', '0']);
		assert_eq!(buf.format(u32::MAX), &['4', '2', '9', '4', '9', '6', '7', '2', '9', '5']);
	}

	#[test]
	fn ctdb_parse_checksums() {
		let xml = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("skel/ctdb.xml");
		let xml = std::fs::read_to_string(&xml).expect("Unable to read CTDB XML.");

		let toc = Toc::from_cdtoc("F+96+4173+8BDF+C8C7+11443+15E6B+192F6+1DB52+21D82+26CD7+2B227+3084B+35C25+39A3F+3D63F+449EF+4E4AB").expect("Invalid TOC");
		let parsed = toc.ctdb_parse_checksums(&xml).expect("Unable to parse checksums");
		let expected = vec![
			BTreeMap::<u32, u16>::from([(763282391, 1), (897588894, 1), (1235037124, 1), (1282249905, 349), (3152505343, 1), (3258244565, 1), (3433605098, 1), (3433918662, 1), (3700015550, 1)]),
			BTreeMap::<u32, u16>::from([(12391530, 1), (254318592, 1), (278329866, 1), (1024246154, 1), (1270661992, 1), (1443436445, 1), (2588556635, 1), (2609626001, 1), (3172155240, 348), (3507001436, 1)]),
			BTreeMap::<u32, u16>::from([(79025172, 1), (576176869, 350), (1124109082, 1), (1202985854, 1), (1920914588, 1), (1936773867, 1), (3216934765, 1), (4144397645, 1)]),
			BTreeMap::<u32, u16>::from([(1087690482, 1), (1089145401, 1), (2243388636, 1), (2618259125, 1), (2968527536, 1), (3345952562, 349), (3498817224, 1), (3856655528, 1), (3901531533, 1)]),
			BTreeMap::<u32, u16>::from([(194084823, 1), (538997881, 1), (1052544287, 1), (1124555716, 1), (1314115734, 1), (1495657225, 1), (1872765379, 1), (2380642597, 1), (2899014380, 348), (3224619805, 1)]),
			BTreeMap::<u32, u16>::from([(1083862139, 351), (1370448129, 1), (1715012676, 1), (1749827653, 1), (2019350366, 1), (2081480574, 1), (3961881527, 1)]),
			BTreeMap::<u32, u16>::from([(501630488, 1), (1403870687, 1), (1818657605, 1), (2052363808, 349), (2233541139, 1), (3001272542, 1), (3702882640, 1), (3790048568, 1), (3830547781, 1)]),
			BTreeMap::<u32, u16>::from([(68973958, 1), (109342161, 349), (623696074, 1), (660407935, 1), (844055799, 1), (1340619147, 1), (3367889202, 1), (3455769741, 1), (4292440658, 1)]),
			BTreeMap::<u32, u16>::from([(177411350, 1), (346112569, 1), (1088872326, 1), (1578224278, 1), (1981717760, 1), (2149128210, 1), (3308254768, 1), (3564527459, 349), (3833535237, 1)]),
			BTreeMap::<u32, u16>::from([(2294948521, 348), (2450394573, 1), (2777818443, 1), (2793045780, 1), (2930033218, 1), (2939809789, 1), (3144352102, 1), (3236236899, 1), (3386702153, 1), (4059469375, 1)]),
			BTreeMap::<u32, u16>::from([(187554328, 1), (692911428, 1), (788436667, 1), (1223347259, 1), (1376669804, 345), (1527008808, 1), (2152877939, 1), (2366724378, 1), (2675505606, 1), (3097721897, 1), (3343301748, 1), (3511184516, 1), (3843536879, 1)]),
			BTreeMap::<u32, u16>::from([(12734792, 1), (86535921, 1), (616290089, 1), (684539636, 1), (1281593774, 1), (1812856593, 1), (2135611934, 1), (2212148269, 1), (2745468274, 1), (3302703816, 1), (3578306685, 343), (3626560659, 1), (3723172092, 1), (4110645161, 1), (4264988901, 1)]),
			BTreeMap::<u32, u16>::from([(664932797, 1), (1274634994, 1), (1613811009, 1), (2115173661, 1), (2398318569, 1), (2422621144, 1), (2576555156, 1), (3099073685, 1), (3121389379, 1), (3206047011, 344), (3337913432, 1), (3680497182, 1), (3784430049, 1), (3890590395, 1)]),
			BTreeMap::<u32, u16>::from([(25067069, 1), (221827694, 344), (577876660, 1), (693458998, 1), (996560215, 1), (1052945790, 1), (1140468679, 1), (1467145287, 1), (1470095296, 1), (2770238194, 1), (3072928107, 1), (3473577055, 1), (4065955341, 1), (4289111766, 1)]),
			BTreeMap::<u32, u16>::from([(294451006, 342), (389993814, 1), (1208424117, 1), (1432532069, 1), (1668030188, 1), (1908843606, 1), (2093656043, 1), (2375395713, 1), (2490836552, 1), (2814016504, 1), (3040872152, 1), (3187181585, 1), (3427874584, 1), (3571217567, 1), (3968174155, 1), (4103045741, 1)]),
		];

		assert_eq!(toc.audio_len(), parsed.len());
		assert_eq!(expected, parsed);
	}
}
