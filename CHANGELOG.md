# Changelog


## [0.1.7](https://github.com/Blobfolio/cdtoc/releases/tag/v0.1.7) - TBD

### Changed

* Add `visit_seq` deserializer support for `Track`



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
