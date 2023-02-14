/*!
# CDTOC: Serde
*/

use crate::{
	Duration,
	Toc,
	Track,
	TrackPosition,
};
#[cfg(feature = "accuraterip")] use crate::AccurateRip;
#[cfg(feature = "cddb")] use crate::Cddb;
#[cfg(feature = "sha1")] use crate::ShaB64;
use serde::{
	de,
	Deserialize,
	ser::{
		self,
		SerializeStruct,
	},
	Serialize,
};
use std::fmt;



macro_rules! deserialize_str_with {
	($ty:ty, $fn:ident) => (
		#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
		impl<'de> Deserialize<'de> for $ty {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where D: de::Deserializer<'de> {
				<&str>::deserialize(deserializer).and_then(|s|
					Self::$fn(s).map_err(de::Error::custom)
				)
			}
		}
	);
}

macro_rules! serialize_with {
	($ty:ty, $fn:ident) => (
		#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
		impl Serialize for $ty {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where S: ser::Serializer { self.$fn().serialize(serializer) }
		}
	);
}



#[cfg(feature = "accuraterip")] deserialize_str_with!(AccurateRip, decode);
#[cfg(feature = "accuraterip")] serialize_with!(AccurateRip, pretty_print);

#[cfg(feature = "cddb")] deserialize_str_with!(Cddb, decode);
#[cfg(feature = "cddb")] serialize_with!(Cddb, to_string);

#[cfg(feature = "sha1")] deserialize_str_with!(ShaB64, decode);
#[cfg(feature = "sha1")] serialize_with!(ShaB64, pretty_print);

deserialize_str_with!(Toc, from_cdtoc);
serialize_with!(Toc, to_string);

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for Duration {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: de::Deserializer<'de> {
		u64::deserialize(deserializer).map(Self::from)
	}
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Serialize for Duration {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: ser::Serializer { self.0.serialize(serializer) }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for Track {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: de::Deserializer<'de> {
		const FIELDS: &[&str] = &["num", "pos", "from", "to"];
		struct TrackVisitor;

		impl<'de> de::Visitor<'de> for TrackVisitor {
			type Value = Track;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("struct Track")
			}

			fn visit_seq<V>(self, mut seq: V) -> Result<Track, V::Error>
            where V: de::SeqAccess<'de> {
				let num = seq.next_element()?
					.ok_or_else(|| de::Error::invalid_length(0, &self))?;
				let pos = seq.next_element()?
					.ok_or_else(|| de::Error::invalid_length(1, &self))?;
				let from = seq.next_element()?
					.ok_or_else(|| de::Error::invalid_length(2, &self))?;
				let to = seq.next_element()?
					.ok_or_else(|| de::Error::invalid_length(3, &self))?;
				Ok(Track { num, pos, from, to })
            }

			fn visit_map<V>(self, mut map: V) -> Result<Track, V::Error>
			where V: de::MapAccess<'de> {
				let mut num = None;
				let mut pos = None;
				let mut from = None;
				let mut to = None;

				macro_rules! set {
					($var:ident, $name:literal) => (
						if $var.is_none() { $var.replace(map.next_value()?); }
						else { return Err(de::Error::duplicate_field($name)); }
					);
				}

				while let Some(key) = map.next_key()? {
					match key {
						"num" => set!(num, "num"),
						"pos" => set!(pos, "pos"),
						"from" => set!(from, "from"),
						"to" => set!(to, "to"),
						_ => return Err(de::Error::unknown_field(key, FIELDS)),
					}
				}

				let num = num.ok_or_else(|| de::Error::missing_field("num"))?;
				let pos = pos.ok_or_else(|| de::Error::missing_field("pos"))?;
				let from = from.ok_or_else(|| de::Error::missing_field("from"))?;
				let to = to.ok_or_else(|| de::Error::missing_field("to"))?;

				Ok(Track { num, pos, from, to })
			}
		}

		deserializer.deserialize_struct("Track", FIELDS, TrackVisitor)
	}
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Serialize for Track {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: ser::Serializer {
		let mut state = serializer.serialize_struct("Track", 4)?;

		state.serialize_field("num", &self.num)?;
		state.serialize_field("pos", &self.pos)?;
		state.serialize_field("from", &self.from)?;
		state.serialize_field("to", &self.to)?;

		state.end()
	}
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for TrackPosition {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: de::Deserializer<'de> {
		<&str>::deserialize(deserializer).map(|s| match s {
			"First" => Self::First,
			"Middle" => Self::Middle,
			"Last" => Self::Last,
			"Only" => Self::Only,
			_ => Self::Invalid,
		})
	}
}

serialize_with!(TrackPosition, as_str);



#[cfg(test)]
mod tests {
	use super::*;

	const TOC: &str = "B+96+5DEF+A0F2+F809+1529F+1ACB3+20CBC+24E14+2AF17+2F4EA+35BDD+3B96D";

	/// # Test Serialize->Deserialize Consistency.
	macro_rules! inout {
		($input:ident, $ty:ty, $nice:literal) => (
			let s = serde_json::to_vec(&$input).expect(concat!($nice, " serialize failed."));
			let d = serde_json::from_slice::<$ty>(&s).expect(concat!($nice, " deserialize failed."));
			assert_eq!($input, d, concat!($nice, " JSON serialize/deserialize does not match the original."));
		);
	}

	#[cfg(feature = "accuraterip")]
	#[test]
	fn serde_accuraterip() {
		let accuraterip = Toc::from_cdtoc(TOC).expect("Invalid TOC.").accuraterip_id();
		inout!(accuraterip, AccurateRip, "AccurateRip");
	}

	#[cfg(feature = "cddb")]
	#[test]
	fn serde_cddb() {
		let cddb = Toc::from_cdtoc(TOC).expect("Invalid TOC.").cddb_id();
		inout!(cddb, Cddb, "CDDB");
	}

	#[cfg(feature = "ctdb")]
	#[test]
	fn serde_ctdb() {
		let ctdb = Toc::from_cdtoc(TOC).expect("Invalid TOC.").ctdb_id();
		inout!(ctdb, ShaB64, "ShaB64");
	}

	#[cfg(feature = "musicbrainz")]
	#[test]
	fn serde_musicbrainz() {
		let mb = Toc::from_cdtoc(TOC).expect("Invalid TOC.").musicbrainz_id();
		inout!(mb, ShaB64, "ShaB64");
	}

	#[test]
	fn serde_duration() {
		let duration = Duration::from(123_u32);
		inout!(duration, Duration, "Duration");
	}

	#[test]
	fn serde_toc() {
		let toc = Toc::from_cdtoc(TOC).expect("Invalid TOC.");
		inout!(toc, Toc, "TOC");
	}

	#[test]
	fn serde_tracks() {
		let toc = Toc::from_cdtoc(TOC).expect("Invalid TOC.");
		let tracks: Vec<Track> = toc.audio_tracks().collect();
		inout!(tracks, Vec<Track>, "Track");
	}
}
