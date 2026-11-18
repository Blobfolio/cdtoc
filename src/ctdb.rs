/*!
# CDTOC: CUETools Database
*/

use crate::{
	CDEXTRA_DATA_OFFSET,
	hex,
	ShaB64,
	Toc,
	TocError,
	TocKind,
};
use dactyl::traits::HexToUnsigned;
use std::{
	collections::BTreeMap,
	fmt::NumBuffer,
	num::{
		NonZeroU16,
		NonZeroU8,
	},
};



#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
/// # CTDB Track Checksums.
///
/// A map matching CRC32 checksums to their cumulative confidence.
pub type CtdbTrackChecksums = BTreeMap<u32, NonZeroU16>;



#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
#[derive(Debug, Clone, Eq, PartialEq)]
/// # CUETools Database Lookup.
///
/// This struct collects the entries from a [CUETools Database](https://cue.tools/wiki/CUETools_Database)
/// lookup.
///
/// ## Examples
///
/// ```
/// use cdtoc::{CtdbLookup, Toc};
///
/// // Parse the TOC first.
/// let toc = Toc::from_cdtoc("4+B6+76B6+10635+1575C+1B64A").unwrap();
///
/// // Grab the lookup URL.
/// let url = toc.ctdb_lookup_url();
///
/// // Download the data any which way you want. (This has to be handled
/// // manually.)
/// # let xml = r#"<entry confidence="29" crc32="3f3db655" hasparity="http://p.cuetools.net/4502864" id="4502864" npar="8" stride="5880" syndrome="Ue9G+LuoQwwlwVaPdTQLTA==" toc="32:30240:66975:87750:112052" trackcrcs="5573f647 3a584eec 392a4c51 2c08e2b2"/>"#;
///
/// // Parse it!
/// let lookup = CtdbLookup::from_xml(xml, &toc).unwrap();
/// for e in lookup.entries() {
///     // …
/// }
/// ```
///
/// (Album metadata is not included.)
pub struct CtdbLookup {
	/// # Entries.
	entries: Vec<CtdbLookupEntry>,
}

impl CtdbLookup {
	/// # Parse CUETools Database XML.
	///
	/// Parse and return the entries from an XML CTDB [lookup](Toc::ctdb_checksum_url).
	///
	/// <div class="warning">
	/// <strong>Warning:</strong><br>
	/// This method is highly optimized, but <em>naive</em>,
	/// tailored to the specific formatting of database responses. If the
	/// content is untrusted or restructured, you'll need to parse it yourself.
	/// </div>
	///
	/// ## Errors
	///
	/// This method will return an error if no entries are successfully
	/// parsed.
	pub fn from_xml(xml: &str, toc: &Toc) -> Result<Self, TocError> {
		// It's rare for albums to have more than a couple dozen entries.
		let mut entries = Vec::with_capacity(32);

		for line in XmlEntries::new(xml) {
			if
				let Some(attr) = XmlAttributes::new(line) &&
				let Some(e) = CtdbLookupEntry::from_xml(attr, toc)
			{
				entries.push(e);
			}
		}

		if entries.is_empty() { Err(TocError::CtdbLookup) }
		else { Ok(Self { entries }) }
	}
}

impl CtdbLookup {
	#[must_use]
	/// # Entries.
	///
	/// Return a slice of [`CtdbLookupEntry`].
	pub const fn entries(&self) -> &[CtdbLookupEntry] { self.entries.as_slice() }

	#[must_use]
	/// # Track Checksums.
	///
	/// Return a vector — indexed by track number (`n-1`) — of the combined
	/// `checksum => confidence` pairs found across all entries.
	///
	/// This method is an infallible alternative to [`Toc::ctdb_parse_checksums`].
	pub fn track_checksums(&self) -> Vec<CtdbTrackChecksums> {
		use std::collections::btree_map::Entry;

		// This shouldn't be reachable.
		let entries = self.entries();
		if entries.is_empty() {
			std::hint::cold_path();
			return Vec::new();
		}

		let audio_len = entries[0].track_checksums().len();
		let mut out: Vec<CtdbTrackChecksums> = vec![BTreeMap::default(); audio_len];

		for e in self.entries() {
			let confidence = e.confidence();
			for (combined, chk) in out.iter_mut().zip(e.track_checksums().iter().copied()) {
				match combined.entry(chk) {
					Entry::Vacant(e) => { e.insert(confidence); },
					Entry::Occupied(mut e) => {
						let e = e.get_mut();
						*e = e.saturating_add(confidence.get());
					},
				}
			}
		}

		// Done!
		out
	}
}



#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
#[derive(Debug, Clone, Eq, PartialEq)]
/// # CUETools Database Lookup Entry.
///
/// This struct holds the details for a single entry in a [CUETools Database](https://cue.tools/wiki/CUETools_Database)
/// lookup.
pub struct CtdbLookupEntry {
	/// # Entry ID.
	id: u64,

	/// # Match Confidence.
	confidence: NonZeroU16,

	/// # Has Parity?
	///
	/// If true, this record has an associated parity data file.
	has_parity: bool,

	/// # Parity Settings.
	///
	/// The entry `npar`, `stride`, and `syndrome`.
	parity_settings: CtdbParitySettings,

