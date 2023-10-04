# Changelog


## [0.3.0](https://github.com/Blobfolio/cdtoc/releases/tag/v0.3.0) - 2023-10-03

### New

* `AccurateRip::DRIVE_OFFSET_URL`
* `AccurateRip::parse_drive_offsets`



## [0.2.3](https://github.com/Blobfolio/cdtoc/releases/tag/v0.2.3) - 2023-09-27

### New

* `Toc::audio_leadin_normalized`
* `Toc::audio_leadout_normalized`
* `Toc::data_sector_normalized`
* `Toc::leadin_normalized`
* `Toc::leadout_normalized`



## [0.2.2](https://github.com/Blobfolio/cdtoc/releases/tag/v0.2.2) - 2023-09-12

### New

* `Toc::htoa`
* `Track::is_htoa`



## [0.2.1](https://github.com/Blobfolio/cdtoc/releases/tag/v0.2.1) - 2023-06-25

### New

* `Track::msf`
* `Track::msf_normalized`
* `Track::sector_range_normalized`

### Changed

* Bump `faster-hex` to `0.8`



## [0.2.0](https://github.com/Blobfolio/cdtoc/releases/tag/v0.2.0) - 2023-06-01

### Changed

* Bump MSRV `1.70`
* Replace some `unsafe` code blocks with safe alternatives
* Add debug/assertions for logical redundancy
* Improve unit test coverage
* Update dependencies



## [0.1.8](https://github.com/Blobfolio/cdtoc/releases/tag/v0.1.8) - 2023-04-20

### Changed

* Minor code cleanup



## [0.1.7](https://github.com/Blobfolio/cdtoc/releases/tag/v0.1.7) - 2023-02-15

### Changed

* Add `visit_seq` deserializer support for `Track`
* Require `faster-hex` dependency (it was previously optional)
* Improved hex decode/encode performance
* Improved base64 decode/encode performance for CTDB/MusicBrainz IDs

### New

* Add "asm" crate feature, disabled by default, a passthru for [sha1/asm](https://github.com/RustCrypto/hashes/blob/master/sha1/Cargo.toml#L20)
* Add `FromStr` and `TryFrom` aliases for `AccurateRip::decode`
* Add `FromStr` and `TryFrom` aliases for `Cddb::decode`
* Add `FromStr` and `TryFrom` aliases for `ShaB64::decode`

### Removed

* Obsolete "faster-hex" crate feature
* Optional "base64" crate feature



## [0.1.6](https://github.com/Blobfolio/cdtoc/releases/tag/v0.1.6) - 2023-02-04

### Changed

* Improve docs.rs environment detection
* Declare "faster-hex" feature explicitly

### New

* `AccurateRip::decode`
* `Cddb::decode`
* Return copy-friendly type for MusicBrainz and CTDB ID methods
* Added `serde` crate feature for optional de/serialization support



## [0.1.5](https://github.com/Blobfolio/cdtoc/releases/tag/v0.1.5) - 2023-01-28

### Fix

* Incorrect `Toc::to_string` for discs with exactly 16 tracks.



## [0.1.4](https://github.com/Blobfolio/cdtoc/releases/tag/v0.1.4) - 2023-01-28

### New

* `AccurateRip::pretty_print`

### Changed

* Various performance improvements



## [0.1.3](https://github.com/Blobfolio/cdtoc/releases/tag/v0.1.3) - 2023-01-26

### Changed

* Bump brunch to `0.4`



## [0.1.2](https://github.com/Blobfolio/cdtoc/releases/tag/v0.1.2) - 2023-01-10

### New

* `Toc::duration`

### Changed

* Bump `base64` to `0.21`



## [0.1.1](https://github.com/Blobfolio/cdtoc/releases/tag/v0.1.1) - 2023-01-01

### New

* `Toc::from_durations`
* `Toc::set_audio_leadin`

### Changed

* Enforce minimum audio leadin (`150`)



## [0.1.0](https://github.com/Blobfolio/cdtoc/releases/tag/v0.1.0) - 2022-12-25

Initial release!
