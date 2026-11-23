# RFTools Storage for Horizon

A clean-room Rust port of **RFTools Storage 1.21-6.0.5** (`McJtyMods/RFToolsStorage`,
MIT) for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft 26.3),
packaged in the SDK layout (`api = "0.2"`): one flat `.hmod` carrying the code in
`src/`, the content in `assets/` and `data/`, built by the launcher against the
game build `horizon-mod.toml` pins.

**The modular storage block.** `rftoolsstorage:modular_storage` stores items with
the jar's own per-tier capacity: a `storage_module0` in its card slot gives **100
slots, one item kind per slot, up to 64 each** (6,400 of a 64-stack item —
`StorageModuleItem.MAXSIZE[0]` and `GlobalStorageItemWrapper.getSlotLimit()`). A
fresh block stores nothing: as upstream's `insertItem`/`extractItem` do, it needs
a module *and* the lock. A right click opens its **menu** — a 10 x 10 grid of the
storage slots, the player's inventory beside it, the card slot that holds the
module and a padlock; the grid takes a stack only while locked and only in a slot
of its own kind. While sneaking, a click is the quick path instead: a module in
hand installs, a held stack inserts, an empty hand seals or gives a stack back.
A broken block's **contents travel on its own item** (`minecraft:block_entity_data`)
with the lock and the module, and come back when it is placed.

**The storage scanner.** `rftoolsstorage:storage_scanner` is the mod's second
machine: a 50,000 FE buffer and `/rftoolsstorage scan <x> <y> <z>`, which reads
the item capability of every inventory in the `pos ± radius` box and answers one
entry a kind (`rftoolsstorage:storage_scanner: minecraft:stone 64 (4800/50000 FE)`).
A scan costs 100 FE (`rfPerRequest`); below that it answers `no power`. The radius
is the jar's GUI slider (1..20, default 1) as `/rftoolsstorage radius <x> <y> <z> <n>`.
Its **tick** pushes the stack in its in slot (and the one automation puts in the
auto slot) into the inventories that box finds, a stack a tick for 20 FE
(`rfPerInsert`) — also when none of them takes it, as the jar charges.
`/rftoolsstorage request <x> <y> <z> <item> [n]` pulls a kind out of those
inventories into the player for `rfPerRequest` (a tenth of it for an explicit
count), and `/rftoolsstorage give <x> <y> <z> <item> [single]` hands one item or
a whole stack straight to the player for the **whole** price — the jar's
give-from-screen path, which never uses the out slot. `/rftoolsstorage insert
<x> <y> <z> <item> [n]` is the screen's insert button: it pushes a stack out of
the player's inventory into those inventories and charges one `rfPerInsert`, but
**only when the stack went in whole** — a partial insert is free.

## Commands

| Command | What it does |
|---|---|
| `/rftoolsstorage info <x> <y> <z>` | a storage's readout: `stored/capacity item state` |
| `/rftoolsstorage lock <x> <y> <z>` | toggles a storage's lock (the GUI's padlock) |
| `/rftoolsstorage scan <x> <y> <z>` | what the inventories in the scanner's box hold, and the FE left |
| `/rftoolsstorage radius <x> <y> <z> <n>` | sets the scanner's radius (1..20) |
| `/rftoolsstorage request <x> <y> <z> <item> [n]` | pulls a kind out of the scanned inventories into the player (a tenth price for an explicit count) |
| `/rftoolsstorage give <x> <y> <z> <item> [single]` | hands a whole stack (or one item with `single`) straight to the player for the full price; no out slot |
| `/rftoolsstorage insert <x> <y> <z> <item> [n]` | pushes the player's stack (or `n` of it) into the scanned inventories; one `rfPerInsert`, charged only when the stack emptied |

They need an operator or a player within 8 blocks of the block.

## What is not ported

The crafting manager (a WIP in the jar itself), the storage scanner's GUI **list**
(search, sort, compact, cycle, visible size) and the out slot's fill for the
no-player fallback, the `routable` star list, the storage control screen module (the give path's own
caller; its behaviour is carried as `/rftoolsstorage give`),
the scanner's remote/tablet variant, the claimed-land check
(the pin has no claim API), storage tiers 2-4, the remote module and block, the
type/filter/dump/control modules, the tablet, the crafting grid, the comparator
output, the compat layers (JEI, TOP, XNet, RFToolsUtility), the `list`/`restore`
commands and the config. The fill-level overlay (`amount`, `type`) is declared and
carried but never written — exactly as in 1.21.1, where `getRenderLevel()` is a
commented `// @todo 1.14` stub. The `storage_scanner` recipe needs
`rftoolsbase:machine_frame` and RFToolsBase is not in this build. Full list and the
reasons: [`notes.md`](notes.md) and [`FINDINGS.md`](FINDINGS.md).

## Building

The package pins Horizon (the SDK nightly)
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a` and horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0` in [`horizon-mod.toml`](horizon-mod.toml);
the old `.hnmod` pin (`1d63e0a5…` / `5a64af93…`) is retired with the format.

```sh
cd ports/rftoolsstorage
cargo test                                    # the pure rules: 35 runtime tests + 4 glue tests (39)
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
then `GAME=<that horizon-client> tools/e2e.sh`. The scenario has its
counterfactual (`COUNTERFACTUAL=1`, a vanilla barrel) and its selftest
(`SELFTEST=1`, `REQUEST_SELFTEST=1`, `GIVE_SELFTEST=1` and `INSERT_SELFTEST=1`,
must fail).

Evidence and the exact commands: [`VALIDATION.md`](VALIDATION.md).

## Layout

```
horizon-mod.toml        the only manifest: id, version 0.2.9, api "0.2", side "both",
                        capabilities, [native] game/horizon/mixins
Cargo.toml              one crate; horizon-mod-api from .horizon/sdk
src/lib.rs              the one #[mod_api::horizon_mod] impl Hooks (register, right_click_block,
                        block_drops, register_commands, run_mod_command, player_container_open)
src/runtime.rs          the pure rules (the storage's, the menu's, the carried payload's and
                        the scanner's, with their tests)
src/support.rs          the game glue: the block-entity Parts, the container, the menu, the
                        click, the carried payload and the commands (readout, scan, radius,
                        request, give, insert)
assets/rftoolsstorage/  blockstates, models, textures and lang, carried from the jar
data/rftoolsstorage/    the block and item declarations, the creative tab, loot tables, recipes, tags
data/minecraft/         the pickaxe/iron-tool tags the blocks join
rftoolsstorage.hmod     the deliverable: one file, code and content together
LICENSE                 the upstream MIT notice (Copyright (c) 2019 McJtyMods)
LICENSE-NOTICE.md       provenance, including the RFToolsBase casing textures
```

## Licence

MIT, upstream's own. This port is **local-only** until the maintainer approves
publishing (the campaign's ARCHITECTURE.md §8); it carries the upstream
copyright and permission notice in every artifact.