	/// # Album CRC.
	///
	/// The cumulative CRC32 of the entire album, equivalent to all the track
	/// CRCs added together.
	album_checksum: u32,

	/// # Track CRCs.
	///
	/// This holds a CRC32 for each track on the album, indexed by track number
	/// (`n-1`).
	track_checksums: Vec<u32>,
}

impl CtdbLookupEntry {
	/// # From XML Key/Value Pairs.
	///
	/// Try to parse an entry from an iterator of key/value pairs from the
	/// raw [lookup](Toc::ctdb_checksum_url) XML.
	///
	/// Unexpected keys will be silently skipped, but this will return `None`
	/// if any required ones are missing or their values are malformed.
	fn from_xml(pairs: XmlAttributes<'_>, toc: &Toc) -> Option<Self> {
		// Expected things.
		let mut id = None;
		let mut confidence = None;
		let mut has_parity = None;
		let mut npar = None;
		let mut stride = None;
		let mut syndrome = None;
		let mut album_checksum = None;
		let mut track_checksums = Vec::with_capacity(toc.audio_len());

		// Run through the pairs.
		for (k, v) in pairs {
			match k {
				"id" => { id = v.parse::<u64>().ok(); },
				"confidence" => { confidence = v.parse::<NonZeroU16>().ok(); },
				"hasparity" => {
					// This should end with the ID.
					let v = v.strip_prefix("http://p.cuetools.net/")
						.or_else(|| v.strip_prefix("http://p.cuetools.net/"))
						.and_then(|v| v.parse::<u64>().ok())?;
					has_parity.replace(v);
				},
				"npar" => { npar = v.parse::<NonZeroU8>().ok(); },
				"stride" => { stride = v.parse::<NonZeroU16>().ok(); },
				"syndrome" => { syndrome.replace(v); },
				"crc32" => { album_checksum = u32::htou(v.as_bytes()); },
				"trackcrcs" => {
					for v in v.split_ascii_whitespace() {
						let v = u32::htou(v.as_bytes())?;
						track_checksums.push(v);
					}
				},
				"toc" if ! verify_toc_loose(v, toc) => return None,
				_ => {},
			}
		}

		// We should have everything by this point.
		if track_checksums.len() != toc.audio_len() { return None; }
		let id = id?;
		let confidence = confidence?;
		let npar = npar?;
		let has_parity = Some(id) == has_parity;
		let stride = stride?;
		let syndrome = syndrome?;
		let parity_settings = CtdbParitySettings::new(npar, stride, syndrome)?;
		let album_checksum = album_checksum?;

		// Done!
		Some(Self {
			id,
			confidence,
			has_parity,
			parity_settings,
			album_checksum,
			track_checksums,
		})
	}
}

impl CtdbLookupEntry {
	#[must_use]
	/// # Album Checksum.
	///
	/// Return the CRC32 checksum corresponding to the entire album.
	pub const fn album_checksum(&self) -> u32 { self.album_checksum }

	#[must_use]
	/// # Confidence.
	///
	/// Return the "confidence" associated with this entry's checksums.
	pub const fn confidence(&self) -> NonZeroU16 { self.confidence }

	#[must_use]
	/// # Entry ID.
	///
	/// Return the ID for the entry.
	pub const fn id(&self) -> u64 { self.id }

	#[must_use]
	/// # Parity Data URL.
	///
	/// Return the download URL for the associated parity data, if any.
	///
	/// Note that while many entries can share the same checksums/syndrome,
	/// the data will only be attached to one of them.
	pub fn parity_data_url(&self) -> Option<String> {
		if self.has_parity { Some(format!("http://p.cuetools.net/{}", self.id())) }
		else { None }
	}

	#[must_use]
	/// # Parity Settings.
	///
	/// Return the parity size, stride, and syndrome.
	pub const fn parity_settings(&self) -> CtdbParitySettings { self.parity_settings }

	#[must_use]
	/// # Track Checksum.
	///
	/// Return the CRC32 checksum corresponding to a specific track. Returns
	/// `None` if out of range.
	///
	/// Note: CTDB does not check HTOA tracks.
	pub const fn track_checksum(&self, track: u8) -> Option<u32> {
		let slice = self.track_checksums();
		if let Some(idx) = track.checked_sub(1) && (idx as usize) < slice.len() {
			Some(slice[idx as usize])
		}
		else { None }
	}

	#[must_use]
	/// # Track Checksums.
	///
	/// Return a slice containing the checksum for each track, indexed by track
	/// number (`n-1`).
	///
	/// Note: CTDB does not check HTOA tracks.
	pub const fn track_checksums(&self) -> &[u32] { self.track_checksums.as_slice() }
}



#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// # CUETools Parity.
///
/// This struct holds the parity size, stride, and syndrome data for a single
/// [CUETools Database](https://cue.tools/wiki/CUETools_Database) lookup entry.
///
/// Note that only records with "npar" values of 8 and 16 are supported. (It's
/// unclear if other sizes are even used?)
pub enum CtdbParitySettings {
	/// # Parity Size: 8.
	///
	/// This is used for `[u16; 8]` syndromes.
	Npar8(CtdbParitySettingsSized<8>),

	/// # Parity Size: 16.
	///
	/// This is used for `[u16; 16]` syndromes.
	Npar16(CtdbParitySettingsSized<16>),
}

