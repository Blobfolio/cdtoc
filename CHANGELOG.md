# Changelog



## [0.11.0](https://github.com/Blobfolio/cdtoc/releases/tag/v0.11.0) - 2025-06-26

### Changed

* Bump `brunch` to `0.11` (dev)
* Bump `dactyl` to `0.13`
* Bump `trimothy` to `0.9`
* Bump MSRV to `1.88`
* Impl `FusedIterator` for `Tracks`
* Miscellaneous code cleanup and lints



## [0.10.0](https://github.com/Blobfolio/cdtoc/releases/tag/v0.10.0) - 2025-06-01

### Changed

* Bump `dactyl` to `0.12`
* Bump `trimothy` to `0.8`
* `Duration::to_f64_lossy` is now const
* `Duration::to_std_duration_lossy` is now const
* `Toc::audio_leadin` is now const
* `Toc::audio_leadin_normalized` is now const
* `Toc::duration` is now const
* `Toc::htoa` is now const
* `Toc::leadin` is now const
* `Toc::leadin_normalized` is now const



## [0.9.1](https://github.com/Blobfolio/cdtoc/releases/tag/v0.9.1) - 2025-05-30

### Changed

* Bump `dactyl` to `0.11`
* Miscellaneous code cleanup and lints



## [0.9.0](https://github.com/Blobfolio/cdtoc/releases/tag/v0.9.0) - 2025-05-15

### Changed

* Bump `brunch` to `0.10` (dev)
* Bump MSRV to `1.87`
* `Toc::audio_len` is now const
* `Toc::audio_sectors` is now const
* `Toc::audio_tracks` is now const
* Miscellaneous code cleanup and lints



## [0.8.1](https://github.com/Blobfolio/cdtoc/releases/tag/v0.8.1) - 2025-04-03

### Changed

* Miscellaneous code cleanup and lints



## [0.8.0](https://github.com/Blobfolio/cdtoc/releases/tag/v0.8.0) - 2025-02-25

### Changed

* Bump `brunch` to `0.9` (dev)
* Bump `dactyl` to `0.10`
* Bump MSRV to `1.85`
* Bump Rust edition to `2024`
* Bump `trimothy` to `0.7`



## [0.7.0](https://github.com/Blobfolio/cdtoc/releases/tag/v0.7.0) - 2025-02-20

### New

* `Toc::ctdb_url`
* `Toc::musicbrainz_url`

### Removed

* `AccurateRip::pretty_print`
* `ShaB64::pretty_print`

### Changed

* Miscellaneous code changes and lints



## [0.6.1](https://github.com/Blobfolio/cdtoc/releases/tag/v0.6.1) - 2025-01-09

### Changed

* Miscellaneous code changes and lints



## [0.6.0](https://github.com/Blobfolio/cdtoc/releases/tag/v0.6.0) - 2024-12-13

### Changed

* Bump `brunch` to `0.8` (dev)
* Bump `dactyl` to `0.9`
* Bump MSRV to `1.83`



## [0.5.3](https://github.com/Blobfolio/cdtoc/releases/tag/v0.5.3) - 2024-11-28

### Changed

* Bump `brunch` to `0.7` (dev)
* Bump `dactyl` to `0.8`
* Bump `trimothy` to `0.6`
* Reduce intermediary allocations for AccurateRip::to_string, ::checksum_url
* Miscellaneous code changes and lints



## [0.5.2](https://github.com/Blobfolio/cdtoc/releases/tag/v0.5.2) - 2024-11-07

### Changed

* Add (more) inline hints
* Add `Formatter` width/fill/align/etc. support for `AccurateRip`, `Cddb`, `ShaB64`
* Improve docs, test coverage



## [0.5.1](https://github.com/Blobfolio/cdtoc/releases/tag/v0.5.1) - 2024-10-10

### Changed

* Bump `faster-hex` to `0.10`



## [0.5.0](https://github.com/Blobfolio/cdtoc/releases/tag/v0.5.0) - 2024-09-05

### Changed

* Miscellaneous code cleanup and lints
* Improved documentation
* Bump MSRV to `1.81`
* Bump `brunch` to `0.6`



## [0.4.0](https://github.com/Blobfolio/cdtoc/releases/tag/v0.4.0) - 2024-07-29

### Changed

* Use new std `<[u8]>::trim_ascii`
* Bump MSRV `1.80`
* Bump `trimothy` to `0.3`



## [0.3.5](https://github.com/Blobfolio/cdtoc/releases/tag/v0.3.5) - 2024-02-08

### Changed

* Bump `dactyl` to `0.7`



## [0.3.4](https://github.com/Blobfolio/cdtoc/releases/tag/v0.3.4) - 2023-11-24

### Changed

* Bump `faster-hex` to `0.9`



## [0.3.3](https://github.com/Blobfolio/cdtoc/releases/tag/v0.3.3) - 2023-11-16

### Changed

* Add explicit lifetime (to fix [#115010](https://github.com/rust-lang/rust/issues/115010))



## [0.3.2](https://github.com/Blobfolio/cdtoc/releases/tag/v0.3.2) - 2023-10-15

### Changed

* Bump `dactyl` to `0.6`



## [0.3.1](https://github.com/Blobfolio/cdtoc/releases/tag/v0.3.1) - 2023-10-05

### Changed

* Minor code lints and cleanup
* Bump `trimothy` to `0.2`



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
