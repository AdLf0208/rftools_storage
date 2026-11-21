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

**Format: the SDK package (2026-11-18), version 0.2.4.** One flat `.hmod` —
`horizon-mod.toml` + `Cargo.toml` + `src/` + `assets/` + `data/` — built by the
launcher against SDK nightly **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** with
horizon-mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** (see
[`## Migration to the SDK format`](#migration-to-the-sdk-format--version-020) and
[`## Slice 2`](#slice-2--the-storages-menu-version-022),
[`## Slice 3`](#slice-3--the-module-s-card-slot-version-023),
[`## Slice 4`](#slice-4--the-contents-travel-on-the-item-version-024)).
The retired `.hnmod` container pinned Horizon `1d63e0a5…` / mixins `5a64af93…`;
`src/runtime.rs` was byte-identical to the retired
`mixins/rftoolsstorage/runtime.rs` until the 2026-11-18 review fixes gave it the
reach rule, and slice 2 added the menu's two rules.

Status: **ported-partial** — slice 1 (the modular storage block with a tier-1
storage module), slice 2 (its **menu**: a 100-slot grid opened by a right click,
its slot rules and the padlock), slice 3 (the menu's **card slot**, where the
module itself lives, so it can be taken back out) and slice 4 (**a broken block's
contents travel on its own item**, with the lock and the module, and come back on
placement) are implemented and verified in a launcher-built game at the pin (the
legs are listed per slice below; `ports/_shared/evidence/rftoolsstorage-e2e/`).
The gap list below is the rest of the mod.

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
- **The player surface.** Slice 1's substitute is gone: a plain right click now
  opens the storage's **menu** (slice 2 below). A **sneak** click keeps the quick
  path — a storage module in hand installs it, a held stack inserts as much as
  fits, an empty hand **seals** an unlocked block or **withdraws** a stack from a
  locked one — and every gesture answers on the action bar with
  `stored/capacity item`. `/rftoolsstorage lock <x> <y> <z>` is the same padlock
  from chat (how a player unlocks to swap the module), and
  `/rftoolsstorage info <x> <y> <z>` is the readout the harness reads.

## Deliberate deviations

1. **The storage is the block entity's own slots**, not upstream's world-level
   `StorageHolder` map keyed by the module item's UUID. The observable behaviour
   of this slice is the same (contents survive a relaunch and outlive a module
   swap) and it needs no world file; the difference is that one module item
   cannot serve two blocks at once, which upstream's UUID model allows.
2. **The menu is a slot grid, not upstream's scrollable item list.** Slice 2
   gives the block its own menu — the 100 storage slots in a 10 x 10 grid, the
   player's inventory beside it and the padlock on the panel — but the list
   widget's search, sort, compact, cycle and visible-size controls are not there:
   the pin's screen is a fixed slot layout, and those controls are the next
   slice's rules. A plain right click opens it; the quick gestures (module in
   hand installs, a held stack inserts, an empty hand seals or gives a stack
   back) are still there but **behind sneak**, as `functionalstorage`'s
   sneak-click is. The padlock button is the GUI's own lock.
3. **The `type`/`amount` overlays are declared but never written.** Upstream
   1.21.1 declares them and ships 19 overlay models but never sets either
   (`getRenderLevel()` is a commented-out `// @todo 1.14` stub), so the fill
   bar never shows. The port keeps the states and the models (the jar's
   blockstate must bind) and leaves them at their defaults, `none`/`none`.
   Restoring the 1.12 formula would be inventing a mechanic; it is a gap, not a
   defect fixed.
4. **The container declares the tier-1 slot count** (100). The pin declares a
   block's container once per block, not per instance, so tiers 2-4 (200/300/500
   slots) cannot be wired through the container path — and a menu type holds at
   most 256 slots, so tiers 2 and 3 would not fit beside the player's 36 either.
   The runtime carries the whole table and its tests.
5. ~~A break spills the contents as loose drops instead of carrying them in the
   item.~~ **Fixed in slice 4** (0.2.4): a player's break that drops the block moves
   the container's stacks aside before the removal and puts them on the block's own
   item as `minecraft:block_entity_data`; a level-side break (piston, water,
   explosion, `/setblock … destroy`) still spills them, as vanilla does when its block
   entity gets no say. See [`## Slice 4`](#slice-4--the-contents-travel-on-the-item-version-024).
6. The `facing` comes from the template's rule (the face you click) rather than
   McJtyLib's "face the placer": the port does not inject `state_for_placement`,
   so the front is the clicked face. The block accepts a click on any face.
7. **The card slot's rule cannot be per-slot** (slice 3): the engine consults the
   card slot's `menu_slot_rules` answer while it plans a click on **another** slot,
   and a refusal there blocks that click (measured: with the card's rule strict, a
   click on a locked storage's grid was refused). So the card slot answers "a
   storage module, or anything once it holds one" — `runtime::card_accepts` — and
   only an *empty* card slot says "module only". Upstream says "module only" and
   "nothing while locked"; the two cases the port differs in are ones the engine's
   own slot caps already cover (a second module cannot stack) or that cannot arise
   (a locked storage always holds its module, because taking it out is what leaves
   the storage unlocked).
8. **A non-module in the card slot sets no capacity**, and a module is what makes
   the storage work: put a stone there and the readout is `0/0 empty no module`;
   the stone is an ordinary item slot's, returned by a click. Upstream refuses the
   stone at `mayPlace` time; see deviation 7 for why the port cannot.
9. **The block's lock byte outlives the module.** A storage whose card slot is
   emptied reads "no module" and takes a module (the pin has no `may_pickup` rule,
   so the module's own removal cannot be refused while locked); the byte stays
   where it is, so the lock returns with the module. A module-less storage is never
   *shown* locked and never guards anything (`Storage::accepts` needs the module).
10. **A refused click in the card slot can show on the client until the menu is
    opened again.** The engine's client-side prediction applies a click on the card
    slot without asking the mod's rule, and a server refusal sends no correction,
    so `ctl screen` (and a player's own screen) shows the stack there until the
    next menu open re-syncs it. The server never takes it: `/rftoolsstorage info`
    and the reopened grid are the truth, and the e2e asserts both. Grid slots do
    not do this (their refusals are predicted).

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
- The GUI's **list controls** (search, sort, compact, cycle, visible-size) and the
  **two card slots beside the module's** (the type module is `@todo 1.14` upstream,
  and the filter slot takes RFToolsBase's `FilterModuleItem`, which this port does
  not have), the module tooltip, JEI/TOP/XNet/RFToolsUtility compat, the
  `/rftoolsstorage list|restore` commands and the config. (The padlock and the
  storage grid are slice 2; the **module slot** is slice 3. `list`/`restore` are
  keyed by upstream's world-level `StorageHolder` UUID map, which this port
  replaced — deviation 1 — so they have nothing to list.)
- The `rftoolsbase:machine_frame` ingredient and `rftoolsbase`'s casing textures
  in the model chain (the port copied the textures into its own namespace, so
  only the recipe needs RFToolsBase installed).

## Hook table

The SDK form (this package) — one `impl Hooks` and one `BlockEntityPart`; the
retired helper files and their mapping are in
[`## Migration to the SDK format`](#migration-to-the-sdk-format--version-020).

| Hook (SDK 0.2) | What it does |
|---|---|
| `register` → `Registrar` | declares the block-entity type `rftoolsstorage:storage_block_entity` over the block, attaches its `BlockEntityPart`, declares the menu's `MenuPart`, and declares the block's container (100 slots, every face, inputs and outputs) with its menu |
| `BlockEntityPart::block_entity` | gives `rftoolsstorage:modular_storage` a block entity with empty saved data (the module tier and the lock live in its compound; `Items` are the container's) |
| `right_click_block` (cancel) | install / insert / seal / withdraw, and the action-bar readout |
| `register_commands` | `/rftoolsstorage info <pos>` and `/rftoolsstorage lock <pos>` (typed `block_pos`) |
| `run_mod_command` | their answers |
| `screen` / `screen_draw` / `screen_click` (BlockEntityPart, API 19/20) | the menu's 366 x 210 layout, the title, the inventory label and the padlock; a press on the padlock's rectangle answers button 0 |
| `menu_button` (BlockEntityPart, API 20) | the padlock: flip the lock of a storage that has a module, answer on the action bar, push the state into the menu's data slot |
| `menu_slot_rules` (MenuPart, API 130) | a grid slot takes a stack only while locked, and only when it is empty or holds that same kind; the card slot takes a storage module (and, once it holds one, anything — deviation 7) |
| `player_container_open` (API 152) | our menu's open fills the data slot with the block's real lock state (data slots are not saved) |
| `PlayerRef::open_menu` | the plain click opens the menu itself: the block's `end_rod` template has no use of its own, so the mod asks for it as `BlockMenuProvider` would |
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

## Slice 2 — the storage's menu (version 0.2.2)

The jar's GUI, as far as the pin expresses it: `ModularStorageContainer`'s grid and
its padlock, not its list widget.

- **The menu.** `data/rftoolsstorage/horizon/menu/modular_storage.json` declares the menu type
  `rftoolsstorage:modular_storage` (100 slots, 1 data slot, no `results`); the block's
  container names it, and the click opens it with `PlayerRef::open_menu` — the `end_rod`
  template has no use of its own, so nothing else would open it. The panel is 366 x 210: the
  100 slots in a 10 x 10 grid on the left, the player's inventory to its right, the title, the
  inventory label and a 46 x 16 padlock at the top left. It is under a double chest's 222 px,
  the tallest screen the smallest vanilla GUI (240 px) holds; the pin's screen layout is fixed
  when the menu opens, so a scroll could not move a slot — a grid is what fits.
- **The grid's rule** is upstream's `Slot.mayPlace`: `if (!isLocked()) return false;` then
  `getItemHandler().isItemValid(...)` — nothing while unlocked, and a slot is empty or already
  holds that kind (`runtime::slot_accepts`, pinned by a test). The engine's own stack limit
  (the item's size, never past 64 through the engine's 1..99 clamp) is the per-slot cap.
- **The padlock** is upstream's `lockButton`: `screen_click` on its rectangle answers button 0,
  `menu_button` flips the lock of a storage that has a module (`runtime::lock_press`), answers
  on the action bar, and pushes the state into the menu's one data slot so the button redraws;
  `player_container_open` fills that slot from the block when the menu opens (data slots are
  not saved).
- **The quick gestures stay**, behind sneak: the module install, the whole-stack insert and the
  seal/withdraw click are unchanged code, and the old e2e's assertions for them still hold.

### Red → green (the menu's rules)

The two rules were written first and stubbed (`slot_accepts` answered `false`, `lock_press`
always `NoModule`), and the suite was run **before** implementing them
(`ports/_shared/evidence/rftoolsstorage/slice2-red.log`, `TEST_EXIT:101`):

```
test runtime::tests::the_grid_takes_a_stack_only_when_locked_and_in_a_slot_of_its_own ... FAILED
test runtime::tests::the_padlock_needs_a_module_and_then_flips_the_lock ... FAILED
...
thread 'runtime::tests::the_grid_takes_a_stack_only_when_locked_and_in_a_slot_of_its_own' panicked at src/runtime.rs:560:9:
a locked, empty slot takes a stack
thread 'runtime::tests::the_padlock_needs_a_module_and_then_flips_the_lock' panicked at src/runtime.rs:584:9:
assertion `left == right` failed
  left: NoModule
 right: Locked
test result: FAILED. 15 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

Implemented, then **17 passed, 0 failed** (`sdk-test.log`).

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built package | `e2e-menu.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-menu-control.log` | **PASS** + `EXIT:0` — `{"error":"unknown item rftoolsstorage:modular_storage"}` |
| counterfactual (a vanilla barrel in the same rig) | `e2e-menu-counterfactual.log` | **PASS** + `EXIT:0` — a 27-slot chest menu whose slot takes the stack the storage refuses |
| `SELFTEST=1` (asserts a 27-slot menu on the storage) | `e2e-menu-selftest.log` | **FAIL** + `EXIT:1` — *"the storage's menu shows 100 slots, not 27: the grid assertion is alive"* |

What `e2e-menu.log` asserts, in order: the block places; a fresh block reads `0/0 empty no
module`; **a plain click opens a 100-slot menu** (panel origin from `ctl ui`); with no module
the grid leaves slot 300 empty and the sneak gesture says `no storage module`; a module
installs on a sneak click (`0/6400 empty unlocked`); **unlocked, the grid still refuses**
(`slot 300: '0'`); the **padlock press** locks it (`0/6400 empty locked`); a locked storage
stores the stack clicked into slot 300 (`64/6400`); **dirt is refused by the stone's slot and
takes slot 399 — the grid's last one**; a click on slot 300 gives the stack back (`64/6400`);
`/rftoolsstorage lock` unlocks and re-locks; the harness fills 99 slots to `6400/6400` and a
further click moves nothing; **one stack withdrawn through the grid** (`6400 -> 6336`) is
taken back by the sneak gesture (`6400`); a relaunch keeps `6400`, `locked` and the reopened
grid's slot 300; **the carried `storage_module0` recipe crafts in a real crafting table** (the
result is the module ×1, in the inventory) while **a stick in the iron's cell and a stone block
where `rftoolsbase:machine_frame` belongs both craft nothing**; and the barrel leg closes the
differential.

### Coverage (every jar family in this port)

| Family | State |
|---|---|
| `modular_storage` block, its states/models/loot/tag | **tested** (place + read back, `e2e-menu.log`) |
| the storage slot rules (locked, one kind a slot, 64 a slot, 100 slots) | **tested** (grid clicks in `e2e-menu.log`, rules in `runtime` tests) |
| the menu: 100-slot grid, the padlock, the screen | **tested** (`e2e-menu.log`) |
| the quick gestures (module, insert, seal, withdraw) | **tested** (`e2e-menu.log`) |
| `storage_module0` recipe (chest, 2 gold nuggets, iron ingot, 2 quartz, 2 redstone) | **tested** — crafted in a real crafting table, the result is `rftoolsstorage:storage_module0` ×1 and it reaches the inventory; a stick in the iron's cell crafts nothing (`e2e-menu.log`) |
| `modular_storage` recipe | **blocked** on `rftoolsbase:machine_frame`: with a stone block in that cell the same pattern crafts nothing, so the frame is what the recipe needs and RFToolsBase is not in this game build (`e2e-menu.log` control) |
| tiers 2-4 (`storage_module1..3`) | **blocked**(container per block, menu ≤ 256 slots) |
| the remote module/block, `storage_scanner`, `crafting_manager` | **not ported** (later slices) |
| the list controls (search/sort/compact/cycle) and the module/card slots in the menu | **not ported** (next slice; the sneak click installs the module, and there is no way to take it out) |
| commands `info`/`lock` | **tested** (`e2e-menu.log`, parsed replies) |
| `/rftoolsstorage list|restore`, config, compat | **not ported** |

### Traps this slice paid for

1. **A template block with no use of its own does not open its container's menu.** The engine's
   block-use path only emits `OpenMenu` for the vanilla classes (`barrel`, `chest`, ..., a
   `MenuProvider` block); an `end_rod` template has none, so the mod must call
   `PlayerRef::open_menu` itself. `functionalstorage` hides this by using the `barrel` template.
2. **`ctl key shift down` needs a tick before `interact`.** Snapshotted in the same tick, the
   click still reads `is_secondary_use_active() == false` and the plain path runs. `ctl wait 6`
   between them is what makes the sneak gesture reach the mod (the first three probes missed it).
3. **A fresh mod block entity's item vector is empty**, not the container's slot count, until
   something writes through the menu (`put_back` grows it). `ctl container` therefore answers
   `container is full` on an untouched block; the e2e fills after a click, and the fill loop
   stops on the first error instead of trusting `script --keep-going`.
4. **`menu_slot_rules` asks per slot and stack** while a click plans: answer from the block
   entity's saved compound (one read per slot) and nothing else; a shift-click asks many.

## Slice 3 — the module's card slot (version 0.2.3)

The gap line slice 2 left: *"the **module slot and the three card slots** (upstream's GUI have
them; here the sneak click installs the module and nothing takes it out)"*. Upstream's
`ModularStorageContainer` keeps the module in a real card slot
(`SLOT_STORAGE_MODULE = 0`, `mayPlace` = `if (isLocked()) return false;` then
`stack.getItem() instanceof StorageModuleItem`), so a module can be swapped; the port only had
the sneak click, which is one-way.

- **The slot.** The block's container is **101 slots**: 0..99 the storage (tier 1's hundred) and
  100 the **card slot**, holding the module item itself. `faces`/`inputs`/`outputs` stay 0..99, so
  hoppers never reach the card (upstream's card handler is not the block's item capability). The
  menu type declares 101 slots and the screen places the card above the grid (8, 4), with the
  padlock moved to (32, 6).
- **The truth is the item.** The tier comes from the card slot's stack (`module_tier`), not from
  the saved compound, so taking the module out *is* the removal — the game's own item move, no
  command and no new hook. `write_storage` never writes the card slot, and writes no storage slot
  at all while no module is in.
- **The contents outlive the module.** A block with an empty card slot keeps its hundred slots
  (hidden: `0/0 empty no module`), and the module back in the slot finds them — upstream's own
  model (its contents live in a world-level map keyed by the module), with this port's one store
  per block (deviation 1).
- **The rule.** `runtime::card_accepts` (a storage module, or anything once it holds one — see
  deviation 7 for the measurement that forced the second half) and `runtime::slot_accepts` (the
  grid's, unchanged from slice 2).
- **A 0.2.2 block is migrated, not lost.** 0.2.2 recorded the tier in the compound's `module` key.
  A block found with that key and an empty card slot has the module item written into the slot
  (and the key cleared) the first time its menu opens, its padlock is pressed, a click reaches it
  or `/rftoolsstorage info` reads it — so no published world loses its module.
- **The sneak click still installs**, now *into the slot* (`put_card`), so both paths leave the
  module where the menu can take it back out.

### Red → green (the rules)

`slot_accepts`/`card_accepts` were stubbed to `false` and the suite run
(`ports/_shared/evidence/rftoolsstorage/slice3-red.log`, `TEST_EXIT:101`):

```
test runtime::tests::a_module_goes_in_only_while_unlocked_and_only_once ... FAILED
test runtime::tests::the_card_slot_takes_a_module_and_never_blocks_another_slot ... FAILED
test runtime::tests::the_grid_takes_a_stack_only_when_locked_and_in_a_slot_of_its_own ... FAILED
...
test result: FAILED. 18 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The slice's other new rules had their own red in the first design — `module_id`, the detach/hide
gate and the storage accessors — `slice3-red-first-design.log`, `16 passed; 5 failed`:

```
test runtime::tests::a_moduleless_storage_reads_as_empty_whatever_the_block_holds ... FAILED
test runtime::tests::a_module_id_round_trips_through_its_tier ... FAILED
test runtime::tests::a_removed_module_hides_the_contents_and_its_return_finds_them ... FAILED
...
test result: FAILED. 16 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Implemented, then **21 passed, 0 failed** (`sdk-test.log`).

### The card slot's answer: what the engine actually asks (measured)

The first design gave the card slot upstream's own per-slot rule. In game that *broke the grid*:
a click on a **locked** storage's first storage slot stored nothing. Instrumenting the hook (the
asked index and an answer encoded into `max_stack`, read back through the readout — the probe is
in this slice's history, not in the package) showed:

| click on | the ask the engine applies |
|---|---|
| the menu's first slot (300) | `slot = 0` |
| the menu's hundredth slot (399) | `slot = 99` |
| the card slot (400) | `slot = 100` |

so the index is honest — but the engine also consults the **card slot** while it plans a click on
another slot, and a `may_place = false` there blocks the click. That is why `card_accepts` answers
"anything" once the card holds the module: the alternative was a storage whose grid refused every
stack while locked. Recorded as deviation 7 and as an engine issue worth reporting: *a
`menu_slot_rules` refusal on one slot must not cancel a click on another*.

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built package | `e2e-cardslot.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-cardslot-control.log` | **PASS** + `EXIT:0` — `{"error":"unknown item rftoolsstorage:modular_storage"}` |
| counterfactual (a vanilla barrel in the same rig) | `e2e-cardslot-counterfactual.log` | **PASS** + `EXIT:0` — a 27-slot chest menu with no card slot at 400 |
| `SELFTEST=1` (asserts the barrel's 27-slot menu on the storage) | `e2e-cardslot-selftest.log` | **FAIL** + `EXIT:1` — *"the storage's menu shows 101 slots, not 27: the card-slot assertion is alive"* |
| `NEGATIVE=1` (asserts the card slot *took* the stone) | `e2e-cardslot-negative.log` | **FAIL** + `EXIT:1` — *"the card slot refused the stone … so this inverted assertion fails as it must"* |

What `e2e-cardslot.log` asserts, in order (every number before and after): the block places and
reads `0/0 empty no module`; **the menu shows 101 container slots** and the card slot is empty;
**a stone is refused by the card slot** — the server reads `0/0` and the stone is still in the
inventory, and the reopened menu's card slot is empty; **a `storage_module0` put into the card
slot sets the storage** (`0/6400 empty unlocked`, none left in the inventory, the reopened menu's
card slot holds it ×1); **it comes back out through the slot** (`0/0 empty no module`, the module
in the inventory) — the gap this slice closes; **the sneak click still installs it** (and leaves
it in the slot); unlocked, the grid still refuses a stack; the padlock locks; a locked storage
stores 64 through the grid (`64/6400`, the re-synced grid shows 64 in slot 300); while locked, the
sneak gesture refuses a module (`locked — unlock it first`); **the module comes out of the card
slot and the block keeps its contents** — `0/0 empty no module`, then the module back in the slot
reads **`64/6400 minecraft:stone locked`**: the 64 stone survived the module's removal; one kind a
slot (dirt refused by the stone's slot, then the grid's last slot, 399, takes it, `128/6400`); a
click on a filled slot gives the stack back; `/rftoolsstorage lock` toggles; **the storage fills to
6,400 with the card slot untouched** and a further click moves nothing; a withdrawal
(`6400 -> 6336`) is taken back; **a relaunch keeps the contents, the lock, the grid and the module
in the card slot**; the carried `storage_module0` recipe crafts in a real crafting table (and a
stick/a block where its keys go craft nothing).

### Coverage (every jar family in this port)

| Family | State |
|---|---|
| `modular_storage` block, its states/models/loot/tag | **tested** (place + read back, `e2e-cardslot.log`) |
| the storage slot rules (locked, one kind a slot, 64 a slot, 100 slots) | **tested** (grid clicks + the readout, `e2e-cardslot.log`; rules in `runtime` tests) |
| the menu: 100-slot grid, the card slot, the padlock, the screen | **tested** (`e2e-cardslot.log`) |
| the **module slot**: install, remove, the tier it sets, the contents it keeps | **tested** (`e2e-cardslot.log`: 0 → 6400 → 64 kept across a removal → 6400/6400) |
| the quick gestures (module, insert, seal, withdraw) | **tested** (`e2e-cardslot.log`) |
| the 0.2.2 module migration (compound `module` → the card slot's item) | **unit-tested** rule (`module_id` round trip); the glue path runs on the first menu open, padlock press, click or `/info` of an old block. **Not in game**: building a 0.2.2-saved block needs a world file this harness does not write |
| `storage_module0` recipe | **tested** — crafted in a real table, result ×1, reaches the inventory; a stick in the iron's cell crafts nothing |
| `modular_storage` recipe | **blocked** on `rftoolsbase:machine_frame` (a stone block in that cell crafts nothing; RFToolsBase is not in this build) |
| tiers 2-4 (`storage_module1..3`) | **blocked**(container per block, menu ≤ 256 slots) |
| the two card slots beside the module's (type module = `@todo 1.14`; filter = RFToolsBase's item) | **not ported** (upstream's type slot is dead code; the filter item belongs to a port this build does not carry) |
| the remote module/block, `storage_scanner`, `crafting_manager` | **not ported** (later slices) |
| the list controls (search/sort/compact/cycle/visible-size) | **not ported** (next slice; the pin's screen is a fixed slot layout) |
| commands `info`/`lock` | **tested** (`e2e-cardslot.log`, parsed replies) |
| `/rftoolsstorage list|restore`, config, compat | **not ported** (`list`/`restore` need upstream's UUID store) |

### Traps this slice paid for

1. **A mod menu's client copy is not the truth.** A click the server refuses in the *card* slot
   stays on the client's screen until the menu is reopened (no corrective packet is sent for a
   refusal), while a grid slot's refusal is predicted locally and shows at once. Every slot read
   in the e2e goes through `sync_menu` (close + reopen) and every state claim through
   `/rftoolsstorage info`.
2. **`ctl command "clear @s"` clears nothing from the console path.** It answers `ok:true`
   anyway (the campaign's known trap), so leftovers double the counts; `clear @a` is the form
   that works (`ctl command` runs as the console, which has no `@s`).
3. **`ctl container` writes the container's first empty slot, card slot included.** The fill loop
   stops at the storage's hundred slots and asserts the card slot still holds the module.
4. **A locked storage can be emptied of its module.** The pin's menu rules gate placement only
   (there is no `may_pickup` answer a mod can give), so the port lets the module come out and
   treats a module-less storage as unlocked and inert — the alternative would be a block whose
   module could never go back in.
5. **A storage slot's stack limit is the item's, not our rule's.** The engine clamps
   `max_stack` to 1..99 and then to the item's own size; a module is `stacksTo(1)`, so a click
   puts one in and leaves the rest on the cursor (the e2e gives exactly one).

## Slice 4 — the contents travel on the item (version 0.2.4)

The gap line slice 3 left, verbatim from [`## Deliberate deviations`](#deliberate-deviations) 5 and
[`FINDINGS.md`](FINDINGS.md) §4:

> **A break spills the contents** as loose drops instead of carrying them in the item.

Upstream keeps a broken storage's contents for the module (`StorageHolder`, keyed by the module's
UUID), so placing the block back with its module finds them; this port keeps one store per block
(deviation 1), so the equivalent is the block's **own item** carrying them. It fits the
**machines** tier: a block entity's saved data, an item component and the carried rules — no new
family, no renderer, no new vanilla hook.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API `0.2`, game 26.3. **Version 0.2.4.**

### What it does

- **Break.** `BlockPart::on_destroyed_by_player` (`IBlockExtension.onDestroyedByPlayer`, asked
  before the removal) checks `will_harvest` — a creative break or a tool the block refuses drops
  nothing and keeps the vanilla spill — reads the container's 101 slots with
  `ItemStack::to_nbt` + `Slot`, writes them under the saved key `carried`, **empties the slots**
  and remembers the saved data for the drop. The engine's own container spill
  (`PreRemove::Contents`) then finds an empty container: nothing scatters.
- **The payload.** `Hooks::block_drops` reads the stash and puts the saved data on the block's own
  drop as `minecraft:block_entity_data` tagged
  `id = "rftoolsstorage:storage_block_entity"` (vanilla's `TypedEntityData.loadInto` loads a
  payload only when its `id` names the block entity). **`Items` is stripped**: the engine spills a
  mod container's stacks itself, so a payload that also carried them would duplicate every stack.
- **Place.** `ItemPart::on_block_placed_by` on `rftoolsstorage:modular_storage` restores the
  carried stacks — the card slot's module and the lock (both in the payload) with them — then
  clears the key so they travel once. `runtime::carried_slot_in` and `runtime::restored_count`
  clamp it: only the container's 100 slots and the card slot, each at most the item's own stack
  size (a `/give`n payload naming 200 stone places 64).
- **The block's own recovery.** The same `carried` key stays in the block entity's saved data
  until the removal, so a removal the level refuses (the block stays) loses nothing:
  `adopt_carried` puts the contents back on the next menu open, click, padlock or `/info`, the way
  `adopt_legacy_module` does for a 0.2.2 block.

### Red → green (the rules)

`carried_slot_in`/`restored_count` were stubbed (`false` / `count`) and the suite run
(`ports/_shared/evidence/rftoolsstorage/slice4-red.log`, `TEST_EXIT:101`):

```
---- runtime::tests::a_carried_slot_comes_back_clamped_and_only_where_the_block_has_one stdout ----

thread 'runtime::tests::a_carried_slot_comes_back_clamped_and_only_where_the_block_has_one' panicked at src/runtime.rs:663:9:
assertion failed: carried_slot_in(100, 100, 0)
...
test result: FAILED. 21 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
TEST_EXIT:101
```

Implemented, then **24 passed, 0 failed** (`sdk-test.log`): the runtime's 21 and three glue tests,
one of the runtime's and two of the glue's new — `a_carried_slot_comes_back_clamped_and_only_where_the_block_has_one`,
`the_payload_carries_the_contents_and_never_items` and `a_break_that_moved_nothing_aside_carries_nothing`.

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built package | `e2e-carried.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-carried-control.log` | **FAIL** + `EXIT:1` — `{"error":"unknown item rftoolsstorage:modular_storage"}` |
| counterfactual (a vanilla barrel in the same rig) | `e2e-carried-counterfactual.log` | **PASS** + `EXIT:0` — a 27-slot chest menu whose first slot takes the stack |
| `SELFTEST=1` (asserts the break *spilled* the contents) | `e2e-carried-selftest.log` | **FAIL** + `EXIT:1` — *"the survival break carried the storage's contents on the block's item (1 drop(s) on the ground, 0 stone spilled), so this inverted assertion fails as it must"* |

What `e2e-carried.log` asserts (every number before and after; the earlier phases are slice 2/3's
and still pass): a fresh storage at the rig's edge takes a module, locks and stores 64 stone; the
player switches to **survival** and mines it with a diamond pickaxe, and the break's own reply is
`{"drops":{"rftoolsstorage:modular_storage":1},"picked_up":{},"items_left_on_ground":1}` — **no
`minecraft:stone`**, one item on the ground; the picked-up item's `SelectedItem` is

```
{components: {"minecraft:block_entity_data": {carried: [{count: 64, Slot: 0b, id: "minecraft:stone"},
 {count: 1, Slot: 100b, id: "rftoolsstorage:storage_module0"}], module: -1,
 id: "rftoolsstorage:storage_block_entity", locked: 1b}}, count: 1, id: "rftoolsstorage:modular_storage"}
```

— the contents, the lock and the module, and **no `Items`**; placed back, the block's **own saved
data** (`/data get block`, no mod code in the path) reads `Items[0] = 64 x minecraft:stone`,
`Items[1]` slot `100` = the module, `carried = []`; `/rftoolsstorage info` reads
`64/6400 minecraft:stone locked` and the reopened menu's slot 300 holds 64 with the module in the
card slot; an item doctored to 200 stone places 64; and the **level-side counterfactual** — the
same block, `/setblock … destroy` — spills `stone 64, modules 1, storage items 1`, drops an item
with no payload, and the block placed back reads `0/0 empty no module` (`0/6400` with the module
put back).

### Coverage (the families this slice changes)

| Family | State |
|---|---|
| a broken storage's contents on its item (`minecraft:block_entity_data`, no `Items`) | **tested** — break, payload, clamp, place-back (`e2e-carried.log`) |
| the lock and the card slot's module travelling with the contents | **tested** — `locked: 1b` and slot 100 in the payload; `Items[1]` slot 100 after placement |
| a doctored payload clamped to the slot | **tested in game** (200 → 64) and by `restored_count` |
| a level-side break (no player) | **tested (the opposite)** — spills the stacks, plain block item (`e2e-carried.log`) |
| a removal the level refuses | rule unit-tested (`carried_slot_in`), the `adopt_carried` path is the same one the 0.2.2 migration uses; not forced in game |

### Deviations this slice adds

1. **A transient break stash.** `block_drops` runs after the removal and the engine snapshots the
   block entity's saved data *before* it (measured), so the stash is a short-lived list in the mod
   keyed by `(dimension, pos)` and expiring after 2 s (functionalstorage's shape). It holds one
   break's data for the same tick; nothing survives a relaunch. The same data is also written into
   the block entity's saved data for the refused-removal recovery above.
2. **A level-side break does not carry the contents.** A piston, water, an explosion or
   `/setblock … destroy` gives no `onDestroyedByPlayer` moment, so the vanilla spill stands. That is
   the differential, and it matches vanilla: a block entity gets no say there either.
3. **The carried key is this port's, not `Items`.** The payload moves the container's stacks under
   `carried`, so vanilla's placement does not apply them itself; `on_block_placed_by` writes them
   with the clamp. The alternative — carrying `Items` — duplicates every stack (the engine spills
   them as well).
4. **`block_drops` never trusts the event's saved data.** The engine reads it before the removal
   (see the trap below); reading it would attach a *pre-break* copy (with `Items`) or nothing. The
   stash is the source, and the payload's `Items` stripping is a second guard.

### Traps this slice paid for

1. **`BlockEntityRef::data()` in `block_drops` is the pre-removal snapshot, not the stash.** The
   engine snapshots the block entity for `block_drops` **before** `onDestroyedByPlayer` runs
   (`live_block_entity` → `saved_block_entity_data` → `hold_broken`), so the event's saved data has
   the container's `Items` and none of the break's own writes. The first version trusted it and
   dropped a plain item while the stone vanished into the emptied container; the mod-side stash is
   the fix.
2. **One Part object answers both its halves.** `StorageBlock` implements `BlockPart` and
   `BlockEntityPart`, and `Registrar::block_entity_part` attaches it to the type: the break's
   `on_destroyed_by_player` fired from the first build (the container's spill stopped, which is how
   the "hook did not run" theory was disproved — the payload was missing for a different reason,
   trap 1). A separate `block_part(block, …)` call is only needed for a Part whose block is not its
   block-entity type.
3. **`ctl place` refuses a cell the player's box overlaps** ("the player is standing in that cell"),
   and after `ctl collect` walks the player onto the drop the box is in the block's own cell. The
   e2e stands two blocks clear and places in creative (the payload path is mode-independent); the
   retry prints the player's position when a placement is refused.
4. **A saved `Slot` is a byte** (`Slot: 0b`, `100b`), so `data get … Items[1].Slot` prints `100b`,
   not `100`; and an item NBT `count` must fit the writer: `count: 200b` is refused, `count: 200`
   is not.
5. **`ctl break`'s reply is the server's report** (`drops`, `picked_up`, `items_left_on_ground`):
   the spill check is `drops["minecraft:stone"] == 0`, not an entity count on the client.

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

## Verification

Full evidence in [`VALIDATION.md`](VALIDATION.md); the e2e logs are in
`ports/_shared/evidence/rftoolsstorage-e2e/` (the game legs) and
`ports/_shared/evidence/rftoolsstorage/` (the ladder, package and fits).

**0.2.4 — the contents travel on the item (2026-11-21, pin `0a875da`)**

| Step | Command | Result |
|---|---|---|
| Red proof | `cargo test` with `carried_slot_in`/`restored_count` stubbed | **FAILED. 21 passed; 1 failed**, `TEST_EXIT:101` (`slice4-red.log`); quoted above |
| Runtime + glue tests | `cargo test` | **24 passed**, 0 failed (`sdk-test.log`, `TEST_EXIT:0`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-clippy.log`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` (`sdk-test.log`) |
| SDK gate | `cargo horizon-mod check` | EXIT 0 — *"…uses 458 items of the API (… 16 hooks …). Made for 0a875da."* (`sdk-check.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **64,513 B**, SHA-256 `a2d6add4…d90d60`, **`cmp`-identical**; 58 entries, no `Cargo.lock` (`sdk-build.log`, `sdk-sha256.txt`) |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.4 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **64,513 B**, `a2d6add4…d90d60`, EXIT 0 (`sdk-check.log`) |
| Fits the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da91dbb…` (same build as `0a875da`) | `0a875da  made for`, EXIT 0, sha `a2d6add4…d90d60` (`sdk-fits.log`) |
| Fits another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8acfd…` (same build as `0b14f8a`) | `0b14f8a  fits`, EXIT 0, sha `a2d6add4…d90d60` (`sdk-fits.log`) |
| In game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` | **PASS** + `EXIT:0` (`e2e-carried.log`) |
| Control | the same rig on the Nightly without the package | **FAIL** + `EXIT:1` (`e2e-carried-control.log`) |
| Counterfactual | a vanilla barrel in the same rig | **PASS** + `EXIT:0` (`e2e-carried-counterfactual.log`) |
| Selftest | `SELFTEST=1` (asserts the break spilled the contents) | **FAIL** + `EXIT:1` (`e2e-carried-selftest.log`) |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=4 ok=4`, exit 0 |
| Package gate | `verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 |
| Skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS |

**0.2.3 — the card slot (2026-11-21, pin `0a875da`)**

| Step | Command | Result |
|---|---|---|
| Red proof | `cargo test` with `slot_accepts`/`card_accepts` stubbed | **3 failed**, 18 passed (`slice3-red.log`, `TEST_EXIT:101`); the detach/hide rules' own red is `slice3-red-first-design.log` (`5 failed`, `TEST_EXIT:101`); both quoted above |
| Runtime + glue tests | `cargo test` | **21 passed**, 0 failed (`sdk-test.log`, `TEST_EXIT:0`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-clippy.log`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` (`sdk-test.log`) |
| SDK gate | `cargo horizon-mod check` | EXIT 0 — *"…uses 395 items of the API (… 13 hooks …). Made for 0a875da."* (`sdk-check.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **57,648 B**, SHA-256 `7e070957…2f3f`, **`cmp`-identical**; 58 entries, no `Cargo.lock` (`sdk-sha256.txt`) |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.3 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **57,648 B**, `7e070957…2f3f`, EXIT 0 (`sdk-check.log`) |
| Fits the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, EXIT 0 (`sdk-check.log`) |
| Fits another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, EXIT 0 (`sdk-check.log`) |
| In game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` | **PASS** + `EXIT:0` (`e2e-cardslot.log`); control `EXIT:0`, counterfactual `EXIT:0`, `SELFTEST=1` **FAIL** + `EXIT:1`, `NEGATIVE=1` **FAIL** + `EXIT:1` (`e2e-cardslot-*.log`) |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=11 ok=11`, exit 0 |
| Package gate | `verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 |
| Skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS |

**0.2.2 — the menu (2026-11-21, pin `0a875da`)**

| Step | Command | Result |
|---|---|---|
| Red proof | `cargo test` with the two new rules stubbed | **2 failed**, 15 passed (`slice2-red.log`, `TEST_EXIT:101`); evidence quoted above |
| Runtime + glue tests | `cargo test` | **17 passed**, 0 failed (`sdk-test.log`, `TEST_EXIT:0`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-test.log`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` |
| SDK gate | `cargo horizon-mod check` | EXIT 0 — *"…uses 372 items of the API (… 12 hooks …). Made for 0a875da."* (`sdk-check.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | 53,878 B, SHA-256 `fd5d7213…fc82`, **`cmp`-identical**; 58 entries, no `Cargo.lock` (`sdk-sha256.txt`) |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.2 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, EXIT 0 (`sdk-check.log`) |
| Fits the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, EXIT 0 (`sdk-check.log`) |
| Fits another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, EXIT 0 (`sdk-check.log`) |
| In game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` | **PASS** + `EXIT:0`, `e2e-menu.log`; control `EXIT:0`, counterfactual `EXIT:0`, `SELFTEST=1` **FAIL** + `EXIT:1` (`e2e-menu-*.log`) |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=6 ok=6`, exit 0 |
| Skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS |

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
| `runtime.rs` untouched (0.2.0 only) | `unzip -p rftoolsstorage.hmod src/runtime.rs \| sha256sum` | `01691fc1…8615f4` = the retired file, byte for byte (`sdk-sha256.txt`, before the review fixes) |
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

## Review fixes (2026-11-18)

`var/review-all/1201-rftoolsstorage.md` (the held PR), the one finding fixed; version **0.2.1**
(46,987 B, sha256 `32f0505ef59a5eb1aa476578430a024bb02e9913edca853cf342bd5d9320a0f6`).

- **Foreign-container wipe through `/rftoolsstorage lock` (src/support.rs:532, :555, :265).**
  The command took a mutable handle to any addressed block entity; a foreign container decoded as
  an empty storage, `toggle_lock` returned false, and the write-back still cleared its slots.
  Three changes: `mutate_checked` refuses a position that is not this mod's modular storage
  *before* reading a mutable handle (`is_storage`), the lock commits only when `toggle_lock`
  actually returned true (`Option`-returning change — a failed mutation writes nothing), and both
  `lock` and `info` are gated to an operator or a player in the same level within 8 blocks.
  Pure rule: `runtime::may_inspect`, pinned by
  `a_readout_or_lock_needs_an_operator_reach_or_the_same_level`.
- **Launcher build needed one dependency change.** The package depended on a second path crate
  (`horizon-core`, for `JavaString`); `launcher-game.sh run` failed with
  `error: No such file or directory (os error 2)` because the launcher resolves the API's own SDK
  only. Saved-compound keys are now read through `Compound::iter` (`support::field`) and written
  with `KEY.into()`, and `Cargo.toml` depends on `horizon-mod-api` alone. The in-game leg then
  built and passed.
- `runtime.rs` gained the reach rule and its test (16 tests total); it is no longer byte-identical
  to the retired bundle.

Ladder: `cargo test` 15 passed (the runtime's 14 plus the added reach test); `cargo clippy
--all-targets -- -D warnings` exit 0; `cargo fmt --check`
clean; `cargo horizon-mod build` twice byte-identical; `check` and `fits` exit 0. In-game:
`launcher-game.sh run rftoolsstorage` → `PASS` + `EXIT:0`
(`ports/_shared/evidence/rftoolsstorage-e2e/e2e-launcher.log`); `GAME=$CONTROL_GAME` → `FAIL`
(`cannot give rftoolsstorage:modular_storage`) + `EXIT:1`.