impl CtdbParitySettings {
	#[must_use]
	/// # New.
	///
	/// This method decodes the base64-encoded syndrome label, returning a new
	/// instance if valid.
	const fn new(
		npar: NonZeroU8,
		stride: NonZeroU16,
		syndrome: &str,
	) -> Option<Self> {
		// Eight.
		if npar.get() == 8 {
			if let Some(syndrome) = CtdbParitySettingsSized::<8>::decode_syndrome(syndrome) {
				return Some(Self::Npar8(CtdbParitySettingsSized {
					stride,
					syndrome,
				}));
			}
		}

		// Sixteen.
		else if
			npar.get() == 16 &&
			let Some(syndrome) = CtdbParitySettingsSized::<16>::decode_syndrome(syndrome)
		{
			return Some(Self::Npar16(CtdbParitySettingsSized {
				stride,
				syndrome,
			}));
		}

		// Dunno!
		None
	}
}

impl CtdbParitySettings {
	#[must_use]
	/// # Parity Size.
	///
	/// Return the number of parity symbols, either eight or sixteen.
	pub const fn npar(&self) -> NonZeroU8 {
		match self {
			Self::Npar8(_) => const { NonZeroU8::new(8).unwrap() },
			Self::Npar16(_) => const { NonZeroU8::new(16).unwrap() },
		}
	}

	#[must_use]
	/// # Parity Stride.
	///
	/// Return the parity stride, usually `5880`.
	pub const fn stride(&self) -> NonZeroU16 {
		match self {
			Self::Npar8(v) => v.stride(),
			Self::Npar16(v) => v.stride(),
		}
	}

	#[must_use]
	/// # Syndrome.
	///
	/// Return the syndrome as a slice of little endian `u16` symbols.
	///
	/// The length will always match `self.npar()`. If you need an array
	/// representation instead, destructure the enum and call `syndrome()` on
	/// the inner value.
	pub const fn syndrome(&self) -> &[u16] {
		match self {
			Self::Npar8(v) => v.syndrome.as_slice(),
			Self::Npar16(v) => v.syndrome.as_slice(),
		}
	}
}



#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// # CUETools Parity (Inner).
///
/// This holds the size-specific data embedded by the variants of the
/// [`CtdbParitySettings`] enum.
pub struct CtdbParitySettingsSized<const N: usize> {
	/// # Stride.
	stride: NonZeroU16,

	/// # Syndrome.
	syndrome: [u16; N],
}

impl<const N: usize> CtdbParitySettingsSized<N> {
	#[expect(clippy::cast_possible_truncation, reason = "False positive.")]
	#[must_use]
	/// # Parity Size.
	///
	/// Return the number of parity symbols, either eight or sixteen.
	pub const fn npar() -> NonZeroU8 {
		const {
			assert!(N == 8 || N == 16, "BUG: parity size must be 8 or 16.");
		}

		const { NonZeroU8::new(N as u8).unwrap() }
	}

	#[must_use]
	/// # Parity Stride.
	///
	/// Return the parity stride, usually `5880`.
	pub const fn stride(&self) -> NonZeroU16 { self.stride }

	#[must_use]
	/// # Syndrome.
	///
	/// Return the syndrome as an array of little endian `u16` symbols.
	pub const fn syndrome(&self) -> [u16; N] { self.syndrome }
}

impl CtdbParitySettingsSized<8> {
	/// # Syndrome From Base64.
	///
	/// The XML representation uses the standard base64 alphabet, so decoding
	/// is straightforward, and because the sizing is fixed, we can be explicit
	/// with it.
	const fn decode_syndrome(raw: &str) -> Option<[u16; 8]> {
		let [ raw @ .., b'=', b'=' ] = raw.as_bytes() else { return None; };
		let Some(raw) = raw.as_array::<22>() else { return None; };
		let Some(raw) = unmap_syndrome_base64(raw) else { return None; };

		Some([
			u16::from_le_bytes([
				((raw[0] & 0b0011_1111) << 2) | (raw[1] >> 4),
				((raw[1] & 0b0000_1111) << 4) | (raw[2] >> 2),
			]),
			u16::from_le_bytes([
				((raw[2] & 0b0000_0011) << 6) | raw[3] & 0b0011_1111,
				((raw[4] & 0b0011_1111) << 2) | (raw[5] >> 4),
			]),
			u16::from_le_bytes([
				((raw[5] & 0b0000_1111) << 4) | (raw[6] >> 2),
				((raw[6] & 0b0000_0011) << 6) | raw[7] & 0b0011_1111,
			]),
			u16::from_le_bytes([
				((raw[8] & 0b0011_1111) << 2) | (raw[9] >> 4),
				((raw[9] & 0b0000_1111) << 4) | (raw[10] >> 2),
			]),
			u16::from_le_bytes([
				((raw[10] & 0b0000_0011) << 6) | raw[11] & 0b0011_1111,
				((raw[12] & 0b0011_1111) << 2) | (raw[13] >> 4),
			]),
			u16::from_le_bytes([
				((raw[13] & 0b0000_1111) << 4) | (raw[14] >> 2),
				((raw[14] & 0b0000_0011) << 6) | raw[15] & 0b0011_1111,
			]),
			u16::from_le_bytes([
				((raw[16] & 0b0011_1111) << 2) | (raw[17] >> 4),
				((raw[17] & 0b0000_1111) << 4) | (raw[18] >> 2),
			]),
			u16::from_le_bytes([
				((raw[18] & 0b0000_0011) << 6) | raw[19] & 0b0011_1111,
				((raw[20] & 0b0011_1111) << 2) | (raw[21] >> 4),
			]),
		])
	}
}

