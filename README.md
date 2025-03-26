# WIP - NOT USABLE YET #

# bevy_bor3d

[![crates.io](https://img.shields.io/crates/v/bevy_bor3d)](https://crates.io/crates/bevy_bor3d)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![docs.rs](https://docs.rs/bevy_bor3d/badge.svg)](https://docs.rs/bevy_bor3d)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/KirmesBude/bevy_bor3d#license)

| bevy | bevy_titan   |
|------|--------------|
| 0.14 | 0.1.0        |

## What is bevy_bor3d?

`bevy_bor3d` brings billboard support to bevy.
- Sprite in 3d space
- Imposters with directions (per view on the gpu)
- TextureAtlas support
- Text in 3d space
- Follow camera (per view on the gpu)

## Quickstart


```toml, ignore
# In your Cargo.toml
bevy_bor3d = "0.1"
```

### main.rs
```rust, ignore
//! A basic example of how to use bevy_bor3d.
use bevy::prelude::*;
use bevy_bor3d::prelude::BillboardPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BillboardPlugin)
        .add_systems(Startup, (setup, load_texture_atlas).chain())
        .run();
}

fn setup(...) {
    /* Setup camera and other stuff */
}
```

## Documentation

[Full API Documentation](https://docs.rs/bevy_bor3d)

[Examples](https://github.com/KirmesBude/bevy_bor3d/tree/main/examples)

## Future Work

* TBD

## License

bevy_bor3d is free, open source and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](https://github.com/KirmesBude/bevy_bor3d/blob/main/LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/KirmesBude/bevy_bor3d/blob/main/LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!

Some of the code was adapted from other sources.
The [assets](https://github.com/KirmesBude/bevy_bor3d/tree/main/assets) included in this repository fall under different open licenses.
See [CREDITS.md](https://github.com/KirmesBude/bevy_bor3d/blob/main/CREDITS.md) for the details of the origin of the adapted code and licenses of those files.

### Your contributions

Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license,
shall be dual licensed as above,
without any additional terms or conditions.
