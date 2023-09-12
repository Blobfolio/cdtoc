# CDTOC

[![docs.rs](https://img.shields.io/docsrs/cdtoc.svg?style=flat-square&label=docs.rs)](https://docs.rs/cdtoc/)
[![changelog](https://img.shields.io/crates/v/cdtoc.svg?style=flat-square&label=changelog&color=9b59b6)](https://github.com/Blobfolio/cdtoc/blob/master/CHANGELOG.md)<br>
[![crates.io](https://img.shields.io/crates/v/cdtoc.svg?style=flat-square&label=crates.io)](https://crates.io/crates/cdtoc)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/cdtoc/ci.yaml?label=ci&style=flat-square)](https://github.com/Blobfolio/cdtoc/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/cdtoc/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/cdtoc)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)
[![contributions welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square&label=contributions)](https://github.com/Blobfolio/cdtoc/issues)



CDTOC is a simple Rust library for parsing and working with audio CD tables of contents, namely in the form of [CDTOC-style](https://forum.dbpoweramp.com/showthread.php?16705-FLAC-amp-Ogg-Vorbis-Storage-of-CDTOC&s=3ca0c65ee58fc45489103bb1c39bfac0&p=76686&viewfull=1#post76686) metadata values.

By default it can also generate disc IDs for services like [AccurateRip](http://accuraterip.com/), [CDDB](https://en.wikipedia.org/wiki/CDDB), [CUETools Database](http://cue.tools/wiki/CUETools_Database), and [MusicBrainz](https://musicbrainz.org/), but you can disable the corresponding crate feature(s) — `accuraterip`, `cddb`, `ctdb`, and `musicbrainz` respectively — to shrink the dependency tree if you don't need that functionality.



## Examples

```rust
use cdtoc::Toc;

// From a CDTOC string.
let toc1 = Toc::from_cdtoc("4+96+2D2B+6256+B327+D84A").unwrap();

// From the raw parts.
let toc2 = Toc::from_parts(
    vec![150, 11563, 25174, 45863],
    None,
    55370,
).unwrap();

// Either way gets you to the same place.
assert_eq!(toc1, toc2);

// You can also get a CDTOC-style string back at any time:
assert_eq!(toc1.to_string(), "4+96+2D2B+6256+B327+D84A");
```



## De/Serialization

The optional `serde` crate feature can be enabled to expose de/serialization implementations for this library's types:

| Type | Format | Notes |
| ---- | ------ | ----- |
| `AccurateRip` | `String` | |
| `Cddb` | `String` | |
| `Duration` | `u64` | |
| `ShaB64` | `String` | MusicBrainz and CTDB IDs. |
| `Toc` | `String` | |
| `Track` | `Map` | |
| `TrackPosition` | `String` | |



## Installation

Add `cdtoc` to your `dependencies` in `Cargo.toml`, like:

```toml
[dependencies]
cdtoc = "0.2.*"
```

The disc ID helpers require additional dependencies, so if you aren't using them, be sure to disable the default features (adding back any you _do_ want) to skip the overhead.

```toml
[dependencies.cdtoc]
version = "0.2.*"
default-features = false
```



## License

Copyright © 2023 [Blobfolio, LLC](https://blobfolio.com) &lt;hello@blobfolio.com&gt;

This work is free. You can redistribute it and/or modify it under the terms of the Do What The Fuck You Want To Public License, Version 2.

    DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
    Version 2, December 2004
    
    Copyright (C) 2004 Sam Hocevar <sam@hocevar.net>
    
    Everyone is permitted to copy and distribute verbatim or modified
    copies of this license document, and changing it is allowed as long
    as the name is changed.
    
    DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
    TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
    
    0. You just DO WHAT THE FUCK YOU WANT TO.