impl CtdbParitySettingsSized<16> {
	/// # Syndrome From Base64.
	///
	/// The XML representation uses the standard base64 alphabet, so decoding
	/// is straightforward, and because the sizing is fixed, we can be explicit
	/// with it.
	const fn decode_syndrome(raw: &str) -> Option<[u16; 16]> {
		let [ raw @ .., b'=' ] = raw.as_bytes() else { return None; };
		let Some(raw) = raw.as_array::<43>() else { return None; };
		let Some(raw) = unmap_syndrome_base64(raw) else { return None; };

		Some([
			u16::from_le_bytes([
				((raw[0] & 0b0011_1111) << 2) | (raw[1] >> 4),
				((raw[1] & 0b0000_1111) << 4) | (raw[2] >> 2),
			]),
			u16::from_le_bytes([
				((raw[2] & 0b0000_0011) << 6) | raw[3] & 0b0011_1111,
				((raw[4] & 0b0011_1111) << 2) | (raw[5] >> 4),
			]),
			u16::from_le_bytes([
				((raw[5] & 0b0000_1111) << 4) | (raw[6] >> 2),
				((raw[6] & 0b0000_0011) << 6) | raw[7] & 0b0011_1111,
			]),
			u16::from_le_bytes([
				((raw[8] & 0b0011_1111) << 2) | (raw[9] >> 4),
				((raw[9] & 0b0000_1111) << 4) | (raw[10] >> 2),
			]),
			u16::from_le_bytes([
				((raw[10] & 0b0000_0011) << 6) | raw[11] & 0b0011_1111,
				((raw[12] & 0b0011_1111) << 2) | (raw[13] >> 4),
			]),
			u16::from_le_bytes([
				((raw[13] & 0b0000_1111) << 4) | (raw[14] >> 2),
				((raw[14] & 0b0000_0011) << 6) | raw[15] & 0b0011_1111,
			]),
			u16::from_le_bytes([
				((raw[16] & 0b0011_1111) << 2) | (raw[17] >> 4),
				((raw[17] & 0b0000_1111) << 4) | (raw[18] >> 2),
			]),
			u16::from_le_bytes([
				((raw[18] & 0b0000_0011) << 6) | raw[19] & 0b0011_1111,
				((raw[20] & 0b0011_1111) << 2) | (raw[21] >> 4),
			]),
			u16::from_le_bytes([
				((raw[21] & 0b0000_1111) << 4) | (raw[22] >> 2),
				((raw[22] & 0b0000_0011) << 6) | raw[23] & 0b0011_1111,
			]),
			u16::from_le_bytes([
				((raw[24] & 0b0011_1111) << 2) | (raw[25] >> 4),
				((raw[25] & 0b0000_1111) << 4) | (raw[26] >> 2),
			]),
			u16::from_le_bytes([
				((raw[26] & 0b0000_0011) << 6) | raw[27] & 0b0011_1111,
				((raw[28] & 0b0011_1111) << 2) | (raw[29] >> 4),
			]),
			u16::from_le_bytes([
				((raw[29] & 0b0000_1111) << 4) | (raw[30] >> 2),
				((raw[30] & 0b0000_0011) << 6) | raw[31] & 0b0011_1111,
			]),
			u16::from_le_bytes([
				((raw[32] & 0b0011_1111) << 2) | (raw[33] >> 4),
				((raw[33] & 0b0000_1111) << 4) | (raw[34] >> 2),
			]),
			u16::from_le_bytes([
				((raw[34] & 0b0000_0011) << 6) | raw[35] & 0b0011_1111,
				((raw[36] & 0b0011_1111) << 2) | (raw[37] >> 4),
			]),
			u16::from_le_bytes([
				((raw[37] & 0b0000_1111) << 4) | (raw[38] >> 2),
				((raw[38] & 0b0000_0011) << 6) | raw[39] & 0b0011_1111,
			]),
			u16::from_le_bytes([
				((raw[40] & 0b0011_1111) << 2) | (raw[41] >> 4),
				((raw[41] & 0b0000_1111) << 4) | (raw[42] >> 2),
			]),
		])
	}
}



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
	#[inline]
	#[must_use]
	/// # CUETools Database Checksum URL.
	///
	/// This is an alias for [`Toc::ctdb_lookup_url`].
	pub fn ctdb_checksum_url(&self) -> String { self.ctdb_lookup_url() }

	#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
	#[must_use]
	/// # CUETools Database Lookup URL.
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
	pub fn ctdb_lookup_url(&self) -> String {
		// We can't efficiently precalculate the exact size, but the next
		// power-of-two isn't too far away, so we might as well start there.
		let mut url = String::with_capacity(128);
		url.push_str("https://db.cue.tools/lookup2.php?version=3&ctdb=1&fuzzy=1&toc=");

		// Digit buffer.
		let mut buf = NumBuffer::<u32>::new();

		// Leading data?
		if matches!(self.kind, TocKind::DataFirst) {
			url.push('-');
			url.push_str((self.data - 150).format_into(&mut buf));
			url.push(':');
		}

		// Each audio track relative to the first.
		for v in &self.audio {
			url.push_str((v - 150).format_into(&mut buf));
			url.push(':');
		}

		// Trailing data?
		if matches!(self.kind, TocKind::CDExtra) {
			url.push('-');
			url.push_str((self.data - 150).format_into(&mut buf));
			url.push(':');
		}

		// And the leadout.
		url.push_str((self.leadout - 150).format_into(&mut buf));

		url
	}

	#[cfg_attr(docsrs, doc(cfg(feature = "ctdb")))]
	/// # Parse Checksums.
	///
	/// This will parse the track checksums from an XML CTDB [lookup](Toc::ctdb_checksum_url),
	/// returning a vector — indexed by track number (`n-1`) — of the
	/// combined `checksum => confidence` pairs found across all entries.
	///
	/// <div class="warning">
	/// <strong>Warning:</strong><br>
	/// This method is highly optimized, but <em>naive</em>,
	/// tailored to the specific formatting of database responses. If the
	/// content is untrusted or restructured, you'll need to parse it yourself.
	/// </div>
	///
	/// If you need other entry-related details too, use [`CtdbLookup::from_xml`]
	/// instead to avoid double-parsing.
	///
	/// ## Errors
	///
	/// This method will return an error if no checksums are found.
	pub fn ctdb_parse_checksums(&self, xml: &str) -> Result<Vec<CtdbTrackChecksums>, TocError> {
		CtdbLookup::from_xml(xml, self).map_or(
			Err(TocError::NoChecksums),
			|v| Ok(v.track_checksums())
		)
	}
}



