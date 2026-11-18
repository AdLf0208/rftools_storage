# RFTools Storage — a Horizon port

**RFTools Storage** is a Minecraft mod. Everything related to storage. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [McJtyMods/RFToolsStorage](https://github.com/McJtyMods/RFToolsStorage) — licence **MIT**.

| | |
|---|---|
| Upstream mod | RFTools Storage ([source](https://github.com/McJtyMods/RFToolsStorage)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `rftoolsstorage.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

### What is not in this slice

The storage scanner, the crafting manager (a WIP in the jar itself), storage
tiers 2-4, the remote module and block, the type/filter/dump/control modules, the
tablet, the crafting grid, the GUI (search/sort/compact/cycle/padlock), the
comparator output, the compat layers (JEI, TOP, XNet, RFToolsUtility), the
`list`/`restore` commands and the config. The fill-level overlay (`amount`,
`type`) is declared and carried but never written — exactly as in 1.21.1, where
`getRenderLevel()` is a commented `// @todo 1.14` stub. Full list and the
reasons: [`notes.md`](notes.md) and [`FINDINGS.md`](FINDINGS.md).

## Not yet ported

- `storage_scanner` (inventory scan, search, FE use, remote/tablet variant),
- Storage tiers 2-4 (`storage_module1..3`; their recipes need McJtyLib's
- The type modules (`generic`/`oredict`), the filter module, the dump module, the
- The GUI (search, sort, compact, cycle, the padlock button, the module tooltip),
- The `rftoolsbase:machine_frame` ingredient and `rftoolsbase`'s casing textures
- **The in-game e2e has not run at this pin.** This machine has no
- **`ports/tools/upstream_ready.py rftoolsstorage` still reports `blocked`**
- **`block_entity_container` has no hook equivalent.** The container is a

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `rftoolsstorage.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

RFTools Storage was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/McJtyMods/RFToolsStorage

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run rftoolsstorage` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/rftoolsstorage/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
