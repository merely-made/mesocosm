# Licensing

Mesocosm deliberately separates the game, reusable libraries, and game
assets. A license applies according to the material's owning boundary, not
merely because its text is present in the repository.

## Game and repository

Game code, application and host crates, tests, examples, and repository
documentation are licensed under the Mozilla Public License 2.0
(`MPL-2.0`). See `LICENSE-MPL-2.0`.

The `0.0.1` name-reservation package was published under `MIT OR
Apache-2.0`; those existing grants remain in force. The MPL game-code grant
begins with repository version `0.0.2`.

## Reusable libraries

A separately identified reusable library crate may be licensed under either:

- the MIT License; see `LICENSE-MIT`
- the Apache License 2.0; see `LICENSE-APACHE`

Such a crate must declare `MIT OR Apache-2.0` in its own `Cargo.toml`.
Keeping those license texts at repository root does not dual-license the
MPL-covered game code. Extraction requires a real reusable boundary and
should not be used merely to evade the game license.

## Assets

Original game assets under `assets/` are licensed under Creative Commons
Attribution-ShareAlike 4.0 International (`CC-BY-SA-4.0`) unless an adjacent
notice says otherwise. See `LICENSE-CC-BY-SA-4.0` and
`assets/ATTRIBUTION.md`.

Imported or third-party assets retain their own licenses. Add their creator,
source, license, and modification history to `assets/ATTRIBUTION.md`; never
silently relicense them as project originals.