#[must_use]
/// # Unamp Syndrome Base64.
///
/// This method unmaps the values in a (standard alphabet) base64-encoded array
/// so that they can be shifted and ORed and ANDed back into the original input
/// values.
const fn unmap_syndrome_base64<const N: usize>(enc: &[u8; N]) -> Option<[u8; N]> {
	let mut out = [0_u8; N];
	let mut i = 0;
	while i < N {
		out[i] = match enc[i] {
			n @ b'A'..=b'Z' => n - 65,
			n @ b'a'..=b'z' => n - 71,
			n @ b'0'..=b'9' => n + 4,
			b'+' => 62,
			b'/' => 63,
			_ => return None,
		};
		i += 1;
	}
	Some(out)
}

#[must_use]
/// # Matches Record TOC (Loose).
///
/// This method compares the `"toc"` from a CTDB lookup entry with the
/// reference [`Toc`], returning true if all the audio tracks have the expected
/// length.
fn verify_toc_loose(raw: &str, toc: &Toc) -> bool {
	/// # Audio or Data Track Position.
	enum AudioOrData {
		Audio(u32),
		Data(u32),
		Invalid,
	}

	impl AudioOrData {
		/// # Parse.
		fn parse(raw: &str) -> Self {
			// Data locations are prefixed with a minus sign.
			raw.strip_prefix('-').map_or_else(
				|| raw.parse::<u32>().ok().map_or(Self::Invalid, Self::Audio),
				|v| v.parse::<u32>().ok().map_or(Self::Invalid, Self::Data),
			)
		}
	}

	// Positions are separated by colons.
	let mut iter = raw.split(':').map(AudioOrData::parse);

	// Pull the start position of the first audio track.
	let (data_first, mut last) = match iter.next() {
		Some(AudioOrData::Audio(v)) => (false, v),
		// If the first entry is data, we can ignore it and pull the next-next
		// instead, which must be audio.
		Some(AudioOrData::Data(_)) if let Some(AudioOrData::Audio(v)) = iter.next() => (true, v),
		_ => return false,
	};

	// Calculate and compare all the track lengths!
	let mut tracks = toc.audio_tracks();
	while let Some(next) = iter.next() {
		// Pull the next position.
		let (cdextra, next) = match next {
			AudioOrData::Audio(v) => (false, v),
			// A CD-Text position indicates the end of the audio. Convert it
			// to a proper leadout value.
			AudioOrData::Data(v) if ! data_first && let Some(v) = v.checked_sub(CDEXTRA_DATA_OFFSET) => (true, v),
			_ => return false,
		};

		// Calculate the track length (and implicitly verify ordering).
		let Some(len) = next.checked_sub(last) else { return false; };

		if
			// Length mismatch.
			tracks.next().is_none_or(|v| v.sectors() != len) ||

			// If this was a CD-Extra position, we're out of audio, but need
			// to verify there's exactly one position (data leadout) left
			// before we call it good.
			(cdextra && iter.by_ref().count() != 1)
		{
			return false;
		}

		// Reset the "last" value for the next go-around.
		last = next;
	}

	// Unless there should have been more tracks, we matched!
	tracks.next().is_none()
}



/// # XML Entry Iterator.
///
/// This is a simple and naive XML "parser" for CTDB lookup responses, yielding
/// the "<entry …/>" matches therein. It would fail miserably were the data
/// more haphazardly formatted, but for our purposes, more robust parsing is
/// out of scope.
struct XmlEntries<'a>(&'a str);

impl<'a> XmlEntries<'a> {
	/// # New.
	const fn new(src: &'a str) -> Self { Self(src) }
}

impl<'a> Iterator for XmlEntries<'a> {
	type Item = &'a str;

