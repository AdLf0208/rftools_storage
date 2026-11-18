# RFTools Storage for Horizon

A clean-room Rust port of **RFTools Storage 1.21-6.0.5** (`McJtyMods/RFToolsStorage`,
MIT) for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft 26.3),
packaged in the SDK layout (`api = "0.2"`): one flat `.hmod` carrying the code in
`src/`, the content in `assets/` and `data/`, built by the launcher against the
game build `horizon-mod.toml` pins.

**Slice 1 — the modular storage block.** `rftoolsstorage:modular_storage` stores
items with the jar's own per-tier capacity: a `storage_module0` in its slot gives
**100 slots, one item kind per slot, up to 64 each** (6,400 of a 64-stack item —
`StorageModuleItem.MAXSIZE[0]` and `GlobalStorageItemWrapper.getSlotLimit()`). A
fresh block stores nothing: as upstream's `insertItem`/`extractItem` do, it needs
a module *and* the lock. Contents and lock survive a relaunch, hoppers move items
both ways, and a broken block spills what it held.

This slice's player surface stands in for the GUI:

| Action | What it does |
|---|---|
| right-click holding a **storage module** | installs it (unlocked, empty slot) |
| right-click holding **any other stack** | inserts as much as fits; the action bar reads `stored/capacity item` |
| right-click with an **empty hand** | seals an unlocked block, or withdraws a stack from a locked one |
| `/rftoolsstorage info <x> <y> <z>` | the readout: `stored/capacity item state` |
| `/rftoolsstorage lock <x> <y> <z>` | toggles the lock (the GUI's padlock; unlock to swap the module) |

## What is not in this slice

The storage scanner, the crafting manager (a WIP in the jar itself), storage
tiers 2-4, the remote module and block, the type/filter/dump/control modules, the
tablet, the crafting grid, the GUI (search/sort/compact/cycle/padlock), the
comparator output, the compat layers (JEI, TOP, XNet, RFToolsUtility), the
`list`/`restore` commands and the config. The fill-level overlay (`amount`,
`type`) is declared and carried but never written — exactly as in 1.21.1, where
`getRenderLevel()` is a commented `// @todo 1.14` stub. Full list and the
reasons: [`notes.md`](notes.md) and [`FINDINGS.md`](FINDINGS.md).

## Building

The package pins Horizon (the SDK nightly)
`0b14f8acfd97fa2c750e42795249dcc981178ca4` and horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0` in [`horizon-mod.toml`](horizon-mod.toml);
the old `.hnmod` pin (`1d63e0a5…` / `5a64af93…`) is retired with the format.

```sh
cd ports/rftoolsstorage
cargo test                                    # the pure rules: 13 runtime tests + 1 glue test
cargo clippy --all-targets -- -D warnings
rustfmt --edition 2024 --check src/lib.rs src/support.rs src/runtime.rs
cargo horizon-mod check                       # the SDK gate: format, manifest, code, API usage
rm -f Cargo.lock                              # never pack a lock (notes.md, trap 3)
cargo horizon-mod build                       # -> target/horizon-mod/rftoolsstorage.hmod
cargo horizon-mod check rftoolsstorage.hmod    # validates the delivered file
cargo horizon-mod fits rftoolsstorage.hmod --build nightly
```

`.horizon/sdk/` is fetched per clone and never packed. Run the cargo steps through
the campaign's build-slot runner (see `../_shared/build-slots-usage.md`).

The in-game legs need a **launcher-built** game carrying the package
(`horizon-launcher native-build ports/rftoolsstorage --instance <i> --trust-native-code`),
then `GAME=<that horizon-client> tools/e2e.sh`. The scenario still has its
counterfactual (`COUNTERFACTUAL=1`, a vanilla barrel) and its selftest
(`SELFTEST=1`, must fail).

Evidence and the exact commands: [`VALIDATION.md`](VALIDATION.md).

## Layout

```
horizon-mod.toml        the only manifest: id, version 0.2.0, api "0.2", side "both",
                        capabilities, [native] game/horizon/mixins
Cargo.toml              one crate; horizon-mod-api (and horizon-core for JavaString) from .horizon/sdk
src/lib.rs              the one #[mod_api::horizon_mod] impl Hooks (register, right_click_block,
                        register_commands, run_mod_command)
src/runtime.rs          the pure rules (the storage engine and its 13 tests), byte-identical
                        to the pre-migration file
src/support.rs          the game glue: the block-entity Part, the container, the click, the readout
assets/rftoolsstorage/  blockstate, models, textures and lang, carried from the jar
data/rftoolsstorage/    the block and item declarations, the creative tab, loot table, recipes, tags
rftoolsstorage.hmod     the deliverable: one file, code and content together
LICENSE                 the upstream MIT notice (Copyright (c) 2019 McJtyMods)
LICENSE-NOTICE.md       provenance, including the RFToolsBase casing textures
```

## Licence

MIT, upstream's own. This port is **local-only** until the maintainer approves
publishing (the campaign's ARCHITECTURE.md §8); it carries the upstream
copyright and permission notice in every artifact.
