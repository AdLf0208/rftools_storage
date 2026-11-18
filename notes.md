# RFTools Storage → Horizon

Ported from **RFTools Storage 1.21-6.0.5** (`McJtyMods/RFToolsStorage`, **MIT**),
Minecraft Java 1.21.1 NeoForge. The jar is
`ports/_shared/licence-cache/rftoolsstorage-1.21-6.0.5.jar` (301 entries); the
behaviour was read from branch `1.21_neo` at commit
`fc1b18a7f070d0030671e1db1d3aab90699fd160` (`gradle.properties`
`version=1.21-6.0.5`, the jar's own version) — analysis only, this is a
clean-room Rust rewrite. Everything cited, and the brief-versus-artefact
differences, are in [`FINDINGS.md`](FINDINGS.md).

**Licence: MIT → rank 0 (permissive).** Local-only until the maintainer approves
publishing (ARCHITECTURE.md §8); the port carries the upstream MIT notice in
[`LICENSE`](LICENSE) and its provenance in [`LICENSE-NOTICE.md`](LICENSE-NOTICE.md).

**Format: the SDK package (2026-11-18), version 0.2.0.** One flat `.hmod` —
`horizon-mod.toml` + `Cargo.toml` + `src/` + `assets/` + `data/` — built by the
launcher against SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** with
horizon-mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** (see
[`## Migration to the SDK format`](#migration-to-the-sdk-format--version-020)).
The retired `.hnmod` container pinned Horizon `1d63e0a5…` / mixins `5a64af93…`;
`src/runtime.rs` is **byte-identical** to the retired `mixins/rftoolsstorage/runtime.rs`
(SHA-256 `01691fc18bd4…8615f4`, including the copy inside the built package).

Status: **ported-partial** — slice 1 (the modular storage block with a tier-1
storage module) is implemented; its 0.1.0 behaviour was verified in-game, and the
migrated 0.2.0 package passes the SDK gate (`check`, `fits`) but has not been run
in a launcher-built game on this machine (no `horizon-launcher` here — see
[`## Migration to the SDK format`](#migration-to-the-sdk-format--version-020),
gap 1). The gap list below is the rest of the mod.

## Class + tier and the approach

- **Class:** `content`; **tier:** `machines` (`ports/ledger.csv`,
  `ports/_shared/source-map/atm10-sources.csv`).
- **Approach** (`horizon/docs/modpacks.md`, "What packs need, and what runs
  today"): a **machines** shape — a block entity with a container and its
  rules — plus the carried block, items, models and data. Not a data-only block
  (it stores items) and nothing here is `deep` (no entity, renderer or new
  vanilla hook).
- **Carry-over vs agent port:** the blockstate, the 21 block models, the front
  and overlay textures, the module texture, the item/block models, the lang
  lines, the loot table and the two recipes are carried from the jar; the three
  machine casing textures come from RFToolsBase 1.21-6.0.11 (MIT, same author)
  so the block renders without that mod. The storage rules, the block entity,
  the container, the interaction, the readout and the tests are the agent port.
- **Parents:** `rftoolsbase` (a required dependency upstream, and ours:
  `ports/rftoolsbase`, `ported-partial`) and `mcjtylib` (a library, mapped, not
  ported). The port's Rust calls neither: it only needs RFToolsBase for the
  `rftoolsbase:machine_frame` ingredient of one recipe (recorded as a gap).

## What the slice does

- **The block.** `rftoolsstorage:modular_storage`, on the **`minecraft:end_rod`**
  template (the vanilla six-way `facing` the jar's blockstate rotates the main
  model by, with no tick, no redstone and no block entity of its own), full-cube
  shapes, hardness 2.0 / blast resistance 2.0 / correct tool / metal sound
  (McJtyLib's `defaultProperties()`, the same numbers the campaign measured for
  `rftoolsbase:machine_infuser`), and the jar's own three state properties:
  `facing` (6), `type` (3) and `amount` (18).
- **The storage.** A storage module in the block's slot sets the capacity:
  `StorageModuleItem.MAXSIZE = {100, 200, 300, 500, 0, 0, -1}` **slots**, each
  holding one item kind up to `GlobalStorageItemWrapper.getSlotLimit()` = 64
  (and never more than the item's own stack size). Tier 1 (`storage_module0`,
  the only one carried) is **100 slots = 6,400 items of a 64-stack item**. One
  kind per slot, `ItemStack` components included; inserted in slot order, the
  way `ItemHandlerHelper.insertItem` walks them.
- **The lock.** Upstream's `insertItem`/`extractItem` both start with
  `if (!isLocked()) return …`, and the module slot's `mayPlace` is
  `if (isLocked()) return false`: a fresh block stores nothing, and only a
  locked one with a module does. Both states are the block entity's saved
  compound, so they survive a relaunch.
- **The container.** `native_block_entity_container` (API 16) declares all 100
  slots on every face, inputs and outputs, so hoppers move items both ways and a
  broken block spills its contents (`PreRemove::Contents` for a mod container).
- **The player surface** (this slice's substitute for the GUI): a storage module
  in hand installs it; a held stack inserts as much as fits; an empty hand
  **seals** an unlocked block or **withdraws** a stack from a locked one; and
  every action answers on the action bar with `stored/capacity item`. The GUI's
  padlock button is also `/rftoolsstorage lock <x> <y> <z>` (it is how a player
  unlocks to swap the module), and `/rftoolsstorage info <x> <y> <z>` is the
  readout the harness reads.

## Deliberate deviations

1. **The storage is the block entity's own slots**, not upstream's world-level
   `StorageHolder` map keyed by the module item's UUID. The observable behaviour
   of this slice is the same (contents survive a relaunch and outlive a module
   swap) and it needs no world file; the difference is that one module item
   cannot serve two blocks at once, which upstream's UUID model allows.
2. **Right-click replaces the GUI** (search/sort/compact/cycle/visible-size are
   not there), and the padlock is the empty-hand click plus a command. A
   `MenuProvider` block opens its menu from any face upstream; this port has no
   menu, so the block does not open one.
3. **The `type`/`amount` overlays are declared but never written.** Upstream
   1.21.1 declares them and ships 19 overlay models but never sets either
   (`getRenderLevel()` is a commented-out `// @todo 1.14` stub), so the fill
   bar never shows. The port keeps the states and the models (the jar's
   blockstate must bind) and leaves them at their defaults, `none`/`none`.
   Restoring the 1.12 formula would be inventing a mechanic; it is a gap, not a
   defect fixed.
4. **The container declares the tier-1 slot count** (100). The pin declares a
   block's container once per block, not per instance, so tiers 2-4 (200/300/500
   slots) cannot be wired through the container path yet; the runtime carries
   the whole table and its tests.
5. **A break spills the contents** as loose drops instead of carrying them in
   the item: `minecraft:copy_components` covers only vanilla block entities at
   this pin (ARCHITECTURE §3), and upstream's loot list names `mcjtylib:*`
   components this port does not have. Nothing is lost either way.
6. The `facing` comes from the template's rule (the face you click) rather than
   McJtyLib's "face the placer": the port does not inject `state_for_placement`,
   so the front is the clicked face. The block accepts a click on any face.

## Upstream defects fixed

None. The two candidates were checked and left alone, on purpose:

- the **dead fill overlay** (§3 above) is a commented `@todo 1.14` deferral, not
  a broken behaviour, so it is a deviation/gap;
- `ModularStorageContainer.clicked` carries a `// @todo 1.14` for the module
  copy, and `cycle()`/`getRenderLevel()` are stubs — all deferred features of
  the GUI this slice does not carry.

## Gaps (the rest of the mod)

- `storage_scanner` (inventory scan, search, FE use, remote/tablet variant),
  `crafting_manager` (the jar's own lang calls it "WIP! … not functional yet").
- Storage tiers 2-4 (`storage_module1..3`; their recipes need McJtyLib's
  `mcjtylib:copy_components` recipe type, which the pin does not have), the
  remote module (`storage_module6`, "DO NOT USE" in the jar's lang) and the
  remote storage block.
- The type modules (`generic`/`oredict`), the filter module, the dump module, the
  storage control module, the tablet, and the crafting grid.
- The GUI (search, sort, compact, cycle, the padlock button, the module tooltip),
  the comparator output, JEI/TOP/XNet/RFToolsUtility compat, the
  `/rftoolsstorage list|restore` commands and the config.
- The `rftoolsbase:machine_frame` ingredient and `rftoolsbase`'s casing textures
  in the model chain (the port copied the textures into its own namespace, so
  only the recipe needs RFToolsBase installed).

## Hook table

The SDK form (this package) — one `impl Hooks` and one `BlockEntityPart`; the
retired helper files and their mapping are in
[`## Migration to the SDK format`](#migration-to-the-sdk-format--version-020).

| Hook (SDK 0.2) | What it does |
|---|---|
| `register` → `Registrar` | declares the block-entity type `rftoolsstorage:storage_block_entity` over the block, attaches its `BlockEntityPart`, and declares the block's container (100 slots, every face, inputs and outputs, no menu) |
| `BlockEntityPart::block_entity` | gives `rftoolsstorage:modular_storage` a block entity with empty saved data (the module tier and the lock live in its compound; `Items` are the container's) |
| `right_click_block` (cancel) | install / insert / seal / withdraw, and the action-bar readout |
| `register_commands` | `/rftoolsstorage info <pos>` and `/rftoolsstorage lock <pos>` (typed `block_pos`) |
| `run_mod_command` | their answers |
| ~~`server_tick`~~ | **retired**: it existed only to resend a LAN guest's inventory after the old `native_give_stack`, which bypassed the inventory sync. `PlayerRef::give_stack` is `/give` with the vanilla sync, so the workaround (and its `Mutex` state) is gone, not a gap |

## Upstream source files read

`ModularStorageBlock.java`, `ModularStorageTileEntity.java`,
`ModularStorageContainer.java`, `ModularAmountOverlay.java`,
`ModularTypeModule.java`, `StorageModuleItem.java`, `StorageModuleData.java`,
`ModularStorageData.java`, `ModularStorageModule.java`,
`storage/GlobalStorageItemWrapper.java`, `storage/StorageEntry.java`,
`storage/StorageInfo.java`, `setup/Registration.java`, `setup/ModSetup.java`.
The clone (`ports/_shared/upstream/rftoolsstorage`) was deleted after these
findings were recorded.

## Migration to the SDK format — version 0.2.0

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70) **retired `.hnmod`**; the
registry closed the 0.1.0 submission (#909) for it on 2026-11-18. This port is
now **one flat `.hmod`**: `horizon-mod.toml` + `Cargo.toml` + `src/` + `assets/`
+ `data/`, built by the launcher against the build `[native]` pins. The recipe
followed is `ports/imfast/notes.md` → `## Migration to the SDK format`; the
closest shape reference (block entity + container + commands) is
`ports/refinedstorage/notes.md` → its migration section.

- **SDK:** nightly `0b14f8acfd97fa2c750e42795249dcc981178ca4` (2026-11-18),
  mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, `cargo-horizon-mod` from
  `horizon-rust/horizon@main`, launcher 0.2.78. `.horizon/sdk/` is fetched per
  clone (byte-identical to `ports/_shared/sdk/0b14f8a/`) and never packed.
- **Class + tier unchanged:** `content` / `machines` (`ports/ledger.csv`,
  `horizon/docs/modpacks.md`). This migration changes the package format, not the
  slice: the block, the rules, the carried content and the gaps are the same.
- **`src/runtime.rs` is the retired `mixins/rftoolsstorage/runtime.rs`, byte for
  byte** (SHA-256 `01691fc18bd4…8615f4`, 13 tests), including the copy inside the
  built package — the campaign's rule made literal: only the glue, manifest and
  packaging were rewritten.

### File map

| retired `.hnmod` layout (0.1.0) | flat SDK package (0.2.0) |
|---|---|
| `mixins/rftoolsstorage/runtime.rs` | `src/runtime.rs` (byte-identical, `pub mod runtime;` so its items are not `dead_code`) |
| `mixins/rftoolsstorage/support.rs` (injected module) | `src/support.rs` (ordinary module) |
| `mixins/rftoolsstorage/{block_entity,block_entity_container,right_click_block,server_tick,register_commands,perform_command}.rs` (one helper per injection) | `register` + the `BlockEntityPart` + `Hooks` methods in `src/lib.rs`/`src/support.rs` (table below) |
| `mixins/rftoolsstorage/embedded_content.rs` + `data/rftoolsstorage-content.hmod` (the AE2 write-out trick) | **gone**: one package holds code and content together |
| `mixins/rftoolsstorage/horizon-mixin.toml` (7 injections, 2 modules) | no plan: `horizon-mod.toml` + the one `impl Hooks` |
| `mixins/rftoolsstorage/data/`, `content/horizon-mod.toml`, `content/rftoolsstorage.hmod` | root `horizon-mod.toml` + `assets/` + `data/` + one `rftoolsstorage.hmod` |
| `rftoolsstorage.hnmod` (29,549 B) | `rftoolsstorage.hmod` (45,351 B) — the only deliverable |
| `compatibility.json`, `rust-toolchain.toml`, `tools/{horizon.py,package.py,package.sh}`, old `Cargo.toml`/`Cargo.lock` | not part of the format; removed. Every retired file is preserved in `ports/_shared/evidence/rftoolsstorage/retired-hnmod-0.1.0/` |

**Kept:** `tools/e2e.sh` (the scenario and its assertions are format-independent;
the launch line is adapted, see gap 1), `notes.md`, `FINDINGS.md`, `VALIDATION.md`,
`LICENSE`, `LICENSE-NOTICE.md`.

### Helper → hook table (the whole glue, mapped)

| Old helper (old pin) | New form (SDK 0.2) | Verdict |
|---|---|---|
| `block_entity.rs` — `block_entity` (15) | `BlockEntityPart::block_entity(&self, LevelSide, &mut BlockEntityEvent) -> Option<Compound>` on the one Part | **kept** (returns the empty saved data) |
| `block_entity_container.rs` — `block_entity_container` (16) | **no hook at this pin**: `Registrar::block_container(&BlockKey, ModContainer)` in `register` (100 slots, six faces, inputs, outputs, no menu) | **kept, moved to registration** |
| `right_click_block.rs` — `right_click_block` (21), bindings `game`/`pos`/`face`/`canceled` | `Hooks::right_click_block(HostSide<'_>, &mut RightClickBlockEvent) -> Flow`; the block, position, hand and stack are `event.click` (`pos`, `hand`, `stack`, `player`), and the click is consumed with `*event.cancellation_result = InteractionResult::Success` + `Flow::Cancel` on the client's copy and on the server's (only the server writes) | **kept** |
| `server_tick.rs` — `server_tick` (8/21) + `support: Mutex<State>` | **retired by the API**: `PlayerRef::give_stack` is `/give` (it syncs the inventory itself), so no deferred guest sync is needed | **retired, not faked** |
| `register_commands.rs` — `register_commands` (8) | `Hooks::register_commands(&mut dyn ServerHost, &mut RegisterCommandsEvent)` pushes `Command`s | **kept** (typed `<pos:block_pos>`) |
| `perform_command.rs` — `perform_command` (8), words split by hand | `Hooks::run_mod_command(&mut dyn ServerHost, &mut RunModCommandEvent) -> Flow`; `event.command.argument("pos")` is `CommandArgumentValue::BlockPos` | **kept** (typed argument, no parsing) |
| `embedded_content.rs` + `content_root` HEAD injection | — | **retired by the format** (one package holds both halves) |
| `support.rs`: `read_storage`/`write_storage`/`mutate`/`read_at`/`kind_of`/`take_from_hand`/readout | `src/support.rs`, same functions over `Host`/`ServerHost`/`BlockEntityRef` instead of `Game` | **kept** |
| `support.rs`: `game.native_with_mod_block_entity`, `game.world.block_entities` | `host.block_entity(level, pos)` → `BlockEntityRef::{data, item_slot, merge_data, set_item_slot, set_changed}`, the writes inside `host.transaction` | **kept** |
| `support.rs`: `game.native_action_bar`/`native_feedback`/`native_give_stack`/`held`/`items().get` | `PlayerRef::{action_bar, give_stack, selected_slot}`, `Commands::feedback`, `event.click.stack`, `ItemsHost::max_stack_size` | **kept** |
| `support.rs`: `kind_of` (`"{item}\|{damage}\|{extra:?}"`) | same identity from `ItemKey::id(host)`, `ItemStack::damage()`, `ItemStack::components().as_nbt()` | **kept** |

`uses.json` in the package is the honest summary: **244 API items** — 6 hooks
(`block_entity`, `register_commands`, `right_click_block`, `run_mod_command`, plus
the groups the `#[horizon_part]` macro derives: `blocks`, `command`), 5 Part
kinds, 1 capability, 21 traits, 83 methods (`BlockEntityRef::{data, item_slot,
merge_data, set_item_slot, set_changed}`, `PlayerRef::{action_bar, give_stack,
selected_slot}`, `ItemsHost::max_stack_size`, `Registrar::{block_key,
block_entity_type, block_entity_part, block_container}` …).

### Manifests

- `horizon-mod.toml` — `id = "rftoolsstorage"` (the namespace), `version =
  "0.2.0"` (a **new** version: the form refuses a duplicate while an older
  version's PR is open), `api = "0.2"` (exact), `side = "both"` (an integrated
  server belongs to the client installation, so single player and a LAN host need
  it), `capabilities = ["registries"]` (the one shared value; the block, the
  module item, the tab and the recipes are content), `[native]` `game = "26.3"`,
  `horizon = "0b14f8a…"`, `mixins = "e59da909…"`, plus `license`, `homepage`,
  `issues`.
- `Cargo.toml` — one crate; `horizon-mod-api` **and** `horizon-core` from
  `.horizon/sdk/` (the API re-exports `Compound`/`Tag` but not `JavaString`, and a
  saved compound's key is a `JavaString`). Cargo's name/version are not the mod's
  identity.

### Build, check, package

`build-slot.sh` is still used as the **lease** (the campaign's hard rule), but its
step names speak the retired tool, so the SDK tool runs inside the lease rather
than through a staged ladder — no staging, no staged tree, no `--target-dir`:

```sh
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-rfs --port ports/rftoolsstorage \
    --step check -- bash -c 'cd ports/rftoolsstorage && cargo horizon-mod check'
cd ports/rftoolsstorage
cargo test && cargo clippy --all-targets -- -D warnings
rustfmt --edition 2024 --check src/lib.rs src/support.rs src/runtime.rs
rm -f Cargo.lock && cargo horizon-mod build        # -> target/horizon-mod/rftoolsstorage.hmod
cp target/horizon-mod/rftoolsstorage.hmod ./rftoolsstorage.hmod
cargo horizon-mod check rftoolsstorage.hmod        # the registry's own check of the delivered file
cargo horizon-mod fits rftoolsstorage.hmod --build nightly
```

Measured: `14 passed` (13 runtime + 1 glue), clippy 0 warnings, `rustfmt` clean,
`cargo horizon-mod check` **EXIT 0** (*"RFTools Storage uses 244 items of the API
… Made for 0b14f8a."*), package **45,351 B**, SHA-256
`d3f57e1e2b7b728e6dedaacc2729704f107f54f106f1fcfbcb3b0ea62d65e1de`, **57
entries** (48 content files + `horizon-mod.toml`, `Cargo.toml`, `src/lib.rs`,
`src/runtime.rs`, `src/support.rs`, `uses.json`, `LICENSE`, `LICENSE-NOTICE.md`,
`README.md`), no `Cargo.lock` packed; a second build is byte-identical;
`check rftoolsstorage.hmod` **EXIT 0**; `fits --build nightly` → *"0b14f8a made
for"*, **EXIT 0**. Evidence: `ports/_shared/evidence/rftoolsstorage/sdk-*.log`.

### Gaps (this migration)

1. **The in-game e2e has not run at this pin.** This machine has no
   `horizon-launcher` (`which horizon-launcher` is empty), so nothing can build a
   game carrying the package and the scenario cannot execute. `tools/e2e.sh` is
   adapted (shared `horizon-ctl`, no `--dev-mod`, `GAME` from a launcher build)
   and is the first thing the next machine with a launcher does. The behaviour
   claims above stay the ones the 0.1.0 run measured, and `src/runtime.rs` — the
   rules behind them — is byte-identical.
2. **`ports/tools/upstream_ready.py rftoolsstorage` still reports `blocked`**
   because it checks the retired layout (`content/*.hmod`, `*.hnmod`,
   `compatibility.json`). It is a campaign tool, outside this port's write
   ownership; the SDK gate it should ask is `cargo horizon-mod check` plus
   `fits` (both green here).
3. **`block_entity_container` has no hook equivalent.** The container is a
   registration-time declaration, and the explicit "declare 100 slots per block"
   limit is unchanged (deviation 4): a per-instance container (the 200/300/500
   tiers) still has no path at this pin.
4. **A no-module block entity carries `module = -1`** in its saved compound,
   because `merge_data` sets a key but cannot remove one. It is internal: every
   value outside `0..=3` reads as "no module", no command or readout shows it, and
   the lock byte round-trips as before.
5. **The block's own use stays skipped, on both sides.** `Flow::Cancel` alone
   answers the click with the default `cancellation_result = Pass`, and `Pass`
   goes on to the held item's own use — a block in hand would have been placed
   behind the insert. The port sets `InteractionResult::Success` before cancelling
   (the click is the storage's, as the old `*canceled = true` made it), and it
   does so on the client's copy of the click too, while only the server writes.
   That is the SDK's shape of the old hook, not a deviation; it is recorded
   because getting it wrong is silent until an item with a use is in hand.

### Traps the next port must know

1. **A leftover old path refuses the package.** Delete/move `mixins/`, the
   `.hnmod`, `compatibility.json`, `rust-toolchain.toml` and the old
   `tools/{horizon,package}.py` *before* the first build, and copy `runtime.rs`
   to `src/` first (then `cmp` it).
2. **`cargo horizon-mod check` (no file) is the SDK gate here** — it compiled the
   crate, printed the 244-item usage summary and exited 0. The imfast trap about
   wasm was a tool bug, gone in this `cargo-horizon-mod`.
3. **A packed `Cargo.lock` changes the artifact and the hash.** `cargo
   test`/`clippy` write one; `rm -f Cargo.lock` immediately before every
   `horizon-mod build`, and keep `/Cargo.lock` in `.gitignore`. (Measured
   elsewhere: 7,570 B → 23,133 B for imfast.)
4. **`cargo fmt --all` reformats `.horizon/sdk`.** Use `rustfmt --edition 2024
   --check src/lib.rs src/support.rs src/runtime.rs`; never "fix" the fetched SDK.
5. **`runtime.rs` as a module.** `pub mod runtime;` — a private `mod runtime;`
   turns every `pub` item into `dead_code`, and adding an attribute to the file
   would break the byte-identical rule.
6. **`BlockEntityRef` writes are not the tick's view.** `data()` is the saved
   compound *without* `Items`; the slots are read one by one with `item_slot()`
   and written with `set_item_slot(host, tx, …)` inside `host.transaction`. The
   raw item write is silent — call `set_changed()` after it (merge_data already
   marks changed when the module/lock moved).
7. **`Flow::Cancel` is not "handled" by default.** The cancellation answers with
   `cancellation_result`, whose default is `Pass` — the block's use is skipped
   but the *item's* use still runs, so a block in hand is placed behind the
   interaction. Set `InteractionResult::Success` (or `Fail`) as well; do it on
   the client's copy of the click too, so the client does not predict the item
   use the server refuses (`extradisks`, `enderstorage` and `tiab` do the same).
8. **`right_click_block` fires on both sides and once per hand.** Guard the world
   writes with `HostSide::Server` and the gesture with `event.click.hand ==
   Hand::Main`, and read the block through the side's own host
   (`is_storage(&**host, pos)`) — a `match` that moves the `&mut dyn` out of
   `HostSide` cannot be reused, so take `&HostSide` and deref.
9. **`Host` methods are flat, not area accessors.** It is
   `host.max_stack_size(item)`, not `host.items().max_stack_size(item)`;
   `JavaString` needs the extra `horizon-core` dependency (`horizon-mod-api`
   re-exports `Compound`/`Tag`, not the key type).
10. **The block check is a `BlockKey` comparison.** A `BlockKey` cannot be built
   from a string without a host, so keep it in a `OnceLock` set in `register` and
   compare handles; `register` takes `&self`, so it cannot live in a `Mod` field.
11. **One Part per block-entity type.** The Part is attached to the declared type,
   not the block, and `BlockEntityPart` needs the empty `BlockPart` impl beside it.
12. **`ModContainer`/`Screen` literals should end with `..Default::default()`**:
    `check` prints a compatibility note when they are built field by field.
13. **`.horizon/sdk` is 21 MB and gitignored.** Copy it from a migrated port
    (`ports/refinedstorage/.horizon/sdk`, byte-identical by hash) instead of
    waiting for a fetch; never pack it.
14. **The delivered `.hmod` lives in the port root**, not `target/`. The build is
    deterministic (fixed zip mtimes), so the SHA-256 is stable across rebuilds.

### Changelog for the resubmission (supersedes #909)

> Supersedes #909, closed on 2026-11-18 because Horizon retired `.hnmod`
> (#17992). Changes: one flat SDK package (`api = "0.2"`, `[native]` pinned to
> `0b14f8a` / `e59da90`, version **0.2.0**); the retired plan/helpers are rewritten
> as `impl mod_api::Hooks` methods against typed hosts and events — 9 old glue
> pieces mapped: 8 kept (`block_entity` as a `BlockEntityPart`, the container moved
> to a registration-time `block_container`, `right_click_block`,
> `register_commands`, `perform_command` → typed `run_mod_command`, and the
> support helpers), 1 retired by the API (`server_tick`'s guest-inventory resync,
> now `/give` semantics), 0 faked; content and code are one package (the AE2
> embedded-content trick is gone); `src/runtime.rs` and its 13 tests unchanged,
> plus 1 glue test for the readout; `cargo horizon-mod check`/`build`/`fits` all
> exit 0, package 45,351 B, SHA-256 `d3f57e1e…`; the in-game e2e is adapted but
> **not yet run** on a launcher-built game (no `horizon-launcher` on the build
> machine).

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

## Verification

Full evidence in [`VALIDATION.md`](VALIDATION.md); the e2e logs are in
`ports/_shared/evidence/rftoolsstorage/`.

**0.2.0 — the SDK package (2026-11-18)**

| Step | Command | Result |
|---|---|---|
| Runtime + glue tests | `cargo test` | **14 passed**, 0 failed (`evidence/rftoolsstorage/sdk-test.log`, `TEST_EXIT:0`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | clean (`sdk-fmt.log`) |
| SDK gate | `cargo horizon-mod check` | EXIT 0 — *"RFTools Storage uses 244 items of the API (… 6 hooks …). Made for 0b14f8a."* (`sdk-check.log`) |
| Package | `cargo horizon-mod build` | `target/horizon-mod/rftoolsstorage.hmod`, EXIT 0; a second build byte-identical (`sdk-build.log`) |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.0 (native-mod, format native-1/0b14f8a/e59da90, for 26.3)`, **45,351 B**, SHA-256 `d3f57e1e…e1de`, EXIT 0 (`sdk-file-check.log`) |
| What it uses | `cargo horizon-mod check rftoolsstorage.hmod --uses` | 244 entries (`sdk-uses.json`), EXIT 0 |
| Fits the pinned build | `cargo horizon-mod fits rftoolsstorage.hmod --build nightly` | *"0b14f8a made for"*, EXIT 0 (`sdk-fits.log`) |
| `runtime.rs` untouched | `unzip -p rftoolsstorage.hmod src/runtime.rs \| sha256sum` | `01691fc1…8615f4` = the retired file, byte for byte (`sdk-sha256.txt`) |
| Symbols line | `python3 ports/tools/check-skills.py --notes ports/rftoolsstorage` | `Skills line present`, **SKILLS GATE: PASS** (`--strict` exit 0) |

**0.1.0 — the retired `.hnmod` (behaviour evidence, still valid)**

The logs `ports/_shared/evidence/rftoolsstorage/e2e*.log` and `ladder-*.log` are
the 0.1.0 run's: the staged ladder and the embedded/control legs that produced
them are retired with the format (the script's `EMBEDDED`/`CONTROL` legs are
gone). The 0.2.0 `tools/e2e.sh` keeps the same scenario, counterfactual and
selftest, and waits for a launcher-built game.

| Step | Result |
|---|---|
| `cargo test --offline` (runtime) | **13 passed**, 0 failed |
| `cargo clippy --all-targets --offline -- -D warnings` | clean, **0 warnings** |
| `cargo fmt --check` | pass |
| `horizon.py stage` | `staging_seconds` ~100 s, sha-verified pin |
| `horizon.py check` | `cargo_exit 0`, **0 warnings** in `target/horizon-check.log` |
| `horizon.py test` (embedded) | **17 passed**, 0 failed (`1240 filtered out`) |
| `horizon.py build` (fast) | `cargo_exit 0` |
| content `.hmod` | `content/rftoolsstorage.hmod` **20,762 B**, SHA-256 `51cfef738e560ca955f8e6f7c38ecee7b7dc53c85957d6042638a8ef7292f274`, 49 entries, `no code: content only` |
| the deliverable `.hnmod` | `rftoolsstorage.hnmod` **29,549 B**, SHA-256 `8c43840a410d5f2d0f75f734ccd7ddbc0826f7082f53f1e1b435301a411e8d82`, **14 entries**, carrying `mixins/rftoolsstorage/data/rftoolsstorage-content.hmod` (the 20,762 B `.hmod`) plus `LICENSE` and `LICENSE-NOTICE.md` |
| e2e default | **PASS**, `EXIT:0` (`e2e.log`) |
| e2e embedded (no `--dev-mod`) | **PASS**, `EXIT:0`, the written content byte-identical (`e2e-embedded.log`) |
| e2e control (unusable `.horizon-cache`) | **PASS**, `EXIT:0`: no storage item, the block id unknown (`e2e-control.log`) |
| e2e counterfactual (vanilla barrel) | **PASS**, `EXIT:0`: the same click stores nothing (`e2e-counterfactual.log`) |
| `SELFTEST=1` | **FAIL** + `EXIT:1`: the capacity assertion is alive (`e2e-selftest.log`) |
| `check-e2e-logs.py rftoolsstorage --strict` | exit 0, 5/5 logs |