	fn next(&mut self) -> Option<Self::Item> {
		// Find the start of the next entry, if any, and chop self to that.
		let start = self.0.find("<entry ")?;
		self.0 = &self.0[start..];

		// Find the close and split again.
		let end = self.0.find("/>")?;
		let (next, rest) = self.0.split_at(end + 2);
		self.0 = rest;

		// There shouldn't be any '<' characters in our tag other than the
		// opening one. If there are, drop the stub and recurse.
		if next.chars().skip(1).any(|v| v == '<') {
			std::hint::cold_path();
			return self.next();
		}

		Some(next)
	}
}



/// # XML Attribute Iterator.
///
/// This is a simple and naive XML "parser" for CTDB entry lines, yielding
/// key/value pairs. It would fail miserably were the data more haphazardly
/// formatted, but for our purposes, anything more robust would be overkill.
struct XmlAttributes<'a>(&'a str);

impl<'a> XmlAttributes<'a> {
	/// # New.
	fn new(line: &'a str) -> Option<Self> {
		if
			let Some(line) = line.trim().strip_prefix("<entry ") &&
			let Some(line) = line.strip_suffix("/>") &&
			line.is_ascii()
		{
			Some(Self(line.trim()))
		}
		// We're extracting tags based on the same prefix/suffix, so this
		// shouldn't fail.
		else {
			std::hint::cold_path();
			None
		}
	}
}

impl<'a> Iterator for XmlAttributes<'a> {
	type Item = (&'a str, &'a str);

	fn next(&mut self) -> Option<Self::Item> {
		// Equal signs can be inside quoted values, but the _first_ should
		// always correspond to a key=value.
		let (k, rest) = self.0.split_once('=')?;

		// Without going overboard, let's make sure the key is ASCII
		// alphanumeric, '_', or ':'.
		let k = k.trim_end();
		if k.is_empty() || ! k.chars().all(|c| matches!(c, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' | ':')) {
			return None;
		}

		// All values should be quoted.
		let rest = rest.trim_start().strip_prefix('"')?;
		let (v, rest) = rest.split_once('"')?;
		let v = v.trim_end();

		// Reset the internal string to the remainder.
		self.0 = rest.trim_start();

		// Return the key and value.
		Some((k, v))
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
	fn t_ctdb_parse_checksums() {
		let xml = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("skel/ctdb.xml");
		let xml = std::fs::read_to_string(&xml).expect("Unable to read CTDB XML.");

		let toc = Toc::from_cdtoc("F+96+4173+8BDF+C8C7+11443+15E6B+192F6+1DB52+21D82+26CD7+2B227+3084B+35C25+39A3F+3D63F+449EF+4E4AB").expect("Invalid TOC");
		let parsed = toc.ctdb_parse_checksums(&xml).expect("Unable to parse checksums");
		let expected = vec![
			CtdbTrackChecksums::from([(763282391, NonZeroU16::MIN), (897588894, NonZeroU16::MIN), (1235037124, NonZeroU16::MIN), (1282249905, NonZeroU16::new(349).unwrap()), (3152505343, NonZeroU16::MIN), (3258244565, NonZeroU16::MIN), (3433605098, NonZeroU16::MIN), (3433918662, NonZeroU16::MIN), (3700015550, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(12391530, NonZeroU16::MIN), (254318592, NonZeroU16::MIN), (278329866, NonZeroU16::MIN), (1024246154, NonZeroU16::MIN), (1270661992, NonZeroU16::MIN), (1443436445, NonZeroU16::MIN), (2588556635, NonZeroU16::MIN), (2609626001, NonZeroU16::MIN), (3172155240, NonZeroU16::new(348).unwrap()), (3507001436, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(79025172, NonZeroU16::MIN), (576176869, NonZeroU16::new(350).unwrap()), (1124109082, NonZeroU16::MIN), (1202985854, NonZeroU16::MIN), (1920914588, NonZeroU16::MIN), (1936773867, NonZeroU16::MIN), (3216934765, NonZeroU16::MIN), (4144397645, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(1087690482, NonZeroU16::MIN), (1089145401, NonZeroU16::MIN), (2243388636, NonZeroU16::MIN), (2618259125, NonZeroU16::MIN), (2968527536, NonZeroU16::MIN), (3345952562, NonZeroU16::new(349).unwrap()), (3498817224, NonZeroU16::MIN), (3856655528, NonZeroU16::MIN), (3901531533, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(194084823, NonZeroU16::MIN), (538997881, NonZeroU16::MIN), (1052544287, NonZeroU16::MIN), (1124555716, NonZeroU16::MIN), (1314115734, NonZeroU16::MIN), (1495657225, NonZeroU16::MIN), (1872765379, NonZeroU16::MIN), (2380642597, NonZeroU16::MIN), (2899014380, NonZeroU16::new(348).unwrap()), (3224619805, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(1083862139, NonZeroU16::new(351).unwrap()), (1370448129, NonZeroU16::MIN), (1715012676, NonZeroU16::MIN), (1749827653, NonZeroU16::MIN), (2019350366, NonZeroU16::MIN), (2081480574, NonZeroU16::MIN), (3961881527, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(501630488, NonZeroU16::MIN), (1403870687, NonZeroU16::MIN), (1818657605, NonZeroU16::MIN), (2052363808, NonZeroU16::new(349).unwrap()), (2233541139, NonZeroU16::MIN), (3001272542, NonZeroU16::MIN), (3702882640, NonZeroU16::MIN), (3790048568, NonZeroU16::MIN), (3830547781, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(68973958, NonZeroU16::MIN), (109342161, NonZeroU16::new(349).unwrap()), (623696074, NonZeroU16::MIN), (660407935, NonZeroU16::MIN), (844055799, NonZeroU16::MIN), (1340619147, NonZeroU16::MIN), (3367889202, NonZeroU16::MIN), (3455769741, NonZeroU16::MIN), (4292440658, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(177411350, NonZeroU16::MIN), (346112569, NonZeroU16::MIN), (1088872326, NonZeroU16::MIN), (1578224278, NonZeroU16::MIN), (1981717760, NonZeroU16::MIN), (2149128210, NonZeroU16::MIN), (3308254768, NonZeroU16::MIN), (3564527459, NonZeroU16::new(349).unwrap()), (3833535237, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(2294948521, NonZeroU16::new(348).unwrap()), (2450394573, NonZeroU16::MIN), (2777818443, NonZeroU16::MIN), (2793045780, NonZeroU16::MIN), (2930033218, NonZeroU16::MIN), (2939809789, NonZeroU16::MIN), (3144352102, NonZeroU16::MIN), (3236236899, NonZeroU16::MIN), (3386702153, NonZeroU16::MIN), (4059469375, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(187554328, NonZeroU16::MIN), (692911428, NonZeroU16::MIN), (788436667, NonZeroU16::MIN), (1223347259, NonZeroU16::MIN), (1376669804, NonZeroU16::new(345).unwrap()), (1527008808, NonZeroU16::MIN), (2152877939, NonZeroU16::MIN), (2366724378, NonZeroU16::MIN), (2675505606, NonZeroU16::MIN), (3097721897, NonZeroU16::MIN), (3343301748, NonZeroU16::MIN), (3511184516, NonZeroU16::MIN), (3843536879, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(12734792, NonZeroU16::MIN), (86535921, NonZeroU16::MIN), (616290089, NonZeroU16::MIN), (684539636, NonZeroU16::MIN), (1281593774, NonZeroU16::MIN), (1812856593, NonZeroU16::MIN), (2135611934, NonZeroU16::MIN), (2212148269, NonZeroU16::MIN), (2745468274, NonZeroU16::MIN), (3302703816, NonZeroU16::MIN), (3578306685, NonZeroU16::new(343).unwrap()), (3626560659, NonZeroU16::MIN), (3723172092, NonZeroU16::MIN), (4110645161, NonZeroU16::MIN), (4264988901, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(664932797, NonZeroU16::MIN), (1274634994, NonZeroU16::MIN), (1613811009, NonZeroU16::MIN), (2115173661, NonZeroU16::MIN), (2398318569, NonZeroU16::MIN), (2422621144, NonZeroU16::MIN), (2576555156, NonZeroU16::MIN), (3099073685, NonZeroU16::MIN), (3121389379, NonZeroU16::MIN), (3206047011, NonZeroU16::new(344).unwrap()), (3337913432, NonZeroU16::MIN), (3680497182, NonZeroU16::MIN), (3784430049, NonZeroU16::MIN), (3890590395, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(25067069, NonZeroU16::MIN), (221827694, NonZeroU16::new(344).unwrap()), (577876660, NonZeroU16::MIN), (693458998, NonZeroU16::MIN), (996560215, NonZeroU16::MIN), (1052945790, NonZeroU16::MIN), (1140468679, NonZeroU16::MIN), (1467145287, NonZeroU16::MIN), (1470095296, NonZeroU16::MIN), (2770238194, NonZeroU16::MIN), (3072928107, NonZeroU16::MIN), (3473577055, NonZeroU16::MIN), (4065955341, NonZeroU16::MIN), (4289111766, NonZeroU16::MIN)]),
			CtdbTrackChecksums::from([(294451006, NonZeroU16::new(342).unwrap()), (389993814, NonZeroU16::MIN), (1208424117, NonZeroU16::MIN), (1432532069, NonZeroU16::MIN), (1668030188, NonZeroU16::MIN), (1908843606, NonZeroU16::MIN), (2093656043, NonZeroU16::MIN), (2375395713, NonZeroU16::MIN), (2490836552, NonZeroU16::MIN), (2814016504, NonZeroU16::MIN), (3040872152, NonZeroU16::MIN), (3187181585, NonZeroU16::MIN), (3427874584, NonZeroU16::MIN), (3571217567, NonZeroU16::MIN), (3968174155, NonZeroU16::MIN), (4103045741, NonZeroU16::MIN)]),
		];

		assert_eq!(toc.audio_len(), parsed.len());
		assert_eq!(expected, parsed);
	}

	#[test]
	fn t_lookup() {
		const CRCS: [u32; 15] = [1282249905, 3172155240, 576176869, 3345952562, 2899014380, 1083862139, 2052363808, 109342161, 3564527459, 2294948521, 1376669804, 3578306685, 3206047011, 221827694, 294451006];

		let toc = Toc::from_cdtoc("F+96+4173+8BDF+C8C7+11443+15E6B+192F6+1DB52+21D82+26CD7+2B227+3084B+35C25+39A3F+3D63F+449EF+4E4AB").expect("Invalid TOC");
		let xml = r#"
		<entry confidence="322" crc32="b4d23c4c" hasparity="http://p.cuetools.net/66312" id="66312" npar="16" stride="5880" syndrome="P7wQVtwc6E1hkDMhoSti83FUPPUo4BoWh0Kypqj+EOM=" toc="0:16605:35657:51249:70573:89557:103008:121532:138476:158785:176529:198581:220047:235945:251305:-280921:320533" trackcrcs="4c6d94b1 bd133f68 2257c2e5 c76f2f32 accb72ec 409a6c7b 7a549620 06846dd1 d4765f63 88ca22a9 520e506c d548a07d bf186523 0d38d26e 118cf73e"/>
		<entry confidence="3" crc32="b4d23c4c" hasparity="http://p.cuetools.net/290640" id="290640" npar="8" stride="5880" syndrome="P7wQVtwc6E1hkDMhoSti8w==" toc="0:16605:35657:51249:70573:89557:103008:121532:138476:158785:176529:198581:220047:235945:251305:269521" trackcrcs="4c6d94b1 bd133f68 2257c2e5 c76f2f32 accb72ec 409a6c7b 7a549620 06846dd1 d4765f63 88ca22a9 520e506c d548a07d bf186523 0d38d26e 118cf73e"/>
		"#;
		let lookup = CtdbLookup::from_xml(xml, &toc).expect("Failed to parse lookup.");
		let entries = lookup.entries();
		assert_eq!(entries.len(), 2);

		// First entry.
		assert_eq!(
			entries[0].album_checksum(),
			3_033_676_876,
		);
		assert_eq!(
			entries[0].confidence(),
			NonZeroU16::new(322).unwrap(),
		);
		assert_eq!(
			entries[0].id(),
			66_312,
		);
		assert_eq!(
			entries[0].parity_settings(),
			CtdbParitySettings::Npar16(CtdbParitySettingsSized {
				stride: NonZeroU16::new(5880).unwrap(),
				syndrome: [48191, 22032, 7388, 19944, 36961, 8499, 11169, 62306, 21617, 62780, 57384, 5658, 17031, 42674, 65192, 58128],
			}),
		);
		assert_eq!(
			entries[0].parity_data_url().as_deref(),
			Some("http://p.cuetools.net/66312"),
		);
		assert_eq!(
			entries[0].track_checksums(),
			CRCS,
		);

		// Second entry.
		assert_eq!(
			entries[1].album_checksum(),
			3_033_676_876,
		);
		assert_eq!(
			entries[1].confidence(),
			NonZeroU16::new(3).unwrap(),
		);
		assert_eq!(
			entries[1].id(),
			290_640,
		);
		assert_eq!(
			entries[1].parity_settings(),
			CtdbParitySettings::Npar8(CtdbParitySettingsSized {
				stride: NonZeroU16::new(5880).unwrap(),
				syndrome: [48191, 22032, 7388, 19944, 36961, 8499, 11169, 62306],
			}),
		);
		assert_eq!(
			entries[1].parity_data_url().as_deref(),
			Some("http://p.cuetools.net/290640"),
		);
		assert_eq!(
			entries[1].track_checksums(),
			CRCS,
		);

		// These entries have the same checksums, so we only need to loop
		// through one of them to verify the per-track fetching works.
		assert!(entries[1].track_checksum(0).is_none());
		for (idx, crc) in CRCS.iter().copied().enumerate() {
			assert_eq!(
				entries[1].track_checksum(idx as u8 + 1),
				Some(crc),
			);
		}
		assert!(entries[1].track_checksum(16).is_none());
	}

	#[test]
	fn t_verify_toc_loose() {
		let toc = Toc::from_cdtoc("F+96+4173+8BDF+C8C7+11443+15E6B+192F6+1DB52+21D82+26CD7+2B227+3084B+35C25+39A3F+3D63F+449EF+4E4AB").expect("Invalid TOC");
		for raw in [
			"0:16605:35657:51249:70573:89557:103008:121532:138476:158785:176529:198581:220047:235945:251305:-280921:320532",
			"0:16605:35657:51249:70573:89557:103008:121532:138476:158785:176529:198581:220047:235945:251305:-280921:320533",
			"0:16605:35657:51249:70573:89557:103008:121532:138476:158785:176529:198581:220047:235945:251305:269521",
			"1:16606:35658:51250:70574:89558:103009:121533:138477:158786:176530:198582:220048:235946:251306:269522",
		] {
			assert!(verify_toc_loose(raw, &toc));
		}

		assert!(! verify_toc_loose(
			"32:16605:35657:51249:70573:89557:103008:121532:138476:158785:176529:198581:220047:235945:251305:-280921:320532",
			&toc,
		));
		assert!(! verify_toc_loose(
			"0:35657:51249:70573:89557:103008:121532:138476:158785:176529:198581:220047:235945:251305:-280921:320532",
			&toc,
		));
		assert!(! verify_toc_loose(
			"0:16000:16605:35657:51249:70573:89557:103008:121532:138476:158785:176529:198581:220047:235945:251305:-280921:320532",
			&toc,
		));
	}
}
