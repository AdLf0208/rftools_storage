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

**Format: the SDK package (2026-11-18), version 0.2.14.** One flat `.hmod` —
`horizon-mod.toml` + `Cargo.toml` + `src/` + `assets/` + `data/` — built by the
launcher against SDK nightly **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** with
horizon-mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** (see
[`## Migration to the SDK format`](#migration-to-the-sdk-format--version-020) and
[`## Slice 2`](#slice-2--the-storages-menu-version-022),
[`## Slice 3`](#slice-3--the-module-s-card-slot-version-023),
[`## Slice 4`](#slice-4--the-contents-travel-on-the-item-version-024),
[`## Slice 5`](#slice-5--the-storage-scanner-version-025),
[`## Slice 6`](#slice-6--the-scanners-item-slots-and-the-inject-tick-version-026),
[`## Slice 7`](#slice-7--the-scanners-request-path-version-027),
[`## Slice 8`](#slice-8--the-scanners-give-from-screen-path-version-028),
[`## Slice 9`](#slice-9--the-scanners-insert-from-screen-path-version-029),
[`## Slice 10`](#slice-10--the-scans-search-filter-version-0210),
[`## Slice 11`](#slice-11--the-scanner-lists-sort-choice-version-0211),
[`## Slice 12`](#slice-12--the-scanners-routable-star-list-version-0212),
[`## Slice 13`](#slice-13--the-scanners-export-choice-version-0213),
[`## Slice 14`](#slice-14--the-lists-row-contents-view-version-0214)).
The retired `.hnmod` container pinned Horizon `1d63e0a5…` / mixins `5a64af93…`;
`src/runtime.rs` was byte-identical to the retired
`mixins/rftoolsstorage/runtime.rs` until the 2026-11-18 review fixes gave it the
reach rule, and slice 2 added the menu's two rules.

Status: **ported-partial** — slice 1 (the modular storage block with a tier-1
storage module), slice 2 (its **menu**: a 100-slot grid opened by a right click,
its slot rules and the padlock), slice 3 (the menu's **card slot**, where the
module itself lives, so it can be taken back out), slice 4 (**a broken block's
contents travel on its own item**, with the lock and the module, and come back on
placement), slice 5 (the **storage scanner**: its block, its block entity, its
50,000 FE buffer and the inventory scan behind `/rftoolsstorage scan`), slice 6
(the scanner's **item slots and inject tick**: the GUI's in slot, the automation
slot and the out slot, and the tick that pushes the first two into the scanned
inventories for `rfPerInsert` each) and slice 7 (the scanner's **request path**:
`/rftoolsstorage request <pos> <item> [<count>]` pulls a kind out of the scanned
inventories, prices it as the GUI does and gives it to the player, or leaves it in
the out slot when there is no player to give it to) and slice 8 (the scanner's
**give-from-screen path**: `/rftoolsstorage give <pos> <item> [single]` hands one
item or a whole stack straight to the player for the full `rfPerRequest`, with no
out slot at all) and slice 9 (the scanner's **insert-from-screen path**:
`/rftoolsstorage insert <pos> <item> [count]` pushes a stack out of the player's
inventory into the scanned inventories and charges one `rfPerInsert`, but only when
the stack went in whole) and slice 10 (the scan's **search filter**: the GUI list's
search box as `scan`'s optional term — every word a case-insensitive name contains
or `@modid` a namespace prefix) and slice 11 (the scan list's **sort choice**:
`/rftoolsstorage sort <pos> <mode>` sets the GUI list's order — `Amount+`, `Amount-`,
`Mod` or `Name`, saved with the block so it survives a relaunch; `Tag` is
`blocked(no item-tag enumeration)`) and slice 12 (the scanner's **routable star
list**: `/rftoolsstorage routable <pos> <target> <on|off>` saves the GUI's
per-inventory star, and only a starred inventory is routed into while the scan
still lists every inventory in the box; a fresh scanner starts with nothing
starred, exactly as upstream's empty `HashSet` does) and slice 13 (the scanner's
**export choice**: `/rftoolsstorage export <pos> <current|first>` sets the GUI's
"Export to current container" choice — `StorageScannerData.exportC` — so the
tick's GUI-slot stack goes only into the inventory the list's selected row names
(`/rftoolsstorage select <pos> <target>`), and refuses rather than falling
through to the starred walk; the mode is saved under upstream's `exportc` key,
and the selection — upstream's transient `lastSelectedInventory` — is saved too,
so a relaunch does not silently change where items go) and slice 14 (the scanner
list's **row contents view**: `/rftoolsstorage contents <pos> <target|all>` lists
one row's inventory — starred or not — while `all`, the list's first "All routable"
line, lists only the **starred** inventories, and neither costs FE because a
listing is not a request) are implemented and
verified in a launcher-built game at the pin (the legs are listed per slice
below; `ports/_shared/evidence/rftoolsstorage-e2e/`). The gap list below is the
rest of the mod.

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
11. **The scanner's selected inventory is saved with the block** (slice 13).
    Upstream's `lastSelectedInventory` is a transient field: nothing saves it, so a
    relaunch makes a `current`-export scanner fall back to its (empty) selection and
    a player's items would silently go nowhere — or, under the walk, somewhere else.
    This port writes the cell into the scanner's compound under `selected` (one
    `{c: [I; x, y, z]}` compound, the shape a `routable` entry has), so the mode and
    the selection both survive a relaunch. `export first` is the way back to the walk
    (upstream's "All routable" list row, which sets the `(-1, -1, -1)` sentinel), so no
    "clear the selection" form is needed. The selection's *contents view* **is** carried
    (slice 14, `/rftoolsstorage contents <pos> <target|all>`); what `scan` still does
    is list the whole box, slice 5's deviation 1, and it stays that way so the priced
    readout and the free row view are two different surfaces.

## Upstream defects fixed

None. The two candidates were checked and left alone, on purpose:

- the **dead fill overlay** (§3 above) is a commented `@todo 1.14` deferral, not
  a broken behaviour, so it is a deviation/gap;
- `ModularStorageContainer.clicked` carries a `// @todo 1.14` for the module
  copy, and `cycle()`/`getRenderLevel()` are stubs — all deferred features of
  the GUI this slice does not carry.

## Gaps (the rest of the mod)

- `crafting_manager` (the jar's own lang calls it "WIP! … not functional yet") and
  the **storage scanner's remaining half**: the GUI list widget's **compact, cycle
  and visible-size controls** (its **search** is slice 10, carried as `scan`'s
  optional query, its **sort** is slice 11, carried as `sort`'s saved mode and
  `scan`'s order, and its **routable star list** is slice 12, carried as
  `/rftoolsstorage routable <pos> <target> <on|off>` — the set is saved and filters
  every routing walk, while the scan still lists every inventory; its **export
  choice** is slice 13, carried as `export`'s saved mode and `select`'s saved cell,
  and its per-row contents view is slice 14, carried as `contents <pos> <target|all>`
  — one row's inventory, or the starred ones for the list's "All routable" line,
  for free), the
  **remote/tablet variant**, the XNet access settings (slice 12 carries the star
  list but not `InventoryAccessSettings`' per-inventory input/output permissions,
  which need the XNet side), and the claimed-land check
  (`scannerNoRestrictions`; the pin has no `can_entity_destroy` **hook** exists at
  the pin, but it is a cancellable game event, not a query a mod can ask for a fake
  player — `blocked(no permission query)`). The **request path** is slice 7, the
  give-from-screen path is slice 8, the insert-from-screen path is slice 9, the
  search filter is slice 10, the sort is slice 11 and the star list is slice 12;
  `Tag` sorting is `blocked(no item-tag enumeration)`.
- Storage tiers 2-4 (`storage_module1..3`; their recipes need McJtyLib's
  `mcjtylib:copy_components` recipe type, which the pin does not have), the
  remote module (`storage_module6`, "DO NOT USE" in the jar's lang) and the
  remote storage block.
- The type modules (`generic`/`oredict`), the filter module, the dump module, the
  storage control module, the tablet, and the crafting grid.
- The GUI's **list controls** (compact, cycle, visible-size; its search is
  slice 10 and its sort is slice 11, both carried) and the **two card slots beside the module's** (the type module is `@todo 1.14` upstream,
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
| `register` → `Registrar` | declares the block-entity type `rftoolsstorage:storage_block_entity` over the block, attaches its `BlockEntityPart`, declares the menu's `MenuPart`, and declares the block's container (100 slots, every face, inputs and outputs) with its menu; **also** the scanner's type `rftoolsstorage:storage_scanner_block_entity` with its Part and `block_energy` (50,000 FE, 500 a move, no external drain) |
| `BlockEntityPart::block_entity` | gives `rftoolsstorage:modular_storage` a block entity with empty saved data (the module tier and the lock live in its compound; `Items` are the container's), and `rftoolsstorage:storage_scanner` one whose compound holds `energy`, `radius`, the list's `sortMode` and the `routable` star list (slice 12: one `{c: [I; x, y, z]}` compound a starred cell, upstream's shape; the key is absent until the first star) |
| `right_click_block` (cancel) | install / insert / seal / withdraw, and the action-bar readout |
| `register_commands` | `/rftoolsstorage info <pos>`, `lock <pos>`, `scan <pos> [query]`, `contents <pos> <target>` and `contents <pos> all` (two usages: a row's cell, and the list's "All routable" line as a literal), `radius <pos> <radius>`, `sort <pos> <mode>`, `routable <pos> <target> <on\|off>` (`block_pos`, `block_pos`, `word`), `export <pos> <current\|first>` (`block_pos`, `word`), `select <pos> <target>` (`block_pos`, `block_pos`), `request <pos> <item> [<count>]` (`block_pos`, `item`, `integer`), `give <pos> <item> [single]` (`block_pos`, `item`, `word`) and `insert <pos> <item> [<count>]` (`block_pos`, `item`, `integer`) |
| `run_mod_command` | their answers (the scan costs `rfPerRequest`; its optional `query` is the GUI list's search box — every word a case-insensitive name contains or `@modid` a namespace prefix, `getMatcher`; the radius is the GUI slider's 1..20; the request prices an explicit count at a tenth of `rfPerRequest`, checks the out slot and the buffer, pulls from the scanned inventories and gives the stack to the player, or leaves it in the out slot when there is no player; the give prices one item or a stack at the whole `rfPerRequest`, has **no** out slot, and pulls straight into the player in one transaction; the insert pushes the player's stack out, pricing the walk at one `rfPerInsert` and charging it **only when the stack emptied**; `routable` is the GUI's per-inventory star, saved under upstream's `routable` key — it starts empty, and only a starred cell is routed into; `contents` is the list's row contents view, `getInventoryForBlock`'s single-row branch for a cell and its `(-1, -1, -1)` "All routable" branch for `all` — the starred rows — over the saved sort, and **free**: a listing is not a request, so it never reads or spends the buffer) |
| `block_entity_ticks` / `block_entity_tick` (BlockEntityPart) | the scanner ticks: the GUI input, then the automation input, each an `rfPerInsert` attempt into the **starred** scanned inventories (slice 6; the star filter is slice 12; the GUI slot's **export choice** is slice 13 — `current` sends it only into the selected inventory and does not fall through, `first` is the walk) |
| `Registrar::block_container` | the scanner's three slots (`SLOT_IN` 0, `SLOT_OUT` 1, `SLOT_IN_AUTO` 2): automation inserts into the auto slot only, every slot extractable, no menu |
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
Slice 5: `modules/scanner/blocks/StorageScannerTileEntity.java` (the scan walk
`findInventories`/`inRange`, `countItems`, the energy fields),
`blocks/StorageScannerBlock.java`, `modules/scanner/StorageScannerConfiguration.java`
(`scannerMaxRF` 50,000, `scannerRFPerTick` 500, `rfPerRequest` 100, `rfPerInsert` 20,
`scannerNoRestrictions`), `modules/scanner/data/StorageScannerData.java` (radius 1,
`SortingMode.NAME`), `modules/scanner/StorageScannerModule.java`, and
`modules/scanner/client/GuiStorageScanner.java` (the radius slider 1..20).
Slice 6: the same `StorageScannerTileEntity.java` (`tickServer`, `injectStackInternal`,
`getInputMatcher`, `isInputFromAuto`, `isRoutable`, the `routable` set),
`blocks/StorageScannerContainer.java` (`SLOT_IN`/`SLOT_OUT`/`SLOT_IN_AUTO`, `SLOTS`),
`client/GuiStorageScanner.java` (the star toggle that fills `routable`), and
**McJtyLib** `1.21_neo` @ `ef393e608eebc188a6e1decbf110506a55cc0725`
(`container/GenericItemHandler.java`: the `Builder.insertable`/`extractable` defaults the
container's policy is read from — a library read, not a port).
Slice 8: `StorageScannerTileEntity.java` again (`giveToPlayerFromScreen`, `giveItemToPlayer`,
`giveToPlayer`, `isItemEqual`, `isOutputFromScreen`, `findInventories`' own-cell/other-scanner
filter), `modules/scanner/items/StorageControlScreenModule.java` (the click that calls it, with
`player.isShiftKeyDown()` as `single`) and `GuiStorageScanner.java`'s `requestItem` (the sibling
path, for contrast). The clone (`/tmp/opencode/rftoolsstorage-up`, never in the tree) was deleted
after these findings were recorded.
Slice 7: the same `StorageScannerTileEntity.java` (`requestStack`, `requestStackFromInv`,
`requestItem`, `giveToPlayerFromScreen`, the `isOutputFromGui`/`isRoutable` filters, the
out-slot guard and `requestStraightToInventory`), `PacketRequestItem.java` (the GUI's click
sends `amount` 1 with shift and -1 without), `GuiStorageScanner.java` (`requestItem`) and
`StorageScannerConfiguration.java` (`requestStraightToInventory` default **true**).
Slice 9: `StorageScannerTileEntity.java` again (`injectStackFromScreen`,
`checkForRoutableInventories`, `isInputFromScreen`, `findInventories`' item-capability
filter, `injectStackInternal`) and `StorageControlScreenModule.java`'s `mouseClick` (the
insert button hands `player.getItemInHand(MAIN_HAND)` over, and "insert all" loops the
inventory slots; each call writes the remainder back into that slot).
The clones (`ports/_shared/upstream/rftoolsstorage`, `…/mcjtylib`) were deleted after
these findings were recorded; slice 9's four files were read from the repository's raw
URLs at the same commit into `/tmp/opencode` and deleted after the findings were recorded.
Slice 10: `StorageScannerTileEntity.java` again (`getMatcher`, `makeSearchPredicate`,
`performSearch`) and `GuiStorageScanner.java` (`startSearch`, the search field's caller and
the client-side filter), read from the repository's raw URLs at the same commit into
`/tmp/opencode` and deleted after the findings were recorded.
Slice 11: the same `StorageScannerTileEntity.java` (`getSortMode`/`setSortMode`),
`SortingMode.java`, `StorageScannerData.java` (`DEFAULT.sortingMode`) and
`GuiStorageScanner.java` (`updateContentsList`, the comparators), read from the raw URLs
into `/tmp/opencode` and deleted after the findings were recorded.
Slice 12: `StorageScannerTileEntity.java` again (`routable`'s field, `isRoutable`,
`toggleRoutable`, the `isRoutable` filters in `injectStackInternal`/`requestItem`/
`checkForRoutableInventories`/`countItems`, and `saveAdditional`/`loadAdditional`),
`modules/scanner/tools/InventoryAccessSettings.java` (the XNet half, read to separate it
from the star), `GuiStorageScanner.java` (`addStorageLine`'s "Routable: Yes/No" choice and
the "All routable inventories" label), and **McJtyLib** `1.21_neo` @
`ef393e608eebc188a6e1decbf110506a55cc0725` `varia/BlockPosTools.java` (the `c` int-array
entry shape). Read from the raw URLs into `/tmp/opencode` and deleted after the findings
were recorded.
Slice 14: the same `StorageScannerTileEntity.java` (`getInventoryForBlock`'s two
branches and `lastSelectedInventory`, `addItemsFromInventory`/`addItemStack`'s
same-item-and-components merge, `countItems(…, starred)`'s `isRoutable` filter,
`findInventories`' row test — in range, not a `StorageScannerTileEntity`, an item
capability — and `isValid`), and `client/GuiStorageScanner.java`
(`updateContentsList`'s sort and search over whichever row is shown). Read with a
shallow clone into `/tmp/opencode/rfsprobe` at `fc1b18a7`, deleted after the
findings were recorded.
Slice 13: `StorageScannerTileEntity.java` again (`exportC`'s field and accessors,
`injectStackInternal`'s `toSelected` branch and its `return`, `tickServer`,
`getInventoryForBlock`, `lastSelectedInventory`, `getInputMatcher`/`isInputFromGui`),
`data/StorageScannerData.java` (the `exportc` codec key and `DEFAULT`),
`client/GuiStorageScanner.java` (`exportToStarred`'s two choices and their tooltips,
`getSelectedContainerPos`'s `(-1, -1, -1)` "All routable" row, `getInventoryOnServer`)
and `modules/scanner/tools/InventoryAccessSettings.java` (the XNet tests, read to confirm
they are true without XNet). Read from the raw URLs into `/tmp/opencode` and deleted
after the findings were recorded.

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

## Slice 5 — the storage scanner (version 0.2.5)

The first gap line slice 1 left, verbatim from [`## Gaps`](#gaps-the-rest-of-the-mod):

> `storage_scanner` (inventory scan, search, FE use, remote/tablet variant)

The slice is the **block, its block entity, its FE buffer and the scan** — not the GUI list
(the jar's `GuiStorageScanner` widget), not the scanner's own item slots, not the remote/tablet
variant. It fits the **machines** tier: a second block entity with a saved compound, an FE buffer
and a machine's readout, no renderer and no new vanilla hook. Upstream source read at
`fc1b18a7f070d0030671e1db1d3aab90699fd160` (`StorageScannerTileEntity`, `StorageScannerBlock`,
`StorageScannerConfiguration`, `StorageScannerData`, `StorageScannerModule`); the clone was deleted
after the findings were recorded.

### What it does

- **The block.** `rftoolsstorage:storage_scanner`, on the **`minecraft:end_rod`** template like the
  storage (the jar's own blockstate is six `facing` variants), full-cube shapes, the same
  hardness/blast/tool/sound properties, its own item, model, texture, loot table, tags and the
  jar's recipe. It sits in the creative tab beside the storage.
- **The block entity.** `rftoolsstorage:storage_scanner_block_entity`, whose saved compound holds
  the two numbers the machine is: `energy` (the game's own energy buffer key) and `radius`.
- **The FE buffer.** `Registrar::block_energy` as upstream's
  `GenericEnergyStorage(this, true, MAXENERGY, RECEIVEPERTICK)`: capacity **50,000**,
  **500** the most one fill moves, **no** external drain (the machine spends its own buffer, as
  upstream's `consumeEnergy` does). The amount lives in the saved compound, so it survives a
  relaunch and `/data merge block … {energy: n}` is the vanilla path to it.
- **The scan.** `/rftoolsstorage scan <pos>` walks the cube `pos - radius ..= pos + radius`
  (upstream's `findInventories`, `SCANNER_DEFAULT_RADIUS` 1), reads every neighbour's **item
  capability** (the game's own answer for a chest, a barrel or a mod container, as
  `Capabilities.ItemHandler.BLOCK` with no side does) slot by slot, sums one entry a kind and prints
  them sorted by item id (upstream's default `SortingMode.NAME`), then the buffer left:
  `rftoolsstorage:storage_scanner: minecraft:stone 64 (4800/50000 FE)`. A scan costs
  `rfPerRequest` (**100**); below that the reply is `no power (n/50000 FE)` and nothing is read.
  The scanner's own cell and any other storage scanner are skipped, as upstream skips its own tile
  entity.
- **The radius.** `/rftoolsstorage radius <pos> <n>` is the GUI's radius slider
  (`GuiStorageScanner`: `.realMinimum(1)`, `.realMaximum(20)`, `StorageScannerData.DEFAULT.radius =
  1`) as a command, saved in the compound and clamped to 1..20.
- **The rest of the machine's numbers** — `rfPerInsert` 20 for the automation slice — are in
  `runtime` with the currency rule, so the next slice spends the same table.

### Deviations this slice adds

1. **The readout costs `rfPerRequest`.** Upstream lists the inventories for free and charges 100
   when the player pulls an item out. This port has no list widget (a gap), so the readout *is* the
   player surface: it answers what a pull would find and costs the pull's price. Without it the FE
   buffer would have no in-game consumer at all and the FE half of the family would be unobservable.
2. **`search` is not carried.** Upstream's `getMatcher` filters the list by item name or `@modid`;
   the port's readout is the unfiltered list, because the GUI that shows a filtered list is the gap
   below. The rule is recorded with the gap, not invented as a command argument.
   *(Slice 10 carries it: `scan`'s optional query — see
   [Slice 10](#slice-10--the-scans-search-filter-version-0210).)*
3. **The claimed-land check is not carried** (`blocked(no permission query)`): upstream asks
   `level.getBlockState(p).canEntityDestroy(level, p, fakePlayer)` and defaults
   `scannerNoRestrictions` to false. The pin's `can_entity_destroy` is a cancellable **hook** (a
   game event another mod can veto), not a query a mod can call with a fake player, and there is no
   claim-permission method in `host/levels.rs` or `host/block_states.rs`; so the scan reads any
   inventory in the box — which is what an unclaimed build does anyway.
4. **The scanner's own item slots are not declared.** `StorageScannerContainer` has an in, an
   auto-in and an out slot, used by the GUI's insert/request paths and the automation; those belong
   to the list slice, and a container with no consumer would be dead data. The block declares no
   container at all.
5. **The radius is a command, not a slider.** `radius` names the GUI's own control, so the value and
   its bounds are the jar's; only the surface differs.
6. **The loot table carries `copy_name`, not the jar's `copy_components`.** The jar copies
   `mcjtylib:item_energy`, `mcjtylib:item_inventory`, `rftoolsstorage:storage_scanner_data` and
   `rftoolsstorage:crafting_grid_data` on the drop; this port has none of those components (the
   radius and the buffer are the block entity's saved compound, and a broken scanner carries
   nothing), so the drop is the plain item. The modular storage's table is the same shape.
7. **The receipt is charged even when nothing is found**, as upstream charges a request only when it
   delivers: a scan of an empty box still cost 100. That is the price of the readout being the
   request's substitute (deviation 1), not a second rule.

### Red → green (the scanner's rules)

The slice's five new rules were written first and stubbed (`scan_positions` answered an empty list,
`clamp_radius`/`scanner_radius` returned the input, `spend_power` never charged, `merge_counts`
did not merge or sort, `scan_line`/`no_power_line` printed no energy), and the suite was run
**before** implementing them (`ports/_shared/evidence/rftoolsstorage/slice5-red.log`,
`TEST_EXIT:101`):

```
---- runtime::tests::the_scanner_scans_a_cube_of_its_radius_and_nothing_outside_it stdout ----

thread 'runtime::tests::the_scanner_scans_a_cube_of_its_radius_and_nothing_outside_it' panicked at src/runtime.rs:898:9:
assertion `left == right` failed: a 3 x 3 x 3 box at the default radius
  left: 0
 right: 27
...
test result: FAILED. 24 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out
TEST_EXIT:101
```

Implemented, then **29 passed, 0 failed** (`sdk-test.log`): the runtime's 26 (21 + 5 new) and the
three glue tests — `the_scanner_scans_a_cube_of_its_radius_and_nothing_outside_it`,
`the_scanners_radius_is_the_gui_sliders_one_to_twenty`,
`a_scan_costs_the_request_price_and_refuses_below_it`,
`the_scan_counts_each_kind_once_across_the_inventories_it_found`,
`the_scan_reply_names_the_items_and_the_energy_left_or_the_missing_power`.

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built package | `e2e-scanner.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-scanner-control.log` | **FAIL** + `EXIT:1` — `{"error":"unknown item rftoolsstorage:storage_scanner"}` |
| counterfactual (a vanilla barrel where the scanner would be) | `e2e-scanner-counterfactual.log` | **PASS** + `EXIT:0` — `scan`/`radius` answer `minecraft:barrel: not a storage scanner` |
| `SELFTEST=1` (asserts the scan did *not* find the stone) | `e2e-scanner-selftest.log` | **FAIL** + `EXIT:1` — *"the scan found minecraft:stone 64 in the chest two blocks away … so this inverted assertion fails as it must"* |

What `e2e-scanner.log` asserts, in order (every number before and after): the scanner places and
reads back with its `facing`; **a fresh one has no power** — `no power (0/50000 FE)`, nothing read;
charged to 5,000 FE (`/data merge block`), the **default radius 1** scan answers
`empty (4900/50000 FE)` although a chest two blocks away holds 64 stone (the 3 x 3 x 3 box);
`/rftoolsstorage radius … 2` answers `radius 2` and the saved compound reads 2; the scan now answers
**`minecraft:stone 64 (4800/50000 FE)`** and not the `minecraft:dirt 32` of the chest three away
(the counterfactual chest outside the box); radius 3 then lists **both**, one entry a kind, sorted;
**an emptied buffer** answers `no power (0/50000 FE)` and reads nothing, and charging it again makes
the same scan answer again at 4,900; `scan` of a chest answers `not a storage scanner` and so does
`scan` of the modular storage (which still reads `0/0 empty no module`); **a relaunch** keeps the
buffer (4,900) and the radius (3) and the scan still answers; the **loot table** drops
`rftoolsstorage:storage_scanner` ×1 on a survival break; the **recipe is loaded** (`recipe give`
answers `Unlocked 1 recipe(s)`) and its pattern with a stone block where
`rftoolsbase:machine_frame` belongs crafts nothing, while `/give rftoolsbase:machine_frame` is
refused — the frame is RFToolsBase's item and that mod is not in this build.

**The brief asked for a second leg that crafts the scanner's recipe. It cannot be met, and the
artefact is why:** the jar's recipe is `ToT / gFg / ToT` with `F = rftoolsbase:machine_frame`, and
this harness builds a game from the Nightly plus **one** package — the campaign's own `rftoolsbase`
port pins `0b14f8a`, so the launcher refuses to build both into a `0a875da` game. The leg proves the
recipe is loaded and parsed, and that the frame's cell is what blocks it; the coverage row is
`blocked(rftoolsbase:machine_frame)`, the same gap the modular storage's recipe has.

### Coverage (the families this slice changes)

| Family | State |
|---|---|
| `storage_scanner` block, its states/models/loot/tags | **tested** (place + read back with `facing`; survival break drops the item, `e2e-scanner.log`) |
| the scanner's FE buffer (50,000; 100 a scan; saved compound) | **tested** — `no power` at 0, 5,000 → 4,900 → 4,800 → 4,700, refused at 0 and restored, and kept across a relaunch (`e2e-scanner.log`) |
| the scan box and the radius (default 1, 1..20) | **tested** — radius 1 finds nothing two away, 2 finds the stone, 3 adds the dirt; the radius survives a relaunch (`e2e-scanner.log`); the box and the slider in `runtime` tests |
| the scan's count and order (one entry a kind, by id) | **tested in game** (two chests, `minecraft:dirt 32, minecraft:stone 64`) and by `merge_counts` |
| the answer for a block that is not a scanner | **tested** — a chest and the modular storage answer `not a storage scanner`; a barrel in the counterfactual does too |
| commands `scan`/`radius` | **tested** (`e2e-scanner.log`, parsed replies) |
| `storage_scanner` recipe | **blocked**(`rftoolsbase:machine_frame`) — loaded and parsed, its pattern with a stone block in the frame's cell crafts nothing, and the frame item is refused by this build (`e2e-scanner.log`) |
| the GUI's list (search/sort/compact/cycle), the scanner's in/auto/out slots, the remote/tablet variant, XNet settings, the claimed-land check | **not ported** (later slices / `blocked(no permission query)`) |

### Traps this slice paid for

1. **A neighbour's items are its capability, not its saved data.** `BlockEntityRef::data` is a *mod*
   block entity's compound and holds no `Items` (slice 4's trap); the scan must ask
   `level.capability(host, &ITEMS, pos, None)`, which the game answers for a chest, a barrel and a
   mod container alike. A scan built on `data()` would find the storage's own `carried`/`locked`
   keys and nothing else.
2. **The energy is the game's key, not a mod one.** `ModEnergyBuffer` keeps its amount in the saved
   compound's `energy` int, so a mod that reads/writes another key would have two buffers. The e2e
   writes it with vanilla's `/data merge block` (the same path `block_entity_merge_data` documents),
   which is also the only way to charge a machine in a build with no FE generator.
3. **`ctl break` on an empty cell fails.** The loot leg first broke a scanner that had never been
   placed (only its support block was set) and `ctl break` answered an error; place the block, then
   break it.
4. **The crafting table's empty result slot can be listed with no item.** A result reader that
   takes `.item` unconditionally prints `null xnull`; filter the empty entry out (the old script's
   `// "empty"` fallback only fires when the slot is absent).
5. **A reply helper that greps the chat for `ns:path:` also matches the previous answer.** Each
   assertion here pins the whole line (`^<block>: … (NNNN/50000 FE)$`), so a stale line fails
   loudly instead of passing.

## Slice 6 — the scanner's item slots and the inject tick (version 0.2.6)

The gap line slice 5 left, verbatim from [`## Gaps`](#gaps-the-rest-of-the-mod) and its own
deviation 4:

> `storage_scanner` (inventory scan, search, FE use, remote/tablet variant)
> … **The scanner's own item slots are not declared.** `StorageScannerContainer` has an in, an
> auto-in and an out slot, used by the GUI's insert/request paths and the automation; those belong
> to the list slice, and a container with no consumer would be dead data.

This slice gives the slots their consumer, the **tick**, and leaves the GUI list widget and the
request path as the gap. It fits the **machines** tier: a block entity's saved data, a container
and its rule, no renderer and no new vanilla hook.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API 0.2, game 26.3. **Version 0.2.6.**

### What it does

- **The container.** `register_scanner` declares three slots — `rftoolsstorage:storage_scanner`'s
  `SLOT_IN` **0** (the GUI's input), `SLOT_OUT` **1** (where a request puts what it pulled) and
  `SLOT_IN_AUTO` **2** (the automation slot the GUI does not show) — with the jar's own capability
  policy: `GenericItemHandler.create(…).insertable(slot(SLOT_IN_AUTO))`, so automation inserts into
  the auto slot only, and McJtyLib's `extractable` default `(slot, stack) -> true`, so every slot may
  be pulled from. Every face reaches the three; there is no menu (the list widget is the gap).
- **The tick.** `ScannerBlock` gains `block_entity_ticks` (`EntityBlock.getTicker` → true) and
  `block_entity_tick` (`TickingTileEntity.tickServer`): the GUI input first, then the automation
  input. Each non-empty slot is **one** `rfPerInsert` (20 FE) attempt: the stack leaves its slot,
  `injectStackInternal`'s walk pushes it into the scanned inventories in scan order, the remainder
  goes back into the slot, and the price is charged — **also when nothing accepted it**, as upstream
  charges. A buffer that cannot pay for a step returns from the whole tick, so a poor buffer holds
  the automation slot back too instead of borrowing.
- **The walk.** `scan_positions` over the saved radius, minus the scanner's own cell and any other
  storage scanner (upstream's `findInventories` keeps no `StorageScannerTileEntity`), each
  neighbour's `ITEMS` capability asked with no side; each handler takes what it will
  (`insert_any`) and the next one gets the rest while the stack is non-empty. Every neighbour write
  is one transaction: a refused write leaves each inventory as it was and the stack whole, and a
  partial remainder is built *inside* the transaction (an unbuildable remainder aborts the move
  instead of dropping items).
- **What the tick costs.** A scanner with empty input slots does no scan and no capability lookup at
  all; the walk runs only when there is a stack to route. The buffer is the saved compound's
  `energy` (the game's own key), written through the tick's payload.

### Red → green (the rules)

The two new rules were written first and stubbed (`inject_plan` answered an empty plan,
`injectable_at` answered `true`), and the suite was run **before** implementing them
(`ports/_shared/evidence/rftoolsstorage/slice6-red.log`, `TEST_EXIT:101`):

```
thread 'runtime::tests::a_tick_drains_the_gui_slot_first_and_the_automation_slot_second_at_a_price_each' (3089002) panicked at src/runtime.rs:1090:9:
assertion `left == right` failed: the GUI slot is the tick's first step
  left: []
 right: [In]
...
thread 'runtime::tests::the_inject_walk_skips_its_own_cell_and_another_scanner' (3089008) panicked at src/runtime.rs:1132:9:
the scanner never pushes into its own container
...
test result: FAILED. 29 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

Implemented, then **32 passed, 0 failed** (`sdk-test.log`): the runtime's 28 (26 + the tick's plan
and the walk's skip) and the four glue tests, one of them new
(`the_scanners_container_is_the_jars_three_slots`).

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built package | `e2e-inject.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-inject-control.log` | **FAIL** + `EXIT:1` — `{"error":"unknown item rftoolsstorage:storage_scanner"}` |
| counterfactual (a vanilla barrel where the scanner would be) | `e2e-inject-counterfactual.log` | **PASS** + `EXIT:0` — `scan`/`radius` answer `not a storage scanner` |
| `SELFTEST=1` (asserts the tick did *not* move the stone) | `e2e-inject-selftest.log` | **FAIL** + `EXIT:1` — *"the tick moved 64 stone from the in-slot into the chest two blocks away (energy 4980), so this inverted assertion fails as it must"* |

What `e2e-inject.log` asserts, in order (every number before and after; the earlier phases are slice
5's and still pass): the scanner at radius **2** stands two blocks from an **empty** chest (two
adjacent chests would be one double chest, so the chest is alone); 64 stone is merged into the
in-slot (`Items[0]`, which read `Found no elements matching Items` before), and after **10 ticks**
the scanner's in-slot is empty, the **chest's** `Items[0]` reads `64 x minecraft:stone` and the
buffer reads **4980** — `5000 − rfPerInsert × 1`; with the buffer at **0** another 64 stone stays in
the in-slot and the chest keeps its **64**; recharged, the same stack goes and the chest reads
**128** — the stall was the power, not the walk; the **auto slot** (2) routes its 32 stone (chest
**160**, buffer 4960 = −2×20); three stone put in through the game's own container path
(`ctl container`, which the tick sees on its next pass) reach the chest (160 → **163**, buffer
−20); a **full barrel** (27 × 64 stone) refuses, so `energy: 20` pays exactly one attempt and
**0** is left while the dirt stays, and `energy: 19` pays **none** and stays 19; the walk passes
that refusing barrel for the chest behind it (barrel `64..64` unchanged, the chest gets the dirt);
and the **out slot** `Items[1]` is never written (`Found no elements matching Items`).

The hopper leg was designed and then **dropped, because the artefact refuses it**: a hopper inside
the scan box is itself an inventory in the box, so the walk pushes the item *into the hopper* and the
hopper pushes it back into the scanner's auto slot — a loop that cost five attempts per item in the
measurement. That is upstream's behaviour too (a hopper is a scanned inventory like any other); the
container's automation policy is what a hopper exercises and that is pinned by
`the_scanners_container_is_the_jars_three_slots` (`inputs == [SLOT_IN_AUTO]`), not by a scenario that
would fight the machine.

### Deviations this slice adds

1. ~~**Every scanned inventory is routable.**~~ **Closed by slice 12**, which carries the list's
   per-inventory star: the walk skips an inventory the player has not starred, and the set starts
   empty. The slice-6 text is kept for the record: *upstream filters the walk with `isRoutable(p)`,
   the GUI's star set (`GuiStorageScanner`'s "All routable inventories" toggle), which starts empty —
   so an upstream scanner pushes nothing until the player stars an inventory; with no list widget
   the slice-6 port treated every inventory the scan finds as starred, the only reading under which
   the family was observable at all.*
2. **The walk order is the scan's order** (`x`, then `y`, then `z`), not the GUI's reorderable list
   order (up/down/top/bottom are list commands). Without the list there is nothing to reorder.
3. **The GUI's insert/request paths are not carried**, only the tick. `injectStackFromScreen`,
   `requestStack`, `giveToPlayerFromScreen` and the out slot's fill belong to the list slice; the out
   slot is declared (the container must bind) and never written.
4. **No XNet access settings** (`isInputFromAuto`/`isOutputFrom…`), so the auto slot and the GUI slot
   route to the same inventories; and no claimed-land check (`blocked(no permission query)`, slice 5's
   deviation 3, unchanged).

### Coverage (the families this slice changes)

| Family | State |
|---|---|
| the scanner's container: `SLOT_IN` 0, `SLOT_OUT` 1, `SLOT_IN_AUTO` 2 | **tested** — in game (the slots seed and read back, `e2e-inject.log`) and by `the_scanners_container_is_the_jars_three_slots` (inputs/outputs/faces/menu) |
| the inject tick: in-slot → the scanned inventories, one `rfPerInsert` an attempt | **tested** — 64 stone reaches the chest's `Items[0]`, buffer 5000 → 4980 (`e2e-inject.log`); the plan and its early stop in `runtime` tests |
| the automation slot (2) | **tested** — 32 stone routed, buffer −20; automation inserts there only (unit test) |
| the walk: scan order, own cell and another scanner skipped, a refusing inventory passed | **tested** — a full barrel untouched while the chest behind it takes the stack; `injectable_at` in a unit test |
| the price: charged per attempt, even when refused; 20 pays one, 19 none | **tested** — 20 → 0 with the dirt still in the slot, 19 → 19 (`e2e-inject.log`); `inject_plan` |
| the no-power stall and the resume | **tested** — buffer 0 keeps the stack and the chest at 64; recharging moves it (chest 128) |
| the out slot is never written by the tick | **tested** — `Items[1]` reads `Found no elements` after every phase |
| the GUI list (search/sort/compact/cycle), the request path, the remote/tablet variant, XNet settings, the claimed-land check | **not ported** (later slices / `blocked(no permission query)`) |

### Traps this slice paid for

1. **Two adjacent chests are one double chest.** The walk phase first placed its "full" and "accepting"
   chests side by side; the game answered a 54-slot double chest, the angles' counts crossed, and the
   numbers looked like a port bug. Use a **barrel** for the refusing container (barrels never join) or
   leave a gap.
2. **`/data merge block … {Items: […]}` replaces the whole list.** It is the cheapest way to seed a mod
   container's slot (the in-slot included) and to fill a barrel to 27 × 64, but every slot it does not
   name is emptied — so a phase that seeds one slot also clears the others.
3. **A hopper in the scan box is an inject target.** The walk treats it like any inventory; a hopper
   that feeds the scanner makes a loop (measured: five attempts per item). Keep hoppers out of the box,
   or expect the loop — upstream's star list is what would prevent it, and this port has none.
4. **Every `data get` costs ticks.** A read helper with a `ctl wait` after it runs six-odd ticks, so
   "wait 10 and expect ten charges" is wrong: assert tick-independent facts instead — `energy: 20`
   pays exactly one attempt (→ 0), `energy: 19` pays none (→ 19).
5. **A path that does not exist answers `Found no elements matching Items`**, and `ctl messages` lists
   the newest line first, so `jq -r '.chat[0]'` is the reply. A helper that greps the whole message
   list matches the *previous* command's answer (the first probe read the energy reply as the items
   reply).
6. **`ctl container` writes through the game's container path** and the tick routes what it lands
   (160 → 163 stone, −20 FE), which is a free second proof that the container is declared — the slot
   it picks is the engine's (`inputs`/first empty), not a mod decision.
7. **The block entity is out of the world in its own tick.** `host.block_entity(pos)` cannot read it
   there; the tick writes its slots and its buffer through `event.entity.items`/`nbt`, and the
   neighbours through `level.capability`.

## Slice 7 — the scanner's request path (version 0.2.7)

The gap line slice 6 left, verbatim from its deviation 3:

> **The GUI's insert/request paths are not carried**, only the tick.
> `injectStackFromScreen`, `requestStack`, `giveToPlayerFromScreen` and the out
> slot's fill belong to the list slice; the out slot is declared (the container
> must bind) and never written.

The GUI list itself stays a gap (the pin's screen is a fixed slot layout), but the
**request path** is the consumer of the out slot slice 6 declared:
`/rftoolsstorage request <pos> <item> [<count>]` does what
`StorageScannerTileEntity.requestStack` does, priced the way it prices it and
delivering where its default delivers. It fits the **machines** tier: a block
entity's saved buffer, the neighbours' item capability and the container's out
slot — no renderer and no new vanilla hook.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API 0.2, game 26.3. **Version 0.2.7.**

### What it does

- **The price.** `requestStack`: `rf = rfPerRequest` (100), then `if (amount >= 0)
  rf /= 10` — an explicit count costs **10**, a request with no count (upstream's
  `amount == -1`, a whole stack) costs **100**.
- **The amount.** No count asks the item's own stack size (`amount = -1` becomes
  `getMaxStackSize()`); an explicit count is capped at one stack. A count of 0
  pulls nothing and charges nothing (upstream's `todo[0] == finalAmount`).
- **The out slot, first.** Before anything is pulled, a non-empty out slot of
  another item or components refuses the whole request, and so does a full one;
  the same kind leaves its free room (upstream's `return`s, in that order).
- **The walk.** The scanned inventories in scan order, the own cell and another
  scanner skipped as the inject tick skips them; each inventory's matching slots
  are extracted (`ItemResource::matches`, item and components) up to the wanted
  count, in **one transaction**.
- **The charge.** Only when something came out: the buffer is the saved `energy`
  and is written through `merge_data`, as the scan's price is.
- **The delivery.** `requestStraightToInventory` is **true** by default upstream,
  so a player's request goes straight into their inventory (`give_stack`, as
  `/give`, with the vanilla sync) and the out slot is left empty; with no player
  to give to, or a refused give, the pulled stack is written into the out slot
  (the reference's own fallback).
- **The command** keeps the port's reach/operator gate, like `scan` and `radius`.

### Red → green (the rules)

The three new rules were written first and stubbed (`request_price` answered 0,
`request_amount` and `out_slot_room` answered `None`, `request_charge` answered
`NoPower`, the lines `""`), and the suite was run **before** implementing them
(`ports/_shared/evidence/rftoolsstorage/slice7-red.log`, `TEST_EXIT:101`):

```
thread 'runtime::tests::a_request_with_a_count_costs_a_tenth_and_the_default_asks_a_whole_stack' (596079) panicked at src/runtime.rs:1226:9:
assertion `left == right` failed: an explicit count is cheaper
  left: 0
 right: 10
...
thread 'runtime::tests::a_request_refuses_an_out_slot_of_another_kind_or_a_full_one' (596078) panicked at src/runtime.rs:1251:9:
assertion `left == right` failed: an empty out slot has a stack of room
  left: None
 right: Some(64)
...
thread 'runtime::tests::a_request_charges_only_what_it_pulled_and_only_with_the_power_to_pay' (596077) panicked at src/runtime.rs:1275:9:
assertion `left == right` failed
  left: NoPower
 right: Paid(4990)
...
test result: FAILED. 32 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
```

Implemented, then **35 passed, 0 failed** (`sdk-test.log`): the runtime's 31 (28 +
the request's price/amount, the out-slot guard and the charge) and the four glue
tests.

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built package | `e2e-request.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-request-control.log` | **FAIL** + `EXIT:1` — `{"error":"unknown item rftoolsstorage:storage_scanner"}` |
| `SELFTEST=1` (slice 6's inverted assertion) | `e2e-request-selftest.log` | **FAIL** + `EXIT:1` — the tick moved the stone |
| `REQUEST_SELFTEST=1` (this slice's inverted assertion) | `e2e-request-selftest-request.log` | **FAIL** + `EXIT:1` — *"the request pulled 32 stone out of the chest (128 -> 96) and charged 10 FE (5000 -> 4990), so this inverted assertion fails as it must"* |

What `e2e-request.log` asserts, in order (the earlier phases are slices 5–6's and
still pass): the scanner stands at radius **2** with one chest two blocks away
holding **128 stone** (slots 0 and 1, 64 each; the full barrel and the dirt chest
the inject phases left are removed, and two adjacent chests would merge, so the
chest is alone) and the buffer at **5,000**; `request <pos> minecraft:stone 32`
answers `pulled 32 minecraft:stone (4990/50000 FE)`, the **chest reads 96**, the
**buffer 4,990** (an explicit count is 10) and the **player's inventory holds
32 stone** — the reference's straight-to-inventory default. A request for
`minecraft:dirt` (nothing in the box holds it) answers `nothing to pull for
minecraft:dirt (4990/50000 FE)` and leaves the chest and the buffer unchanged. At
buffer **9** the same stone request answers `not enough power for 32
minecraft:stone: costs 10, have 9/50000 FE` — the price line — and pulls nothing.
The same request through `ctl command` (16 stone) also goes straight to the
player (his stone 32 → **48**), leaves the chest at **80** and the buffer at
**4,990**, and the out slot empty, as the reference's default does. The
**counterfactual** seeds the out slot with `minecraft:dirt`: the same stone
request answers `the out slot holds minecraft:dirt; request refused (5000/50000
FE)`, the chest stays at **80** and the buffer at **5,000**.

### Deviations this slice adds

1. **The out slot is a fallback, because the reference's default is.** Upstream's
   `requestStraightToInventory` defaults to true: a player's request goes into
   their inventory and the out slot is cleaned. The port follows that default
   (there is no config), so the out slot is written only when there is no player
   source or the give is refused. The harness's `ctl command` still carries the
   acting player, so the fallback write is not reachable in the leg; the guard
   (another kind or a full out slot) is, and is the counterfactual above.
2. **An explicit count is capped at one stack.** Upstream's `requestStack` is
   handed only 1 (shift) or -1 (default) by the GUI; the command surface has no
   shift, so a larger explicit count is clamped to the item's stack size rather
   than pulling several stacks into one out slot (which upstream's `grow` could
   not represent anyway).
3. **A full inventory throws the overflow** (`give_stack` is `/give`: what does
   not fit is thrown from the player's eyes), where upstream's
   `Inventory.add` returns false and keeps the whole stack in the out slot. The
   pin's give API does not report how much fit.
4. **No config**: `requestStraightToInventory`, `scannerNoRestrictions` and the
   rest are not carried; the port uses their defaults.

### Coverage (the families this slice changes)

| Family | State |
|---|---|
| `/rftoolsstorage request <pos> <item> [<count>]`, its parsed reply | **tested** — `pulled 32 minecraft:stone (4990/50000 FE)`, `nothing to pull`, `not enough power … costs 10`, `the out slot holds …` (`e2e-request.log`) |
| the price: an explicit count 10, a whole stack 100 | **tested** — 10 in game; both and the amount rule in `a_request_with_a_count…` |
| the amount: default = the item's stack size, explicit capped, 0 pulls nothing | **tested** — `a_request_with_a_count…` (unit); the game's 32 is explicit |
| the out slot guard: another kind / a full slot refuses | **tested** — the dirt-seeded counterfactual leaves the chest untouched (`e2e-request.log`); `a_request_refuses_an_out_slot…` |
| the out slot fallback write (no player) | **not exercised** — the harness always has an acting player; `blocked(no console source)`, the reference's default makes it a fallback |
| the scanned-inventory pull (item and components, scan order, own cell skipped) | **tested** — the chest 128 → 96; the pull reuses `injectable_at` |
| charge only when something came out | **tested** — the dirt request leaves the buffer at 4,990; `a_request_charges…` |
| the buffer check before the walk | **tested** — buffer 9 refuses with the price line and pulls nothing; `a_request_charges…` |
| straight to the player (the reference's default) | **tested** — player 32 after the chat request, 48 after the command request |
| the GUI's insert-from-screen, give-from-screen, the list widget, the remote/tablet variant, XNet settings, the claimed-land check | **not ported** (later slices / `blocked(no permission query)`) |

### Traps this slice paid for

1. **`ctl command "/modcmd"` is not a console source.** The first leg expected the
   out slot to receive the 16 stone; the player got it instead (32 → 48) and the
   out slot stayed empty. The harness executes commands for the acting player even
   through `command`, so the reference's no-player fallback cannot be reached that
   way. Assert the default (inventory) instead of the fallback.
2. **`fits` source-scans `uses.json`, and a bare `CommandArgumentValue::Item(..)`
   pattern is read as building the SDK's `Item` type.** `fits --build 0b14f8a`
   refused with *"the type `Item` can no longer be built with a literal, and
   `src/support.rs:1914` builds it"* although the port never touches `Item`. The
   match lives in an `item_argument!` macro now (the `ports/toolbelt` precedent):
   the scan does not expand macros, and both builds fit.
3. **The request phases must keep the scanner's own slots empty.** The tick still
   runs during the `ctl wait`s; with a stack in the in or auto slot it would route
   it into the same chest the request is counting. `seed_slots "$SX" "$SY" "$SZ"
   ''` clears the container before each request phase.
4. **A `data merge block … {Items: […]}` replaces the whole list**, so seeding the
   out slot with dirt (the counterfactual) clears the in/auto slots too — harmless
   here, but it is why the phase reads the out slot after the merge, not before.

## Slice 8 — the scanner's give-from-screen path (version 0.2.8)

The gap line slice 7 left, verbatim from this file's gap list:

> the GUI's **insert-from-screen path** (`injectStackFromScreen`) and its
> **give-from-screen path** (`giveToPlayerFromScreen`, the list's click that hands
> a matching stack straight to the player without the out slot)

Slice 7's coverage table named it the last uncarried request-family method. The GUI list
stays a gap (the pin's screen is a fixed slot layout), but the **give** is a scanner
behaviour with no screen in it: `/rftoolsstorage give <pos> <item> [single]` does what
`StorageScannerTileEntity.giveToPlayerFromScreen` does — the click the storage control
screen module makes on a listed stack — priced the way it prices it, delivered where it
delivers, and **without the out slot** the request path has. It fits the **machines**
tier: a block entity's saved buffer, the neighbours' item capability and
`PlayerRef::give_stack` — no renderer and no new vanilla hook.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API 0.2, game 26.3. **Version 0.2.8.**

### What it does

- **The price is the whole one.** `giveToPlayerFromScreen` checks
  `getStoredPower() < rfPerRequest` first and `consumeEnergy(rfPerRequest)` once after a
  give that took something: **100 FE for one item exactly as for a stack**. The
  tenth-price mode belongs to `requestStack` alone (`request_price`).
- **The amount.** `cnt = {single ? 1 : stack.getMaxStackSize()}`: `single` hands one
  item, no flag hands the item's own stack size (clamped at [`SLOT_LIMIT`] like every
  request).
- **No out slot at all.** No guard, no fallback write: the pulled stack goes straight into
  the player (`give_stack`, as `/give` — what does not fit is thrown from their eyes).
  A source with no player is refused **before** anything leaves an inventory, because
  there is no out slot to keep the stack in.
- **One transaction.** The walk extracts in scan order and the give happens inside the
  same transaction, so a give the game refuses aborts the walk instead of spending a
  chest's stone on nothing.
- **The walk** is the request path's: the scanned inventories in scan order, the own cell
  and another storage scanner skipped (upstream's `findInventories` never lists them),
  item and components matched as `ItemResource`. Upstream's `isItemEqual`
  (`ItemStack.isSameItem`) matches the item id only — see deviation 1.
- **The command** keeps the port's reach/operator gate, like `scan`, `radius` and
  `request`.

### Red → green (the rules)

The two new rules were written first and stubbed (`give_price` answered 0, `give_amount`
0, the lines `""`), and the suite was run **before** implementing them
(`ports/_shared/evidence/rftoolsstorage/slice8-red.log`, `TEST_EXIT:101`):

```
thread 'runtime::tests::a_give_is_the_full_price_whether_it_hands_over_one_item_or_a_stack' (3759699) panicked at src/runtime.rs:1380:9:
assertion `left == right` failed: a give is rfPerRequest
  left: 0
 right: 100
...
thread 'runtime::tests::the_give_reply_names_what_was_given_or_refuses_an_empty_walk' (3759716) panicked at src/runtime.rs:1417:9:
assertion `left == right` failed
  left: ""
 right: "gave 64 minecraft:stone (4900/50000 FE)"
...
test result: FAILED. 35 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

Implemented, then **37 passed, 0 failed** (`sdk-test.log`): the runtime's 33 (31 + the
give's price/amount and its two reply lines) and the five glue tests. The charge reuses
slice 7's `request_charge`, so "paid only when something came out" is unchanged.

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built 0.2.8 package | `e2e-give.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-give-control.log` | **FAIL** + `EXIT:1` — `{"error":"unknown item rftoolsstorage:storage_scanner"}` |
| `GIVE_SELFTEST=1` (this slice's inverted assertion) | `e2e-give-selftest.log` | **FAIL** + `EXIT:1` — *"the give handed 64 stone to the player (chest 64 -> 0, player holds 64) and charged 100 FE (5000 -> 4900), so this inverted assertion fails as it must"* |

`e2e-give.log` re-runs every earlier phase (slices 1–7 still pass), then asserts:

```
before the give: chest 64 stone, energy 5000, player stone 0, out slot 'minecraft:dirt' (Slot 1b)
give stone: rftoolsstorage:storage_scanner: gave 64 minecraft:stone (4900/50000 FE)
after the give: chest 0 stone, energy 4900, player stone 64, out slot 'minecraft:dirt' (Slot 1b)
== a give with single hands one item and drains one ==
give single stone: rftoolsstorage:storage_scanner: gave 1 minecraft:stone (4800/50000 FE)
after the single give: chest 2 stone, energy 4800, player stone 65
== a give for a kind nothing holds refuses and charges nothing ==
give dirt: rftoolsstorage:storage_scanner: nothing to give for minecraft:dirt (4800/50000 FE)
after the refused dirt give: chest 2 stone, energy 4800, player dirt 0
== counterfactual: buffer 99 refuses the give with the price line and leaves the chest alone ==
give stone at 99 FE: rftoolsstorage:storage_scanner: not enough power for 64 minecraft:stone: costs 100, have 99/50000 FE
after the 99-FE refusal: chest 2 stone, energy 99, player stone 65
== a give on a block that is not a scanner is refused before anything is read ==
give on the chest: minecraft:chest: not a storage scanner
```

Two things make the transcript evidence rather than a print: the out slot still holds the
**request counterfactual's dirt** (slot `1b`) after the give — the same slot that refuses
a `request` is invisible to the give, which is the slice's whole point — and the chest's
reading moves 64 → **0** while the player's stone moves 0 → **64** for exactly **100** FE.
The single give then drains exactly one (chest 3 → 2, buffer 4,900 → 4,800, player 64 →
65), a kind nothing holds charges nothing, and the 99-FE leg is the fail-capable price
check (`costs 100`, chest untouched). `GIVE_SELFTEST=1` inverts the first assertion and
the run ends `FAIL:` + `EXIT:1`.

### Deviations this slice adds

1. **The match is item and components, not the item id alone.** Upstream's
   `isItemEqual` is `ItemStack.isSameItem`, so its give pulls *any* variant of the item
   (an enchanted book of any enchantment, a potion of any effect). The port keeps the
   request path's `ItemResource` match, so a click never hands back a different potion or
   enchantment than the one asked for; for the plain stacks the list is built from the two
   agree. A deliberate looseness upstream, not a defect: recorded as a deviation, not
   "fixed".
2. **A give with no player is refused.** Upstream's signature takes a `Player`; a command
   can come from the console. The port refuses that source before pulling, because the
   path has no out slot to keep the stack in — the alternative would destroy items.
   (Slice 7's out-slot fallback stays `blocked(no console source)`; the give has no
   fallback by design.)
3. **One transaction for walk + give.** Upstream extracts and gives slot by slot, so a
   give the player's inventory refuses could lose a stack. The port's give is
   all-or-nothing: a refused give aborts the walk.
4. **The overflow is thrown** (`give_stack` is `/give`: what does not fit is thrown from
   the player's eyes), where upstream's `giveToPlayer` does the same
   (`spawnAtLocation(stack, 1.05f)`) — the one place the two agree exactly.

### Coverage (the families this slice changes)

| Family | State |
|---|---|
| `/rftoolsstorage give <pos> <item> [single]`, its parsed reply | **tested** — `gave 64 minecraft:stone (4900/50000 FE)`, `gave 1 … (4800/…)`, `nothing to give for …`, the price line `costs 100` (`e2e-give.log`) |
| the whole price: one item or a stack costs `rfPerRequest` | **tested** — 100 for the stack and 100 for `single` in game; `a_give_is_the_full_price…` |
| the amount: `single` → 1, no flag → the item's stack size | **tested** — chest 3 → 2 and player 64 → 65; `a_give_is_the_full_price…` |
| no out slot: the guard and the fallback are absent | **tested** — the dirt the request path refuses on is still there after the give, and the give still works (`e2e-give.log`) |
| straight into the player (no out slot write) | **tested** — player 0 → 64, chest 64 → 0, out slot unchanged |
| charge only when something came out | **tested** — the dirt give leaves the buffer at 4,800; the 99-FE give changes nothing |
| the buffer check before the walk, with the price | **tested** — `costs 100, have 99`, chest untouched |
| a non-scanner is refused | **tested** — `minecraft:chest: not a storage scanner` |
| a console source with no player | **not exercised** — the harness always has an acting player; refused by design (deviation 2) |
| the GUI list widget, its search/sort/compact/cycle, `injectStackFromScreen`, the storage control module, the remote/tablet variant, XNet settings, the claimed-land check | **not ported** (later slices / `blocked(no permission query)`) |

### Traps this slice paid for

1. **A `/data merge block … {Items: […]}` writes the list, not the slot.** Seeding one
   entry with `{Slot: 1b, …}` leaves it at **`Items[0]`** carrying `Slot: 1b`; reading
   `Items[1].id` finds nothing and the first run failed with *"the out slot holds '',
   expected the request counterfactual's dirt"* although the request guard could see it.
   Read the list position (and assert `Items[0].Slot` = `1b`), never the slot number as an
   index.
2. **The scanner's own slots must be empty while a give phase counts a chest.** The tick
   runs during every `ctl wait`; a stack in the in/auto slot would be routed into the same
   chest. `seed_slots "$SX" … ''` clears the container before the single-item phase.
3. **The give phase must not clear the out slot before the first give** — that dirt is the
   differential against the request path. It is cleared only after the stack give is
   asserted.
4. **`GIVE_SELFTEST` belongs right after the first assertion**, not at the end: the later
   phases need the state the give produced, and the inverted leg must fail on *this*
   slice's numbers (chest 64 → 0, 100 FE), not on a later phase's.
5. **The package version is what forces the launcher to recompile.** After the bump the
   launcher's `native-build` printed `Compiling rftoolsstorage (1 of 1)` and a new build
   tag; a leg without the bump can run the cached previous package (the slice 8 trap the
   brief named).

## Slice 9 — the scanner's insert-from-screen path (version 0.2.9)

The gap line slice 8 left, verbatim from this file's gap list:

> the GUI's **insert-from-screen path** (`injectStackFromScreen`; its
> give-from-screen path is slice 8, carried as `/rftoolsstorage give`)

It is the last of the scanner's screen-input methods. The GUI list stays a gap (the
pin's screen is a fixed slot layout), but the insert is a scanner behaviour with no
screen in it: `/rftoolsstorage insert <pos> <item> [count]` does what
`StorageScannerTileEntity.injectStackFromScreen` does — the screen control module's
insert button — priced the way it prices it, delivered where it delivers, and
without an out slot at all. It fits the **machines** tier: a block entity's saved
buffer, the neighbours' item capability and the player's inventory slot — no
renderer and no new vanilla hook.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API 0.2, game 26.3. **Version 0.2.9.**

### What it does

- **The price is `rfPerInsert` (20), and only an emptied stack pays it.**
  `injectStackFromScreen` checks `getStoredPower() < rfPerInsert` first, walks the
  stack into the scanned inventories, and calls `consumeEnergy(rfPerInsert)` only
  `if (stack.isEmpty())`. A partial insert is **free**: what went in went in and the
  remainder stays with the player. (That is different from the tick, which charges
  each attempt even when nothing accepted it — slice 6.)
- **The order is the reference's.** Power first (below 20 refuses with the price
  line and nothing is read), then `checkForRoutableInventories` (the box must hold
  at least one inventory; upstream's `findInventories` records a cell only when its
  `ItemHandler.BLOCK` capability answers), then the walk.
- **The amount.** No count hands over the stack the player holds; an explicit count
  is capped at that stack (a count of 0 inserts nothing). `single`/shift do not
  exist here, unlike the give: the screen's insert always hands the stack over.
- **The hand.** The GUI's click is on `player.getItemInHand(MAIN_HAND)`, and its
  caller writes the remainder back into that slot. The command names an item, so it
  takes that item's matching stack from the player (the selected hotbar slot first,
  then the lowest slot), item and components as the request path matches, and writes
  the remainder back into the same slot.
- **One transaction.** The walk's writes and the player's remainder commit
  together: a refused write leaves every inventory and the player's stack as they
  were.
- **No out slot** and no out-slot guard: this path is the player's inventory, not
  the scanner container. A source with no player is refused before anything moves.
- **The command** keeps the port's reach/operator gate, like `scan`, `radius`,
  `request` and `give`.

### Red → green (the rules)

The two new rules were written first and stubbed (`insert_price` answered 0,
`insert_amount` `Some(0)`, `insert_charge` always `NoPower`, the lines `""`), and the
suite was run **before** implementing them
(`ports/_shared/evidence/rftoolsstorage/slice9-red.log`, `TEST_EXIT:101`):

```
thread 'runtime::tests::a_screen_insert_costs_one_rf_per_insert_and_only_when_the_stack_emptied' (475386) panicked at src/runtime.rs:1493:9:
assertion `left == right` failed: an insert is rfPerInsert
  left: 0
 right: 20
...
thread 'runtime::tests::the_insert_reply_names_what_went_in_and_what_did_not' (475396) panicked at src/runtime.rs:1538:9:
assertion `left == right` failed
  left: ""
 right: "inserted 64 minecraft:stone (4980/50000 FE)"
...
test result: FAILED. 37 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

Implemented, then **39 passed, 0 failed** (`sdk-test.log`): the runtime's 35 (33 + the
insert's price/amount/charge and its four reply lines) and the four glue tests.

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built 0.2.9 package | `e2e-insert.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-insert-control.log` | **FAIL** + `EXIT:1` — `{"error":"unknown item rftoolsstorage:storage_scanner"}` |
| `INSERT_SELFTEST=1` (this slice's inverted assertion) | `e2e-insert-selftest.log` | **FAIL** + `EXIT:1` — *"the insert moved 64 stone from the player into the chest (player 64 -> 0, chest 0 -> 64) and charged one rfPerInsert (5000 -> 4980), so this inverted assertion fails as it must"* |

`e2e-insert.log` re-runs every earlier phase (slices 1–8 still pass), then asserts
(the log has every `ctl` reply):

```
== THE INSERT PATH: /rftoolsstorage insert <pos> minecraft:stone pushes the player's stack out, for one rfPerInsert when it emptied ==
before the insert: chest 0 stone, energy 5000, player stone 64
insert stone: rftoolsstorage:storage_scanner: inserted 64 minecraft:stone (4980/50000 FE)
after the insert: chest Items[0] = 64 x minecraft:stone, energy 4980, player stone 0
== an explicit count inserts that many: 10 of the player's 64, one rfPerInsert when they all went in ==
insert 10 stone: rftoolsstorage:storage_scanner: inserted 10 minecraft:stone (4960/50000 FE)
after the 10-stone insert: chest 74 stone, energy 4960, player stone 54
== counterfactual: a barrel with room for 32 takes 32 of the 64 and charges nothing (the stack did not empty) ==
insert stone into a barrel with room for 32: rftoolsstorage:storage_scanner: inserted 32 of 64 minecraft:stone; the rest stays with you, no charge (5000/50000 FE)
after the partial insert: barrel slot 26 = 64 stone, energy 5000, player stone 32
== counterfactual: a full barrel as the only inventory moves nothing, charges nothing and leaves the player's 64 stone ==
insert stone into a full barrel: rftoolsstorage:storage_scanner: inserted 0 of 64 minecraft:stone; the rest stays with you, no charge (5000/50000 FE)
after the full-barrel insert: barrel 64..64 stone, energy 5000, player stone 64
== a box with nothing routable refuses with 'there are no routable inventories' and charges nothing ==
insert stone into a scanner with no inventory in its box: rftoolsstorage:storage_scanner: there are no routable inventories (5000/50000 FE)
after the empty-box refusal: energy 5000, player stone 64
== an insert on a block that is not a scanner is refused before anything is read ==
insert on the chest: minecraft:chest: not a storage scanner
```

Three state transitions make the transcript evidence rather than a print: the
**chest's `Items[0]`** moves `none -> 64 x minecraft:stone` while the **player's**
stone moves `64 -> 0` for exactly **20** FE (one `rfPerInsert`); the partial barrel
takes **32 of 64** (its last slot `32 -> 64`) and the buffer does **not** move —
the free-on-partial rule, which is what distinguishes this path from the tick; and
the full barrel and the empty box both answer without a charge and leave the
player's 64 stone where it was. `INSERT_SELFTEST=1` inverts the first assertion and
the run ends `FAIL:` + `EXIT:1`.

### Deviations this slice adds

1. **The count is a command argument.** Upstream's click hands over the whole stack
   in the hand; the command takes an item (and optionally a count) from the
   player's inventory. The charge reads "the portion handed over emptied": with no
   count that is the whole stack (upstream's rule exactly); with a count it is the
   `n` items asked for, which is the only reading under which the count form has a
   price. The amount is capped at the stack the player actually holds.
2. **The stack is looked up, not read from the main hand.** The GUI's click always
   uses `getItemInHand(MAIN_HAND)`; a command names an item, so the port takes that
   item's matching stack (the selected hotbar slot first, then the lowest), which is
   the same stack whenever the player holds the item in hand. Item and components
   match, as the request path does (`ItemResource`); upstream's insert has no matcher
   at all — it inserts the very stack it was handed.
3. ~~**Every scanned inventory is routable**~~ — slice 6's deviation 1, closed by slice 12:
   `checkForRoutableInventories` now means "the box holds a **starred** inventory".
4. **A console source is refused** before anything is pulled: the path hands a
   player's stack over, and a console has no inventory. (The request path's out-slot
   fallback stays `blocked(no console source)`; the insert has no fallback by
   design.)

### Coverage (the families this slice changes)

| Family | State |
|---|---|
| `/rftoolsstorage insert <pos> <item> [count]`, its parsed reply | **tested** — `inserted 64 … (4980/…)`, `inserted 10 … (4960/…)`, `inserted 32 of 64 … no charge`, `inserted 0 of 64 …`, `there are no routable inventories`, `nothing to insert` (unit), `not a storage scanner` (`e2e-insert.log`) |
| the price: one `rfPerInsert`, charged only when the stack emptied | **tested** — 5,000 → 4,980 for a full stack, 5,000 unchanged for 32-of-64 and 0-of-64; `a_screen_insert_costs…` |
| the amount: the held stack, an explicit count capped at it, 0 inserts nothing | **tested** — 64 and 10 in game; the cap and the 0 in `a_screen_insert_costs…` |
| power checked first | **tested** by `insert_charge` (19 → `NoPower`, 20 → `Paid(0)`); the command refuses with the price line before reading (the readout's order) |
| `checkForRoutableInventories` | **tested** — a box with only the scanner answers `there are no routable inventories` and charges nothing (`e2e-insert.log`) |
| the walk (scan order, own cell and another scanner skipped) | **tested by reuse** — the insert walk is the inject walk's (`scan_targets`); the game leg pushes into the chest two away and not into the scanner |
| the player's stack leaves the slot; the remainder stays | **tested** — player 64 → 0 after a full insert, 64 → 54 after 10, and 64 unchanged against a full barrel (`e2e-insert.log`) |
| one transaction (walk + the player's write) | unit-level by construction (`host.transaction`); not forced to abort in game |
| a console source with no player | **not exercised** — the harness always has an acting player; refused by design (deviation 4) |
| the GUI list widget and its controls, the storage control module, the remote/tablet variant, XNet settings, the claimed-land check | **not ported** (the list is the gap; `blocked(no permission query)`) |

### Traps this slice paid for

1. **The harness's `zero` is a paid reply too.** `inserted 0 of 64` is the full
   barrel's answer, and a grep for "inserted" alone would pass on it — every
   assertion here pins the whole line (count, wanted, item and the buffer).
2. **The partial case is the one that proves the rule.** A full barrel proves "a
   refused insert charges nothing", but only a barrel with **room for part** of the
   stack proves that a partial insert is free; seed 26 full slots and one with 32
   (`nearly_full_slots`), and the last slot's `32 -> 64` is the assertion.
3. **`/data merge block … {Items: […]}` replaces the whole list**, so the nearly-full
   and full barrels are each seeded from scratch, not grown from the previous phase.
4. **The scanner's own slots must be empty while the insert phase counts the
   chest** — the tick runs during every `ctl wait` and would route an in-slot stack
   into the same chest. `seed_slots "$SX" … ''` clears the container (the give
   phase's out-slot dirt with it) before the first insert.
5. **`INSERT_SELFTEST` belongs right after the first assertion**, on this slice's
   numbers (player 64 → 0, chest 0 → 64, 5,000 → 4,980), not at the end where a later
   phase's state would mask it.
6. **`launcher-game.sh env` exports `CONTROL_GAME`.** A default leg started from that
   env runs the *control* branch and fails with "there is no … storage_scanner to
   place" — run the default with `CONTROL_GAME=` (that log **is** the control leg;
   keep it as `e2e-insert-control.log`).

## Slice 10 — the scan's search filter (version 0.2.10)

The gap line slice 9 left, verbatim from this file's gap list:

> the GUI list widget's **sort, compact, cycle and visible-size controls** (its **search** is
> slice 10, carried as `scan`'s optional query)

and slice 5's deviation 2, which this slice closes:

> **`search` is not carried.** Upstream's `getMatcher` filters the list by item name or `@modid`;
> the port's readout is the unfiltered list, because the GUI that shows a filtered list is the gap
> below.

The slice is the **search half of the scanner's list widget**, carried as the readout's optional
term: `/rftoolsstorage scan <pos> [query]`. It fits the **machines** tier: the same block entity's
saved buffer and radius, the neighbours' item capability and the command the scan already is — no
renderer, no screen and no new vanilla hook. Upstream source read at
`fc1b18a7f070d0030671e1db1d3aab90699fd160`
(`StorageScannerTileEntity.getMatcher`/`makeSearchPredicate`, lines 475–499; the raw files were
read into `/tmp/opencode` and deleted after the findings were recorded).

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API 0.2, game 26.3. **Version 0.2.10.**

### What it does

- **The rule is `getMatcher`'s.** The query is lower-cased and split on whitespace; **every** word
  must match (upstream builds `matcher.and(...)`), and an empty query matches every kind
  (`if (matcher == null) matcher = s -> true`).
- **A word that starts with `@` is the namespace.** Upstream tests
  `Tools.getModid(s).toLowerCase().startsWith(split.substring(1))` — a **prefix**, not a contains, so
  `@min` matches `minecraft` and `@craft` does not.
- **Any other word is a contains on the item's name**, case-insensitively
  (`s.getHoverName().getString().toLowerCase().contains(split)`), so `dirt` matches *Dirt* and
  `ak lo` matches *Oak Log*.
- **The filter narrows the readout before the count is merged**, one entry a kind as before: a kind
  the query rejects is not listed, and when nothing matches the reply is `empty`.
- **The price does not change.** A filtered scan costs the same `rfPerRequest` (100 FE), charged
  before the walk and even when nothing matches: the readout *is* the list a request is made from
  (slice 5's deviation 1), and upstream's search narrows that same list.

### Red → green (the rule)

The three new rules (`matches_search`, `search_namespace`, `search_name`) and their four tests were
written first with the functions stubbed (the matcher `false`, the namespace `""`, the name `""`),
and the suite was run **before** implementing them
(`ports/_shared/evidence/rftoolsstorage/slice10-red.log`, `TEST_EXIT:101`):

```
thread 'runtime::tests::a_search_word_matches_the_name_case_insensitively' (37135) panicked at src/runtime.rs:1600:9:
the hover name is compared lower-cased
...
thread 'runtime::tests::the_search_name_comes_from_the_item_ids_path' (37153) panicked at src/runtime.rs:1664:9:
assertion `left == right` failed: the path's underscores are spaces
  left: ""
 right: "oak log"
...
test result: FAILED. 39 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
...
TEST_EXIT:101
```

Implemented, then **43 passed, 0 failed** (`sdk-test.log`): the runtime's 39 (35 + the four search
rules) and the four glue tests.

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built 0.2.10 package | `e2e-scanfilter.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-scanfilter-control.log` | **FAIL** + `EXIT:1` — `{"error":"unknown item rftoolsstorage:storage_scanner"}` |
| `SCAN_SELFTEST=1` (this slice's inverted assertion) | `e2e-scanfilter-selftest.log` | **FAIL** + `EXIT:1` — *"the 'dirt' query listed only minecraft:dirt 32 and not the stone 64 two blocks away …, so this inverted assertion fails as it must"* |

`e2e-scanfilter.log` re-runs every earlier phase (slices 1–9 still pass), then asserts the brief's
rig — scanner at S, radius 3, a chest at S+2 holding 64 stone, one at S+3 holding 32 dirt, the
buffer set back to 5,000 — with every `ctl` reply in the log:

```
== THE SCAN FILTER: the GUI list's search box — every word is '@modid' (a namespace prefix) or a case-insensitive name contains ==
scan @minecraft: rftoolsstorage:storage_scanner: minecraft:dirt 32, minecraft:stone 64 (4900/50000 FE); buffer now 4900
scan dirt: rftoolsstorage:storage_scanner: minecraft:dirt 32 (4800/50000 FE); buffer now 4800
scan stone: rftoolsstorage:storage_scanner: minecraft:stone 64 (4700/50000 FE); buffer now 4700
scan @nomatch: rftoolsstorage:storage_scanner: empty (4600/50000 FE); buffer now 4600
== the unfiltered scan still lists both — the differential for the filter ==
scan (no query): rftoolsstorage:storage_scanner: minecraft:dirt 32, minecraft:stone 64 (4500/50000 FE); buffer now 4500
```

Three facts make that more than a print: `@minecraft` lists **both** kinds while `dirt` and `stone`
each list **one** (the contains works on both names, and the namespace prefix keeps both); the buffer
walks **5,000 → 4,900 → 4,800 → 4,700 → 4,600 → 4,500**, so a filtered scan and the `empty` answer
each cost one `rfPerRequest`; and the unfiltered scan right after them lists both again — the
**differential** that proves the filter, not the rig, removed the stone from `dirt`. `SCAN_SELFTEST=1`
inverts the `dirt` assertion and the run ends `FAIL:` + `EXIT:1`.

### Deviations this slice adds

1. **The name is the item id's path, not `getHoverName()`.** Upstream compares the English hover name
   on the server (`Component.getString()` resolves the server's language map). The SDK exposes the
   hover name only on the client (`ItemsClientHost::hover_name` is `ClientSetup`; `Host` has no
   item-name read and `translate` is client-only), so the port compares the id's path with its
   underscores as spaces (`minecraft:oak_log` → `oak log`). That is identical for every item whose
   lang key is its path — all vanilla items and the modded items this build carries — and a mod item
   whose image is a hand-written name (`rftoolsstorage:storage_module0` → *Storage Module (T1)*) is
   found by its id's words rather than its display words.
2. **The query is a command term, not a text field.** Upstream's search is the GUI's `TextField`;
   the port has no screen for the list (still the gap), so the same rule is `scan`'s optional
   argument. The command word is a **greedy `text`**: Brigadier's `word` excludes `@`, so a `word`
   argument could not carry `@modid` at all (trap 1).

### Coverage (the families this slice changes)

| Family | State |
|---|---|
| `/rftoolsstorage scan <pos> [query]`, its parsed reply | **tested** — `@minecraft` → both kinds, `dirt` → `minecraft:dirt 32`, `stone` → `minecraft:stone 64`, `@nomatch` → `empty`, each at its own buffer (`e2e-scanfilter.log`) |
| each word a case-insensitive contains on the name | **tested** — `dirt`/`stone` in game; `dirt`/`STONE`/`ak lo` and the negative in `a_search_word_matches_the_name_case_insensitively` |
| `@modid` a namespace **prefix** | **tested** — `@minecraft` in game; `@mine` true, `@craft` false, `@nomatch` false in `a_search_that_starts_with_at_matches_the_namespace_by_prefix` |
| every word ANDed; empty query matches all | **tested** — `@minecraft stone` true, `@minecraft dirt` false in `every_search_word_must_match_and_an_empty_search_matches_everything`; the in-game no-query differential |
| the price: one `rfPerRequest`, also for `empty` | **tested** — 4,900 → 4,600 → 4,500 in game (every query costs 100) |
| the name from the id path; the namespace | **tested** — `the_search_name_comes_from_the_item_ids_path` |
| the GUI's sort, compact, cycle and visible-size controls, the remote/tablet variant, XNet settings, the claimed-land check | **not ported** (the rest of the list; `blocked(no permission query)` for the claim) |

### Traps this slice paid for

1. **`@` is not in a Brigadier `word`.** The first usage sketch was `[query:word]`, and its character
   set (letters, digits, `_ - . +`) would have made `@minecraft` unparseable — the command would have
   refused the very assertion this slice rests on. The argument is the greedy `text` kind, which is
   the usage's last argument.
2. **Charge before the filter.** The price is spent before the walk (slice 5's order), so `@nomatch`
   costs 100 and answers `empty (…/50000 FE)`. A filter applied before `spend_power` would have made
   an unmatched scan free, which upstream's request path never does.
3. **The e2e's rig is the radius-3 one.** The brief's chests (S+2 stone 64, S+3 dirt 32) are the
   slice-5 rig; the filter phase reuses them at radius 3 rather than seeding new chests, and sets the
   buffer back to 5,000 first so its 4,900… is exact. Two adjacent chests are one double chest, so
   S+2 and S+3 stay two cells apart in a row.
4. **The unfiltered scan is the differential, not a cost.** Without it a filter that dropped
   *everything* would pass every per-query assertion; the last scan lists both kinds and proves the
   chests still hold them.
5. **`launcher-game.sh env` exports `CONTROL_GAME`** (slice 9's trap 6 still applies): the default
   leg runs with `CONTROL_GAME=` and the control leg with the export.

## Slice 11 — the scanner list's sort choice (version 0.2.11)

The gap line slice 10 left, verbatim from this file's gap list:

> the GUI list widget's **sort, compact, cycle and visible-size controls** (its **search** is
> slice 10, carried as `scan`'s optional query)

Slice 11 is the **sort** half, carried as the readout's saved choice:
`/rftoolsstorage sort <pos> <mode>` sets the GUI list's order and `scan` applies it. It fits the
**machines** tier: the same block entity's saved compound, the neighbours' item capability and the
readout the scan already is — no renderer, no screen, no new vanilla hook. Upstream read at
`fc1b18a7f070d0030671e1db1d3aab90699fd160`
(`modules/scanner/tools/SortingMode.java`, `modules/scanner/data/StorageScannerData.java`,
`modules/scanner/client/GuiStorageScanner.java` `updateContentsList`/`compareByMod`/`compareByTag`/
`findLargestTag`, `modules/scanner/blocks/StorageScannerTileEntity.java` `getSortMode`/`setSortMode`;
the raw files were read into `/tmp/opencode` and deleted after the findings were recorded).

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API 0.2, game 26.3. **Version 0.2.11.**

### The artefact's rules (read, not invented)

`SortingMode` is `AMOUNT_ASCENDING("Amount+")`, `AMOUNT_DESCENDING("Amount-")`, `MOD("Mod")`,
`TAG("Tag")`, `NAME("Name")`; `StorageScannerData.DEFAULT.sortingMode` is **`NAME`**, the order
every earlier slice printed. `GuiStorageScanner.updateContentsList`:

| mode | comparator | tie-break |
|---|---|---|
| `AMOUNT_ASCENDING` | `Comparator.comparing(ItemStack::getCount)` | none — `List.sort` is **stable**, so equal counts keep the server list's order |
| `AMOUNT_DESCENDING` | `Comparator.comparing(ItemStack::getCount).reversed()` | none — stable, same |
| `MOD` | `Tools.getId(s1).getNamespace().compareTo(…)` | then `getHoverName().getString().compareTo(…)` (raw, case-sensitive) |
| `TAG` | `findLargestTag(s1).compareTo(findLargestTag(s2))` | then the hover name (raw) |
| `NAME` | `Comparator.comparing(itemStack -> itemStack.getHoverName().getString())` | n/a |

The port reads the display name server-side as the item id (slice 10's deviation 1, the hover name
is client-only), so `Name` is the item id's order, `Mod` is the namespace then the id (the tie
upstream breaks on the hover name), and the amount sorts are stable over `merge_counts`' first-seen
order — the walk's order, the server list's analogue. `merge_counts` therefore no longer sorts by
id; it keeps the order the walk met each kind, and `sort_found` applies the mode (the default
`Name` still prints the id order, so every earlier leg is unchanged).

`TAG` is **`blocked(no item-tag enumeration)`**: upstream sorts by the "largest" tag an item is in
(`findLargestTag`), which needs the set of tags an item belongs to. The SDK's `Tags` service
(`crates/horizon-mod-vocab/src/world/tags.rs`) answers only `key`, `is_in_tag(item, TagKey)` and a
tag's members (`items`/`fluids`/`blocks`), and `surface.json` confirms there is no `Tags::names` or
item→tags read — forward membership only. The command refuses `Tag` with a line naming the gap, and
a saved `TAG` (only another build could write one) reads back as `NAME`.

The saved key is upstream's `sortMode` (`Value.createEnum("sortMode", …)`); the value is the enum's
serialized name (`AMOUNT_DESCENDING`), as `StorageScannerData.CODEC` writes it.

### Red → green (the rules)

The three sort rules were written first with the functions stubbed (`description`/`serialized_name`
returned `""`, `from_description` `None`, `from_saved` `Name`, `available` `false`, `sort_found`
returned its input), and the suite was run **before** implementing them
(`ports/_shared/evidence/rftoolsstorage/slice11-red.log`, `TEST_EXIT:101`):

```
thread 'runtime::tests::the_list_modes_are_the_gui_s_five_choices_and_default_to_name' (55349) panicked at src/runtime.rs:1770:9:
assertion `left == right` failed
  left: ""
 right: "Amount+"
...
thread 'runtime::tests::amount_sorting_is_stable_so_equal_counts_keep_the_walk_order' (55340) panicked at src/runtime.rs:1895:9:
assertion `left == right` failed: the ascending direction is stable too
  left: ["minecraft:dirt", "minecraft:sand", "minecraft:stone"]
 right: ["minecraft:stone", "minecraft:dirt", "minecraft:sand"]
...
thread 'runtime::tests::the_scan_counts_each_kind_once_across_the_inventories_it_found' (55352) panicked at src/runtime.rs:1351:9:
assertion `left == right` failed: the walk's first-seen order is the base the amount sort's stability keeps
  left: "minecraft:dirt"
 right: "minecraft:stone"
...
test result: FAILED. 42 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
TEST_EXIT:101
```

Implemented, then **46 passed, 0 failed** (`sdk-test.log`): the runtime's 42 (39 + the three sort
rules) and the four glue tests.

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built 0.2.11 package | `e2e-sort.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-sort-control.log` | **FAIL** + `EXIT:1` — `{"error":"unknown item rftoolsstorage:storage_scanner"}` |
| `SORT_SELFTEST=1` (the `Amount-` reply must not start with stone) | `e2e-sort-selftest.log` | **FAIL** + `EXIT:1` — *"the Amount- scan started with minecraft:dirt 64 and not the stone 32 …"* |

`e2e-sort.log` re-runs every earlier phase (slices 1–10 still pass), then asserts the brief's rig —
scanner at S, radius 3, chest S+2 = **32 stone**, chest S+3 = **64 dirt**, buffer 5,000 — with every
`ctl` reply in the log:

```
== THE SORT CHOICE: the GUI list's sort — Amount+, Amount-, Mod, Name (Tag is blocked at this pin) ==
sort Amount-: rftoolsstorage:storage_scanner: sort Amount-
saved sortMode: AMOUNT_DESCENDING
scan Amount-: rftoolsstorage:storage_scanner: minecraft:dirt 64, minecraft:stone 32 (4900/50000 FE); buffer now 4900
sort Amount+: rftoolsstorage:storage_scanner: sort Amount+
scan Amount+: rftoolsstorage:storage_scanner: minecraft:stone 32, minecraft:dirt 64 (4800/50000 FE)
sort Name: rftoolsstorage:storage_scanner: sort Name
scan Name: rftoolsstorage:storage_scanner: minecraft:dirt 64, minecraft:stone 32 (4700/50000 FE)
== the Mod order puts a rftoolsstorage stack after both minecraft kinds ==
sort Mod: rftoolsstorage:storage_scanner: sort Mod
scan Mod: rftoolsstorage:storage_scanner: minecraft:dirt 64, minecraft:stone 32, rftoolsstorage:storage_module0 1 (4600/50000 FE)
```

Three facts make that evidence rather than a print: the same two chests answer **dirt-first** under
`Amount-` and **stone-first** under `Amount+`, so the order is the mode, not the rig; `Name` answers
the dirt first again — the differential against `Amount+` — while the id order (`dirt` < `stone`)
agrees with `Amount-` on this rig by coincidence, which is why `Amount+` is the leg that moves; and
the third chest's `rftoolsstorage:storage_module0` lands **after both** `minecraft:` kinds under
`Mod` (`minecraft` < `rftoolsstorage`). The saved compound reads `AMOUNT_DESCENDING` after the
command, and the **relaunch** leg sets `Amount-` on the search phase's restored rig, quits, re-enters
and reads `minecraft:stone 64, minecraft:dirt 32` back with `sortMode` still `AMOUNT_DESCENDING`
(`relaunch: buffer 4800 -> 4800, radius 3, sort AMOUNT_DESCENDING; scan: … minecraft:stone 64,
minecraft:dirt 32 (4700/50000 FE)`). `SORT_SELFTEST=1` inverts the first `Amount-` assertion and the
run ends `FAIL:` + `EXIT:1`.

### Deviations this slice adds

1. **`TAG` is refused** (`blocked(no item-tag enumeration)`, above): the other four modes are carried.
2. **The tie-break is the item id, not the hover name**, for `Name` and for `Mod`'s same-namespace
   tie: upstream compares `getHoverName().getString()`, which the SDK exposes only on the client
   (slice 10's deviation 1). For every vanilla item and every carried mod item whose lang key is its
   path the two agree; a hand-named mod item sorts by its id's words.
3. **The amount sort's base order is the scan walk's** (`x`, then `y`, then `z`, then slot): upstream
   sorts the server's per-inventory list stably, which this port's merged readout replaces. Equal
   counts keep that first-seen order, which is the artefact's own tie rule (a stable sort with no
   secondary key), not a new one.

### Coverage (the families this slice changes)

| Family | State |
|---|---|
| `/rftoolsstorage sort <pos> <mode>`, its parsed reply | **tested** — `sort Amount-`, `sort Amount+`, `sort Name`, `sort Mod` (`e2e-sort.log`) |
| the four carried orders | **tested** — dirt-first/stone-first/dirt-first/module-last in game; the unit rules in `the_scan_list_sorts_by_the_chosen_mode` |
| the saved choice (`sortMode`, serialized enum) | **tested** — `AMOUNT_DESCENDING` read after the command and after a relaunch, with the same order (`e2e-sort.log`) |
| the amount tie-break (stable) | **tested** — `amount_sorting_is_stable_so_equal_counts_keep_the_walk_order`; the in-game rig has no tie |
| `Mod` groups the namespace then the id | **tested** — the module after both `minecraft:` kinds; the unit rule |
| `Name` is the default and the id order | **tested** — the default scans of every earlier leg, plus `Name` in this one; the unit rule |
| `Tag` | **blocked(no item-tag enumeration)** — the command refuses it with a line naming the gap |
| the GUI list widget itself, compact/cycle/visible-size, the remote/tablet variant, XNet settings, the claimed-land check | **not ported** (the screen is a fixed slot layout; `blocked(no permission query)` for the claim) |

### Traps this slice paid for

1. **The brief's rig is not the search phase's rig.** The search phase's chests are 64 stone / 32
   dirt; the sort assertion needs 32 stone / 64 dirt, so the phase re-seeds both, then restores them
   (and `Name`) so the relaunch, inject, request, insert and give legs keep their numbers. Seeding a
   chest with `setblock` + `item replace` is the clean way; `data merge` would replace the whole list.
2. **The module chest must not be adjacent.** Two adjacent chests join (slice 6's trap); the module
   chest goes two blocks the **other** way on x and is removed after the `Mod` phase, so no later
   scan or inject walk sees it.
3. **`sort` is a `word`, and `+`/`-` are in Brigadier's word set.** `Amount+` and `Amount-` parse;
   a `text` would be greedier than a single word and swallow the next argument if one is added later.
4. **The saved key is a string.** `data get block … sortMode` prints the quoted enum name; the port
   stores `AMOUNT_DESCENDING`, not the label `Amount-`, so the command's vocabulary and the saved
   value stay separate (the label is the GUI's, the serialized name is the codec's).
5. **The version bump is what forces the launcher to recompile.** With `0.2.11` the build printed
   `Compiling rftoolsstorage (1 of 1)` and a new build tag; without it the cached 0.2.10 package runs.

## Slice 12 — the scanner's routable star list (version 0.2.12)

The gap line slice 11 left, verbatim from [`## Gaps`](#gaps-the-rest-of-the-mod) and slice 6's
deviation 1:

> the XNet access settings and its `routable` star list (the port treats every scanned
> inventory as routable — slice 6's deviation 1)

The slice is the **star half**: the GUI's per-inventory "Routable: Yes/No" choice
(`GuiStorageScanner.addStorageLine`, under the list's "All routable inventories" label), carried as
`/rftoolsstorage routable <pos> <target> <on|off>`. It fits the **machines** tier: one more key in
the scanner's saved compound, the same routing walks and the same item capability — no renderer, no
screen, no new vanilla hook. The XNet **access settings** (`InventoryAccessSettings`: per-inventory
input/output permissions per path) stay the gap: they are a different feature, fed by XNet, and no
part of the star list. Upstream read at `fc1b18a7f070d0030671e1db1d3aab90699fd160`
(`StorageScannerTileEntity.java`, `GuiStorageScanner.java`, `InventoryAccessSettings.java`) and
McJtyLib `ef393e60` `BlockPosTools.java`; the raw files were read into `/tmp/opencode` and deleted
after the findings were recorded.

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API 0.2, game 26.3. **Version 0.2.12.**

### The artefact's rules (read, not guessed)

- **The default is an empty set.** `private final Set<BlockPos> routable = new HashSet<>();` — a
  field, not part of `StorageScannerData` (which holds `radius`, `exportC`, `wideview` and
  `sortingMode` only). Nothing seeds it, so **a freshly scanned inventory is not routable**: an
  upstream scanner pushes nothing, requests nothing and refuses an insert with *"There are no
  routable inventories!"* until the player stars an inventory. The port starts the same way; a
  block saved with no `routable` key reads as the empty set.
- **The saved shape is a list of int-array compounds.** `saveAdditional` writes
  `tag.put("routable", list)` where each entry is `BlockPosTools.write(c)`:
  `{c: [I; x, y, z]}`. `loadAdditional` reads the same list, and `BlockPosTools.read` returns null
  for an empty array, so an unreadable entry is dropped; a `Set` also holds a position once. The
  port writes and reads exactly that (in game: `routable` reads back as
  `[{c: [I; 12, 102, 3]}]`, and the key is simply absent before the first star).
- **The star filters routing, not the list.** `injectStackInternal` (the tick and the screen insert),
  `requestItem` (the request/give pulls) and `checkForRoutableInventories` all test
  `isRoutable(p)`; `getInventoryForBlock`/`countItems(..., starred=false)` do **not** — the GUI's
  list shows every inventory in the box regardless of its star. The port's `scan_targets` (the one
  walk every routing path uses) filters by the set, and `scan_found` (the readout) does not.
- **The GUI's toggle is a set membership**, one inventory at a time ("Routable: Yes/No"), and
  `toggleRoutable` calls `setChanged()`. The port's `on`/`off` are idempotent and save the set with
  the block, so it survives a relaunch (`block_entity` saved compound).
- **A stale star is inert.** The set keeps a position whose inventory is gone; the walk tests the
  capability at the cell, so nothing happens until an inventory is back — `checkForRoutableInventories`
  too (upstream's `level.getBlockEntity(p) != null`).

### What the slice does

- **`runtime`** (pure, tested): [`is_routable`] (the set's `contains`), [`star`] (add/remove one cell,
  idempotent, other cells kept), [`routing_target`] (`injectable_at` **and** starred — upstream's
  `!p.equals(getBlockPos()) && isRoutable(p)`), [`routable_line`] (the reply).
- **`support`**: `KEY_ROUTABLE = "routable"`, `saved_routable` (the list → a de-duplicated set, the
  malformed entries dropped), `routable_tag` (the set → the same list), the command
  `routable <pos> <target> <on|off>` (the target must be a cell the GUI's list could show: in the
  scan box, with an item capability, never the scanner itself), and the `routable: &[[i32; 3]]`
  parameter on `scan_targets`, `pull_request`, `pull_to_player`, `insert_to_inventories`,
  `has_routable` and `inject_stack`.
- **Unchanged**: the readout (`scan`) lists every inventory in the box, the scan's price, the sort,
  the inject tick's per-attempt `rfPerInsert` (paid even when nothing is routable, as upstream's
  tick pays it), the request/give prices and the insert's pay-only-when-emptied rule.
- **The earlier legs keep their behaviour by starring their chests**: the inject/request/give/insert
  phases now call `routable … on` for the chest (and the refusing barrel) they route into, which is
  what a player does in the GUI. The star is a real behaviour change from 0.2.11 — a scanner with
  nothing starred routes nothing — and the e2e proves both sides.

### Red → green (the rules)

The three rules were written first with `is_routable`/`star`/`routable_line` stubbed
(`false`/`Vec::new()`/`""`) and `routing_target` ignoring the star, and the suite was run
**before** implementing them (`ports/_shared/evidence/rftoolsstorage/slice12-red.log`,
`TEST_EXIT:101`):

```
---- runtime::tests::a_fresh_scanners_star_list_is_empty_and_a_star_switches_one_cell stdout ----

thread 'runtime::tests::a_fresh_scanners_star_list_is_empty_and_a_star_switches_one_cell' (85089) panicked at src/runtime.rs:1979:9:
assertion `left == right` failed: a press adds the cell
  left: []
 right: [[8, 102, 3]]
...
thread 'runtime::tests::the_walk_skips_a_cell_that_is_not_starred' (85130) panicked at src/runtime.rs:2009:9:
an unstarred inventory is not routable even though the scan lists it
...
thread 'runtime::tests::the_star_reply_names_the_cell_and_its_state' (85127) panicked at src/runtime.rs:2025:9:
assertion `left == right` failed
  left: ""
 right: "routable 8 102 3 on"
...
test result: FAILED. 46 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
TEST_EXIT:101
```

Implemented, then **50 passed, 0 failed** (`sdk-test.log`, `slice12-test.log`): the runtime's 45
(42 + the three star rules) and the five glue tests, one of them new
(`the_routable_set_is_upstreams_list_of_int_array_compounds` — the saved shape and its malformed
entries). `uses.json` grew from 537 to **540 API items** (the `BlockPos` argument's pattern, the
`Tag::IntArray` and one more method).

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built 0.2.12 package | `e2e-routable.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-routable-control.log` | **FAIL** + `EXIT:1` — `{"error":"unknown item rftoolsstorage:storage_scanner"}` |
| `ROUTABLE_SELFTEST=1` (the un-starred chest must *not* have taken the stack) | `e2e-routable-selftest.log` | **FAIL** + `EXIT:1` — *"the un-starred chest A stayed empty (0) and the starred B took the 64 stone (energy 4980), so the star filtered routing; this inverted assertion fails as it must"* |
| the first reaching run (kept: it found the tick-charge trap) | `e2e-routable-first-run.log` | **FAIL** + `EXIT:1` — *"the unstarred attempt left 0, expected one rfPerInsert off 5000 (4980)"* |

`e2e-routable.log` re-runs every earlier phase (slices 1–11 still pass, with the routing phases
starring their chests), then asserts the brief's rig — scanner at S, radius 2, two empty chests two
blocks away (A at S+2 on **x**, B at S+2 on **z**), sort `Name`, buffer 20 for the first attempt —
with every `ctl` reply in the log:

```
saved routable (default): Found no elements matching routable
no star: in-slot 64, A 0, B 0, energy 0
star B: rftoolsstorage:storage_scanner: routable 12 102 3 on
star A: rftoolsstorage:storage_scanner: routable 14 102 1 on
un-star A: rftoolsstorage:storage_scanner: routable 14 102 1 off
saved routable (B only): 12, 102, 1 has the following block data: [{c: [I; 12, 102, 3]}]
B starred, A unstarred: in-slot '', A 0, B 64, energy 4980
chest at 14,102,1: 32 x minecraft:dirt
scan with A unstarred: rftoolsstorage:storage_scanner: minecraft:dirt 32, minecraft:stone 64 (4900/50000 FE); buffer now 4900
star A again: rftoolsstorage:storage_scanner: routable 14 102 1 on
un-star B: rftoolsstorage:storage_scanner: routable 12 102 3 off
A re-starred, B unstarred: in-slot '', A 96, B 64, energy 4980
saved routable after the relaunch: 12, 102, 1 has the following block data: [{c: [I; 14, 102, 1]}]
after the relaunch: in-slot '', A 160, B 64, energy 4980
```

Five facts make that evidence rather than a print: **the default set is empty** — `data get` answers
`Found no elements matching routable`, and with `energy: 20` the tick pays exactly one attempt
(→ 0) while the 64 stone stays in the in-slot and **both** chests read 0; **the star is what lets
the same stack go** — after starring B and un-starring A the very stack that stayed is routed to B
(64) for one `rfPerInsert` (5,000 → 4,980), A still 0; **the scan still lists the un-starred
inventory** — A, seeded with 32 dirt after the walk, is in the reply
(`minecraft:dirt 32, minecraft:stone 64`), so the set filtered routing, not the list; **the
membership is the rule, not the walk order** (B is first in scan order, so re-starring A and
un-starring B makes A the recipient — the dirt chest's 96 is its 32 dirt plus the new 64 stone,
B unchanged); and **the saved list survives a relaunch** — the block reads back
`[{c: [I; 14, 102, 1]}]` (A only) after quit and re-entry, and the next stack still goes to A
(160) while the un-starred B stays at 64. The one paid attempt with nothing starred is the tick's
own rule (slice 6), not a gap: upstream charges `rfPerInsert` after `injectStackInternal` whatever
the walk did.

### Deviations this slice adds

1. **A command, not a screen.** Upstream's star is the list line's "Routable: Yes/No" choice
   (`GuiStorageScanner.addStorageLine`); the port's list is still a gap (slice 5's deviation 1), so
   the same set membership is `/rftoolsstorage routable <pos> <target> <on|off>`. The command takes
   `on`/`off` rather than toggling, because a command should be idempotent; `runtime::star` is the
   set operation either way.
2. **The target must be an inventory in the current box.** The GUI can only star a line its list
   shows, and the list is every inventory in the box: the command refuses the scanner's own cell, a
   cell outside the box and a cell with no item capability. Upstream's `toggleRoutable` itself would
   accept any position; the saved shape and the walk are the same, only the surface is narrower.
   A star whose inventory later goes away stays saved and is inert (the walk asks the capability).
3. **The star list is the only half of the XNet surface carried.** The per-inventory
   `InventoryAccessSettings` (an input/output permission for the GUI, the screen and automation,
   plus a matcher) arrives from XNet and is still a gap; with no settings the port's access tests
   are upstream's `!inventoriesFromXNet.contains(p)` (true), so every cell the star allows is open
   to every path. The scan's list and price are untouched by the star (upstream passes
   `starred=false` for the list).
4. **A stale star costs nothing.** Upstream's `findInventories` records the box's inventories each
   scan; the port walks the box each time and tests the capability, so a saved star on an empty
   cell is skipped exactly as an unstarred one (the e2e's empty-box insert reaches the refusal with
   CHEST_X still in the saved list).

### Coverage (the families this slice changes)

| Family | State |
|---|---|
| `/rftoolsstorage routable <pos> <target> <on\|off>`, its parsed reply | **tested** — `routable 12 102 3 on`, `routable 14 102 1 on`, `routable 14 102 1 off` (`e2e-routable.log`) |
| the default: a fresh scanner routes nothing | **tested** — the saved key is absent (`Found no elements matching routable`), the tick pays one `rfPerInsert` and both chests stay empty (`e2e-routable.log`); `a_fresh_scanners_star_list_is_empty_and_a_star_switches_one_cell` |
| the star filters the routing walks (tick here; the request/give/insert walk uses the same `scan_targets`) | **tested** — only the starred B receives; A stays empty while unstarred (`e2e-routable.log`); `the_walk_skips_a_cell_that_is_not_starred`; the earlier routing phases now star their chests and still pass |
| the scan still lists an un-starred inventory | **tested** — A's 32 dirt appears beside B's 64 stone with A unstarred (`e2e-routable.log`) |
| the saved shape (`routable` = `[{c: [I; x, y, z]}]`) and its round trip | **tested** — in game the block reads `[{c: [I; 12, 102, 3]}]`, and after a relaunch `[{c: [I; 14, 102, 1]}]` (`e2e-routable.log`); `the_routable_set_is_upstreams_list_of_int_array_compounds` (malformed entries dropped) |
| re-starring restores the walk | **tested** — A unstarred → starred becomes the recipient (96), B unstarred (`e2e-routable.log`) |
| the set survives a relaunch | **tested** — the saved list and the routing both survive (`e2e-routable.log`) |
| `checkForRoutableInventories` = a **starred**, loaded inventory | **tested by reuse** — the insert's empty-box leg refuses with the saved star on the removed chest (`e2e-routable.log`, the insert phase) |
| XNet access settings (per-path permissions, the matcher) | **not ported** (needs XNet; the star list is separate — deviation 3) |
| the GUI list widget itself, compact/cycle/visible-size, the remote/tablet variant, the claimed-land check | **not ported** (the screen is a fixed slot layout; `blocked(no permission query)` for the claim) |

### Traps this slice paid for

1. **The tick charges on every tick while the stack stays.** An unstarred walk moves nothing, so the
   in-slot keeps its stack and the next tick pays another `rfPerInsert` — 5,000 FE is gone in 250
   ticks, and the "energy −20" assertion read 0 on the first run. A tick-independent assertion is
   slice 6's: set `energy: 20`, one attempt is paid (→ 0) and the stack is still there. The failed
   run is kept as `e2e-routable-first-run.log`.
2. **The walk is x-major, so the z chest comes first.** The scan order is x, then y, then z: a chest
   at S+(0, 0, +2) sorts before one at S+(+2, 0, 0). Star exactly one of them for every directed
   assertion; "both starred" would make every stack go to the z chest and look like a bug.
3. **`data get` of the key prints differently by state.** Absent: `Found no elements matching
   routable`. Present: `[{c: [I; 12, 102, 3]}]` (an int array, `I;`, spaces after the commas) —
   the saved-shape assertion parses exactly that, which is also the upstream `BlockPosTools` shape.
4. **`fill_chest` recreates the block but not the star.** The star is a position in the scanner's
   saved compound, so replacing a chest (or chest → barrel → chest) at the same cell keeps it
   starred — which is why the later inject/insert phases' barrels are still routable.
5. **The earlier routing legs are a real behaviour change.** With the empty default, every chest the
   inject/request/give/insert phases route into has to be starred first; the phase adds the calls
   (and the e2e's own star leg is between the relaunch and the inject phases). A leg that forgets
   one fails with the chest untouched and the stack still in the slot — not a port bug.

## Slice 13 — the scanner's export choice (version 0.2.13)

The gap line slice 12 left, verbatim from this file's gap list and slice 12's own
next-slice note:

> the storage scanner's remaining half: the GUI list widget's **compact, cycle and
> visible-size controls** (its **search** is slice 10, its **sort** is slice 11, its
> **routable star list** is slice 12)

The slice is the list's **export choice** and the **row selection** it uses:
`/rftoolsstorage export <pos> <current|first>` sets `StorageScannerData.exportC`
(the GUI's `exportToStarred` "Export to current container" choice) and
`/rftoolsstorage select <pos> <target>` names the row the list would select
(`getInventoryForBlock`'s `lastSelectedInventory`). Under `current` the tick's GUI-slot
stack goes **only** into that inventory and refuses rather than falling through to the
starred walk. It fits the **machines** tier: one saved flag and one saved cell in the
scanner's compound, the routing walk it already has, no renderer, no screen, no new
vanilla hook. Upstream read at `fc1b18a7f070d0030671e1db1d3aab90699fd160`
(`StorageScannerTileEntity.java`, `data/StorageScannerData.java`,
`client/GuiStorageScanner.java`; the raw files were read into `/tmp/opencode` and deleted
after the findings were recorded).

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094c0fc2fdda6c627fe9eff22dec6a906f0`, mod API 0.2, game 26.3. **Version 0.2.13.**

### The artefact's rules (read, not invented)

- **The choice is `exportC`.** `StorageScannerData` is `(radius, exportC, wideview,
  sortingMode)` with `DEFAULT` `(1, false, true, NAME)`, and the codec key is
  **`exportc`**. `GuiStorageScanner.exportToStarred` has two choices — "Export to current
  container" and "Export to first routable container" — and
  `setCurrentChoice(isExportToCurrent() ? 0 : 1)`: `exportC = true` is the **current**
  container, `false` (the default) the **first routable** one, the walk every slice since
  6 uses.
- **It affects the tick's GUI slot only.** `tickServer` passes `exportC` to
  `injectStackInternal` for `SLOT_IN` and `false` for `SLOT_IN_AUTO`;
  `injectStackFromScreen` also passes `false`. So the automation slot and the screen's
  insert always walk.
- **The selected attempt is the only attempt.** With `toSelected` and a real
  `lastSelectedInventory`, `injectStackInternal` tries that one block entity (not another
  scanner, not a null one), inserts what it will, and **returns** with the remainder: a
  selected container that refuses leaves the stack in the slot. It never falls through to
  the walk.
- **The selection is the list's row.** `getInventoryForBlock(cpos)` sets
  `lastSelectedInventory = cpos`, then lists that row's inventory — or, for the list's
  first "All routable" row, `(-1, -1, -1)`, every starred inventory. The selected attempt
  ignores that sentinel (`getY() != -1`) and does **not** test `isRoutable` (the list shows
  every inventory in the box); the star filters the walk alone.
- **Upstream's field is transient.** `lastSelectedInventory` is a plain field with no
  saved key, so a relaunch loses it and a `current` scanner with no selection falls back
  to the walk.

### What the slice does

- **`runtime`** (pure, tested): [`ExportMode`] — `word`/`from_word` (`current`,
  `first`), `saved`/`from_saved` (the `exportC` boolean, default `First`) —,
  [`ExportRoute`] and [`export_route`] (`Only(selected)` under `Current` with a
  selection, `Walk` otherwise), [`export_line`] and [`selection_line`].
- **`support`**: `KEY_EXPORT = "exportc"` and `KEY_SELECTED = "selected"`, the
  `saved_export`/`saved_selection`/`selection_tag` readers and writer, the two commands
  (their replies `export current|first` and `select x y z`), and the `route` parameter on
  `inject_stack`: `Only(at)` builds its target list from `injectable_at` alone — the own
  cell and another scanner skipped, no star and no radius, upstream's selected branch —
  and the tick asks for the route on the **In** step only.
- **The selection is saved** (deviation 11): upstream's transient field becomes a cell in
  the block's compound, so a relaunch keeps both the mode and the row.
- **Unchanged**: the scan readout (still the whole box), the star's filter on the walks,
  the inject/request/give/insert paths and their prices.

### Red → green (the rules)

The three rules were written first with `word`/`from_word`/`export_line`/`selection_line`
stubbed (`""`/`None`), `saved` `false`, `from_saved` always `First` and `export_route`
always `Walk`, and the suite was run **before** implementing them
(`ports/_shared/evidence/rftoolsstorage/slice13-red.log`, `TEST_EXIT:101`):

```
thread 'runtime::tests::the_current_choice_sends_the_stack_only_to_the_selected_inventory' (131071) panicked at src/runtime.rs:2139:9:
assertion `left == right` failed: the current-container choice exports into the selected inventory
  left: Walk
 right: Only([12, 102, 3])
...
thread 'runtime::tests::the_export_and_selection_replies_name_the_choice_and_the_cell' (131072) panicked at src/runtime.rs:2163:9:
assertion `left == right` failed
  left: ""
 right: "export current"
...
thread 'runtime::tests::the_export_choice_is_upstreams_exportc_flag_and_defaults_to_the_walk' (131073) panicked at src/runtime.rs:2109:9:
assertion `left == right` failed
  left: None
 right: Some(Current)
...
test result: FAILED. 50 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Implemented, then **54 passed, 0 failed** (`sdk-test.log`, `slice13-test.log`): the
runtime's 48 (45 + the three export rules) and the six glue tests, one of them new
(`the_export_choice_and_the_selection_round_trip_through_the_saved_compound`).

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built 0.2.13 package | `e2e-export.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-export-control.log` | **FAIL** + `EXIT:1` — `{"error":"unknown item rftoolsstorage:storage_scanner"}` |
| `EXPORT_SELFTEST=1` (the selected chest must *not* have taken the stack) | `e2e-export-selftest.log` | **FAIL** + `EXIT:1` — *"the current-container export sent the 64 stone to the selected B (64) and left A empty (0) … so this inverted assertion fails as it must"* |
| the first reaching run (kept: it found the charge-trap in an extra assertion) | `e2e-export-first-run.log` | **FAIL** + `EXIT:1` — *"the walk left 5000, expected one rfPerInsert off 5000"* |

`e2e-export.log` re-runs every earlier phase (slices 1–12 still pass), then asserts the
brief's rig — scanner at S, radius 2, two empty chests two blocks away (A at S+2 on x,
B at S+2 on z, so B is first in the x-major walk), both starred — with every `ctl` reply
in the log:

```
saved exportc (default): Found no elements matching exportc
export current: rftoolsstorage:storage_scanner: export current
saved exportc: 1b
select B: rftoolsstorage:storage_scanner: select 16 102 12
saved selected (B): 16, 102, 10 has the following block data: {c: [I; 16, 102, 12]}
export current, B selected: in-slot '', A 0, B 64, energy 4980
select A: rftoolsstorage:storage_scanner: select 18 102 10
export current, A selected: in-slot '', A 64, B 64
saved exportc after the relaunch: 1b
saved selected after the relaunch: 16, 102, 10 has the following block data: {c: [I; 18, 102, 10]}
select the full barrel: rftoolsstorage:storage_scanner: select 16 102 11
selected full barrel: in-slot 64, barrel slot 26 64, A 64, B 64
export first: rftoolsstorage:storage_scanner: export first
saved exportc: 0b
export first: in-slot '', A 64, B 128, energy 5000
```

Five facts make that evidence rather than a print: the flag is **absent** on a fresh
scanner and reads `1b` after the command (the mode is the saved `exportc`); the very
first tick puts the 64 stone in the **selected B only** while the equally starred A stays
0, for exactly one `rfPerInsert`; `select A` moves the next stack to A while B keeps its
64 (the selection, not the walk order, decides); both keys survive quit + re-entry
(`exportc` 1b, `selected` = A) and the mode still works; a **selected full barrel**
leaves the 64 in the slot with A and B unchanged — the no-fall-through rule, which a
walk would have broken (both chests had room); and `export first` saves `0b` and routes
that same stack to the walk's first starred cell (B 128). `EXPORT_SELFTEST=1` inverts the
first assertion and the run ends `FAIL:` + `EXIT:1`.

### Deviations this slice adds

1. **The selection is saved with the block** (deviation 11 above): upstream's
   `lastSelectedInventory` is transient. The mode is upstream's own `exportc` key; the
   cell is this port's `selected` key, written in the `{c: [I; x, y, z]}` shape a star
   entry has.
2. **The selected row does not change the readout.** Upstream's `getInventoryForBlock`
   lists only the selected row's inventory; this port has no list widget, and `scan`
   lists the whole box (slice 5's deviation 1), so `select` only feeds the export. The
   row's contents view stays a gap with the widget.
3. **`export first` is the way back to the walk**, not a "clear the selection" form: the
   list's first row ("All routable") sets upstream's `(-1, -1, -1)` sentinel, which merely
   disables the selected branch; the port's `first` mode does the same thing explicitly.

### Coverage (the families this slice changes)

| Family | State |
|---|---|
| `/rftoolsstorage export <pos> <current\|first>`, its parsed reply | **tested** — `export current` then `export first`, with `exportc` `1b` → `0b` (`e2e-export.log`) |
| the choice is saved with the block (`exportc`) and survives a relaunch | **tested** — absent on a fresh scanner, `1b` after the command, `1b` after quit + re-entry (`e2e-export.log`); `the_export_choice_is_upstreams_exportc_flag…`, the round-trip glue test |
| `/rftoolsstorage select <pos> <target>`, its parsed reply, and the saved cell | **tested** — `select B`, `select A`, `select <barrel>`; `{c: [I; …]}` read back, and after the relaunch (`e2e-export.log`); the glue test's malformed-entry cases |
| `current`: the tick's GUI-slot stack goes to the selected inventory only | **tested** — B 64 with A 0 for one `rfPerInsert` (`e2e-export.log`); `the_current_choice_sends_the_stack_only…` |
| the selection follows the list's row | **tested** — `select A` makes A 64 while B keeps 64 (`e2e-export.log`) |
| a selected container that refuses does not fall through to the walk | **tested** — a full selected barrel leaves the 64 in the slot while both starred chests had room (`e2e-export.log`) |
| `first` restores the starred walk | **tested** — the same stack goes to B (128) after `export first` (`e2e-export.log`) |
| the choice affects the GUI slot only (the automation slot always walks) | unit-level by construction (`InjectSlot::In` is the only step routed through `export_route`); the earlier auto-slot leg still passes |
| the selected row's contents view (`getInventoryForBlock`'s listing) | **not ported** (no list widget; `scan` lists the whole box — deviation 2) |
| the XNet access tests and matcher on the selected attempt | **not ported** (true without XNet; slice 12's deviation 3) |
| the GUI list widget itself, compact/cycle/visible-size, the remote/tablet variant, the claimed-land check | **not ported** (the screen is a fixed slot layout; `blocked(no permission query)` for the claim) |

### Traps this slice paid for

1. **The tick charges on every tick while a refused stack stays.** The barrel phase's
   buffer drains while the 64 stone sits there, so a later "energy −20" assertion is
   decided by how many ticks the chat round trip cost — the first reaching run failed on
   exactly that (`e2e-export-first-run.log`). Assert the stack, and pin a charge only in
   a phase whose stack leaves in one tick (the first export leg: 4,980).
2. **The selected cell is not filtered by the star.** Upstream's selected attempt tests
   `te != null`, "not a scanner", the XNet tests and the matcher — no `isRoutable`, no
   radius. The port keeps that: `select` validates the cell at command time (in the box,
   with an item capability, not the scanner), and the tick's `Only(at)` route uses
   `injectable_at` alone, so a stale selection is inert rather than silently rerouted.
3. **`data get` of `selected` prints one compound, not a list** — `{c: [I; 16, 102, 12]}`
   — while `routable` prints the list. The assertion parses the cell inside either shape.
4. **A relaunch mid-phase is what proves the deviation.** The star list's own relaunch
   leg cannot stand in for it: the mode and the selection are two different keys, and
   only a re-entry shows that a `current` scanner still knows where to export.
5. **Two adjacent chests are one double chest** (slice 6's trap): A is on x, B on z, and
   the full barrel is a barrel, so no pair joins.

## Slice 14 — the list's row contents view (version 0.2.14)

The gap line slice 13 left, verbatim from its coverage table and deviation 11:

> the selected row's contents view (`getInventoryForBlock`'s listing) | **not ported** (no list
> widget; `scan` lists the whole box — deviation 2)

The slice is that view: `/rftoolsstorage contents <pos> <target|all>`. It is the other half of
`getInventoryForBlock` — a cell lists **that** inventory's contents, and the list's first
"All routable" line (upstream's `(-1, -1, -1)` sentinel) lists **only the starred** inventories,
which is `countItems(…, starred = true)`'s half of the star, observable in this port for the first
time. It fits the **machines** tier: the same block entity's saved radius, sort and star set, the
neighbours' item capability and a readout — no renderer, no screen, no new vanilla hook. Upstream
read at `fc1b18a7f070d0030671e1db1d3aab90699fd160` (`StorageScannerTileEntity.java`,
`GuiStorageScanner.java`; the shallow clone was deleted after the findings were recorded).

**Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API 0.2, game 26.3. **Version 0.2.14.**

### The artefact's rules (read, not invented)

- **Two branches, one method.** `getInventoryForBlock(cpos, stacks, craftable)` sets
  `lastSelectedInventory = cpos` and then: for `cpos.getY() == -1` it walks `inventories` and calls
  `addItemsFromInventory` only for `routable.contains(blockPos)`; otherwise it calls
  `addItemsFromInventory(cpos)` — no `isRoutable`, no radius, no star. A single row is therefore
  listed whether or not the player starred it. (Slice 13's `select` writes the same field; this
  slice is its read.)
- **The rows are the scan's own list.** `inventories` is filled by `findInventories`: in range, not a
  `StorageScannerTileEntity`, not a `CraftingManagerTileEntity` (a family this port does not carry),
  and `getCapability(Capabilities.ItemHandler.BLOCK, …, null) != null`. A cell outside the box, the
  scanner's own cell, another scanner or an empty cell has no row, so the GUI cannot select it. The
  port's `contents_row` is exactly that test.
- **The merge is item + components.** `addItemStack` grows an entry only for
  `ItemStack.isSameItemSameComponents`. This port's readout is one entry a kind (`Found {item,
  count}`, the shape `scan` already prints), so the row view shares that model — deviation 2.
- **The listing is free.** `getInventoryForBlock` never calls `consumeEnergy`: the GUI lists for
  free and the buffer is spent on the request paths alone (`requestStack`,
  `giveToPlayerFromScreen`, `injectStackFromScreen`, the tick). **`contents` costs no FE** — not
  `rfPerRequest`, not a tenth — and a listing at a 0 buffer answers. `scan` stays priced: it is the
  *request substitute* readout of slice 5's deviation 1, a different surface, and every earlier
  leg's prices are unchanged.
- **The GUI sorts and filters whichever row it shows** (`updateContentsList`). The port applies the
  saved sort to `contents` (one saved mode, `sort`); the search stays `scan`'s typed argument
  (deviation 3).

### What the slice does

- **`runtime`** (pure, tested): `in_scan_box` (the cube `findInventories` walks), [`contents_row`]
  (in the box, not the scanner's cell, an inventory), `contents_line` (the row's kinds, or the all
  row's), `contents_refused_line`, and `found_list` — the listing lifted out of `scan_line` so the
  priced readout and the free row view print in one voice.
- **`support`**: `cell_entries` (one cell's capability slot by slot, extracted from `scan_found`),
  `contents_readout` (the two branches, the row test, the saved sort, no charge), the two usages
  `contents <pos> <target>` and `contents <pos> all`, and the dispatch arm. `all` is the second
  usage's literal, so the invocation's literal path is `[rftoolsstorage, contents, all]` and the
  **absent** `target` names that row; a cell is the `block_pos` argument.
- **Unchanged**: `scan` (the whole box, still 100 FE), the star's routing filter, the
  request/give/insert prices, the export route.

### Red → green (the rules)

The three rules were written first with the functions stubbed (`in_scan_box`/`contents_row` false,
`contents_line`/`contents_refused_line` empty) and the suite was run **before** implementing them
(`slice14-red.log`, `TEST_EXIT:101`):

```
thread 'runtime::tests::a_contents_row_is_an_inventory_the_scan_lists' panicked at src/runtime.rs:2252:9:
a chest in the box is a row
...
thread 'runtime::tests::the_contents_reply_names_the_row_or_the_all_row_and_its_kinds' panicked at src/runtime.rs:2290:9:
assertion `left == right` failed
  left: ""
 right: "contents 16 102 12: minecraft:stone 128"
...
thread 'runtime::tests::a_cell_the_scan_did_not_list_is_refused_with_the_box_reason' panicked at src/runtime.rs:2327:9:
assertion `left == right` failed
  left: ""
 right: "19 102 10 is not an inventory in the scan box"
...
test result: FAILED. 54 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
TEST_EXIT:101
```

The next run found a **defect in the test**, not in the code: its "y is bounded" case named
`[18, 103, 10]` for a radius-2 box, which is *inside* it (`the box is a cube: y is bounded too`
failed because the cell was in the box). The rule was right; the case was corrected to
`[18, 105, 10]`. Implemented, then **57 passed, 0 failed** (`sdk-test.log`, `slice14-test.log`): the
runtime's 51 (48 + 3) and the six glue tests.

### In game (`ports/_shared/evidence/rftoolsstorage-e2e/`)

| Leg | Log | Verdict |
|---|---|---|
| default, the launcher-built 0.2.14 package | `e2e-contents.log` | **PASS** + `EXIT:0` |
| control (the same Nightly without the package) | `e2e-contents-control.log` | **FAIL** + `EXIT:1` — `{"error":"unknown item rftoolsstorage:storage_scanner"}` |
| `CONTENTS_SELFTEST=1` (the row view of B must NOT have listed its 128) | `e2e-contents-selftest.log` | **FAIL** + `EXIT:1` — *"the row view of B answered 'rftoolsstorage:storage_scanner: contents 13 102 0: minecraft:stone 128' for the 128 stone it holds, so this inverted assertion fails as it must"* |

The rig — scanner at S = 13,102,10, radius 2, A at S+2 on **x** with 64 stone, B at S+2 on **z**
with 128 stone, both starred, buffer **0** — with every `ctl` reply in the log:

```
radius: rftoolsstorage:storage_scanner: radius 2
star A: rftoolsstorage:storage_scanner: routable 15 102 10 on
star B: rftoolsstorage:storage_scanner: routable 13 102 12 on
contents B: rftoolsstorage:storage_scanner: contents 13 102 12: minecraft:stone 128
contents A: rftoolsstorage:storage_scanner: contents 15 102 10: minecraft:stone 64
energy after two row listings: 0/50000
un-star B: rftoolsstorage:storage_scanner: routable 13 102 12 off
contents all (B unstarred): rftoolsstorage:storage_scanner: contents all: minecraft:stone 64
contents B (unstarred): rftoolsstorage:storage_scanner: contents 13 102 12: minecraft:stone 128
un-star A: rftoolsstorage:storage_scanner: routable 15 102 10 off
contents all (nothing starred): rftoolsstorage:storage_scanner: contents all: empty
star B again: rftoolsstorage:storage_scanner: routable 13 102 12 on
contents all (B only): rftoolsstorage:storage_scanner: contents all: minecraft:stone 128
contents at S+3 (outside radius 2): rftoolsstorage:storage_scanner: 16 102 10 is not an inventory in the scan box
contents of the scanner itself: rftoolsstorage:storage_scanner: 13 102 10 is not an inventory in the scan box
contents of an empty cell: rftoolsstorage:storage_scanner: 14 102 10 is not an inventory in the scan box
energy after three refusals: 0/50000
```

Five facts make that evidence rather than a print: **the two rows differ** (128 vs 64 from one
scanner, so the target is what selects the inventory); **the buffer is 0 before and after** every
listing and every refusal — a listing is not a request, and this is the assertion that would fail if
`contents` were priced (the earlier `scan` legs at 0 FE answer `no power`); **`all` is the starred
half** — un-starring B drops it from the all row (64) while **B's own row still lists its 128**,
which is the single-row branch's missing `isRoutable` test; with nothing starred the all row is
`empty`, and starring B alone lists its 128 (the membership is the rule, not the walk order); and
**the rows are the scan's list** — an outside cell, the scanner itself and an empty cell are each
refused with the same reason. `CONTENTS_SELFTEST=1` inverts the first assertion and the run ends
`FAIL:` + `EXIT:1`.

### Deviations this slice adds

1. **The commands are the list's two rows, not a widget.** Upstream's view is
   `GuiStorageScanner.updateContentsList` over `getInventoryForBlock`'s answer; this port has no list
   widget (still the gap), so the same answer is `contents <pos> <target|all>` — a cell for a row,
   `all` for the first line. The rows and the star filter are the same; only the surface differs.
2. **One entry a kind**, item id and count, as `scan` (slice 5's `Found`): upstream keeps two stacks
   of one item apart when their components differ. The collapse `scan` already makes; the row's and
   the star's behaviour are unaffected.
3. **The search is `scan`'s argument, not `contents`'s.** The GUI's search box narrows whichever row
   is shown; the port's search is a typed term (slice 10), so `contents` lists the row unfiltered and
   `scan <pos> <query>` is how a listing is narrowed. The saved sort *is* applied to `contents`,
   because it is saved state, as it is to `scan`.
4. **A row the scan did not find is refused** rather than answering `empty`: the GUI can only select
   a line its list shows. A cell that lost its inventory between the scan and the command is refused
   the same way (the port walks the box per command; there is no cached list).

### Coverage (the families this slice changes)

| Family | State |
|---|---|
| `/rftoolsstorage contents <pos> <target\|all>`, its parsed reply | **tested** — `contents 13 102 12: minecraft:stone 128`, `contents all: minecraft:stone 64`, `… empty` (`e2e-contents.log`) |
| a row lists that inventory, starred or not | **tested** — B's row lists 128 while B is unstarred (`e2e-contents.log`); `a_contents_row_is_an_inventory_the_scan_lists` |
| `all` lists only the starred rows (`countItems(…, starred = true)`'s other half) | **tested** — the all row reads 128, then 64 when B is un-starred, `empty` with nothing starred, 128 with B alone (`e2e-contents.log`) |
| a row the scan did not find is refused | **tested** — outside the box, the scanner's cell and an empty cell (`e2e-contents.log`); `contents_refused_line` |
| the price: none, a listing is not a request | **tested** — every listing and refusal runs at buffer 0, which stays 0 (`e2e-contents.log`) |
| the saved sort orders the row | by construction (the same `sort_found` + `saved_sort` as `scan`); the rig holds one kind, so there is no in-game order assertion |
| the GUI list widget itself, compact/cycle/visible-size, the remote/tablet variant, XNet settings, the claimed-land check | **not ported** (the screen is a fixed slot layout; `blocked(no permission query)` for the claim) |

### Traps this slice paid for

1. **A literal may follow an argument in a usage** (`contents <pos:block_pos> all`): the pin's
   `usage_nodes` accepts a word after an argument (only a word after an *optional* argument, or after
   a greedy `text`, is refused) and merges the two usages into one tree. So the dispatch reads
   `literals` by index (not the old two-element destructure), and the `all` row is identified by the
   **absent** `target` argument — which doubles as proof the parse went down that branch.
2. **The buffer is the fail-capable half of "free".** Assert `energy` before and after, at 0: a
   priced `contents` answers `no power` and the assertion fails on the reply, not on a coincidence.
3. **Two adjacent chests are one double chest** (slice 6's trap): A is on x, B on z, two blocks out
   on different axes — the rig the star and export phases already use.
4. **The scanner's own slots must stay empty** while the phase counts a chest: the tick runs during
   every `ctl wait`, and a stack in the in/auto slot would be routed into A or B between two
   listings. `seed_slots "$SX" "$SY" "$SZ" ''` clears the container before the star calls.
5. **A doc edit after the build breaks `package matches the tree`.** The first delivered 0.2.14
   (101,395 B) was built, checked, fitted and played green — and then this README's "What is not
   ported" line was corrected, which the gate caught as `differs: README.md`. The honest route is a
   rebuild (101,409 B) and the three legs again on the delivered file, which is what the logs above
   are. Edit docs **before** the final build, every time.

## Verification

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

Full evidence in [`VALIDATION.md`](VALIDATION.md); the e2e logs are in
`ports/_shared/evidence/rftoolsstorage-e2e/` (the game legs) and
`ports/_shared/evidence/rftoolsstorage/` (the ladder, package and fits).

**0.2.14 — the list's row contents view (2026-11-23, pin `0a875da`)**

| Step | Command | Result |
|---|---|---|
| Red proof | `cargo test` with `in_scan_box`/`contents_row` false and `contents_line`/`contents_refused_line` empty | **FAILED. 54 passed; 3 failed**, `TEST_EXIT:101` (`slice14-red.log`); quoted above |
| Runtime + glue tests | `cargo test` | **57 passed**, 0 failed (`sdk-test.log`, `slice14-test.log`, `TEST_EXIT:0`) — 51 runtime + 6 glue |
| Lints | `cargo clippy --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`slice14-clippy.log`; the first run flagged `cloned_ref_to_slice_refs` in the new test, fixed with `std::slice::from_ref`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` (`slice14-fmt.log`; the first check listed the wrapped `contents_readout` match arm, formatted, then re-run green) |
| SDK gate | `cargo horizon-mod check` | `TREE_CHECK_EXIT:0` — *"…uses 540 items of the API (5 Part kinds, 2 capabilitys, 149 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 168 methods, 29 traits, 115 types). Made for 0a875da."* (`slice14-tree-check.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **101,409 B**, SHA-256 `4fde9c1eff9f97aea14f1324981d3c872e1b8d41d2155117dbe8d8bf22b53463`, **`cmp`-identical** (`slice14-build-1.log`, `slice14-build-2.log`); the first delivered build (101,395 B, sha `8d839714…bb90`) was superseded when this README's "What is not ported" line was corrected after it — `verify-sdk-package` caught that as `differs: README.md`, and the delivered file is the rebuild from the final tree |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.14 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **101,409 bytes**, `4fde9c1e…3463`, `FILE_CHECK_EXIT:0` (`slice14-check.log`) |
| Entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`2badcc21abb7`) |
| Fits the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, `FITS_PIN_EXIT:0` (`slice14-fits.log`) |
| Fits another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, `FITS_OTHER_EXIT:0` (`slice14-fits.log`) |
| Launcher build | `BUILD_LOCAL=1 launcher-game.sh build rftoolsstorage` (the two stale `native-*` tags carrying 0.2.13 removed first) | `Compiling rftoolsstorage (1 of 1)`, tag `native-0a875da-3123db72eda1`, `LAUNCHER_BUILD_EXIT:0` (`slice14-launcher-build.log`) |
| In game | `build-slot.sh run --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` (`CONTROL_GAME=`) | **PASS** + `EXIT:0` (`e2e-contents.log`) |
| Control | the same rig on the Nightly without the package | **FAIL** + `EXIT:1` (`e2e-contents-control.log`) |
| Selftest | `CONTENTS_SELFTEST=1` (the row view of B must not have listed 128) | **FAIL** + `EXIT:1` (`e2e-contents-selftest.log`) |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=39  ok=39  missing-exit=0  bad-exit=0`, exit 0 |
| Package gate | `verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 |
| Recipe types | `check-recipe-types.py rftoolsstorage` | `rftoolsstorage: ok`, exit 0 |
| Skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS |

**0.2.13 — the scanner's export choice (2026-11-23, pin `0a875da`)**

| Step | Command | Result |
|---|---|---|
| Red proof | `cargo test` with `word`/`from_word`/`export_line`/`selection_line` stubbed, `saved` false, `from_saved` always `First` and `export_route` always `Walk` | **FAILED. 50 passed; 3 failed**, `TEST_EXIT:101` (`slice13-red.log`); quoted above |
| Runtime + glue tests | `cargo test` | **54 passed**, 0 failed (`sdk-test.log`, `slice13-test.log`, `TEST_EXIT:0`) — 48 runtime + 6 glue |
| Lints | `cargo clippy --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`slice13-clippy.log`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` (`slice13-fmt.log`; the first check listed the import wrapping and one join, formatted, then re-run green) |
| SDK gate | `cargo horizon-mod check` | `TREE_CHECK_EXIT:0` — *"…uses 540 items of the API (5 Part kinds, 2 capabilitys, 149 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 168 methods, 29 traits, 115 types). Made for 0a875da."* (`slice13-tree-check.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **98,817 B**, SHA-256 `f89f5974f411cf012a7c5a28242185e630bd6c9ffa726939863ceb1197baeb83`, **`cmp`-identical** (`slice13-build-1.log`, `slice13-build-2.log`); the packed README's 0.2.10 text was corrected after the first green leg, and `slice13-readme-delta.log` proves that rebuild changed **only** `README.md` (every `src/` entry and every other file byte-identical), after which all three game legs were re-run on the delivered file |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.13 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **98,817 bytes**, `f89f5974…eb83`, `FILE_CHECK_EXIT:0` (`slice13-check.log`) |
| Entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`9af88f59eccb`) |
| Fits the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, `FITS_PIN_EXIT:0` (`slice13-fits.log`) |
| Fits another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, `FITS_OTHER_EXIT:0` (`slice13-fits.log`) |
| Launcher build | `BUILD_LOCAL=1 launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, build tag `native-0a875da-f1a40e1d7361`, `LAUNCHER_BUILD_EXIT:0` (`slice13-launcher-build.log`) |
| In game | `build-slot.sh run --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` (`CONTROL_GAME=`) | **PASS** + `EXIT:0` (`e2e-export.log`) |
| Control | the same rig on the Nightly without the package | **FAIL** + `EXIT:1` (`e2e-export-control.log`) |
| Selftest | `EXPORT_SELFTEST=1` (the selected chest must not have taken the stack) | **FAIL** + `EXIT:1` (`e2e-export-selftest.log`) |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=36  ok=36  missing-exit=0  bad-exit=0`, exit 0 |
| Package gate | `verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 |
| Skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS |

**0.2.12 — the scanner's routable star list (2026-11-23, pin `0a875da`)**

| Step | Command | Result |
|---|---|---|
| Red proof | `cargo test` with `is_routable`/`star`/`routable_line` stubbed and `routing_target` ignoring the star | **FAILED. 46 passed; 3 failed**, `TEST_EXIT:101` (`slice12-red.log`); quoted above |
| Runtime + glue tests | `cargo test` | **50 passed**, 0 failed (`sdk-test.log`, `slice12-test.log`, `TEST_EXIT:0`) — 45 runtime + 5 glue |
| Lints | `cargo clippy --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-clippy.log`, `slice12-clippy.log`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` (`slice12-fmt.log`; the first check listed two wrapping diffs, fixed with `rustfmt` on the three files, then re-run green) |
| SDK gate | `cargo horizon-mod check` | `TREE_CHECK_EXIT:0` — *"…uses 540 items of the API (5 Part kinds, 2 capabilitys, 149 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 168 methods, 29 traits, 115 types). Made for 0a875da."* (`slice12-tree-check.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **95,274 B**, SHA-256 `d194c73b31d2d3e2b3bf60e9ed3b727a2feac90e26ff33cf2234fbd3df47991b`, **`cmp`-identical** (`slice12-build-1.log`, `slice12-build-2.log`) |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.12 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **95,274 bytes**, `d194c73b…991b`, `FILE_CHECK_EXIT:0` (`slice12-check.log`) |
| Entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`7522c53c…a7c`) |
| Fits the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, `FITS_PIN_EXIT:0` (`slice12-fits.log`) |
| Fits another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, `FITS_OTHER_EXIT:0` (`slice12-fits.log`) |
| Launcher build | `BUILD_LOCAL=1 launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, build tag `native-0a875da-d4df6925a64a`, `LAUNCHER_BUILD_EXIT:0` (`slice12-launcher-build.log`) |
| In game | `build-slot.sh run --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` (`CONTROL_GAME=`) | **PASS** + `EXIT:0` (`e2e-routable.log`) |
| Control | the same rig on the Nightly without the package | **FAIL** + `EXIT:1` (`e2e-routable-control.log`) |
| Selftest | `ROUTABLE_SELFTEST=1` (the un-starred chest must not have taken the stack) | **FAIL** + `EXIT:1` (`e2e-routable-selftest.log`) |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=32  ok=32  missing-exit=0  bad-exit=0`, exit 0 |
| Package gate | `verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 |
| Skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS |

**0.2.11 — the scanner list's sort choice (2026-11-23, pin `0a875da`)**

| Step | Command | Result |
|---|---|---|
| Red proof | `cargo test` with `description`/`serialized_name`/`from_description`/`from_saved`/`available`/`sort_found` stubbed | **FAILED. 42 passed; 4 failed**, `TEST_EXIT:101` (`slice11-red.log`); quoted above |
| Runtime + glue tests | `cargo test` | **46 passed**, 0 failed (`sdk-test.log`, `slice11-test.log`, `TEST_EXIT:0`) — 42 runtime + 4 glue |
| Lints | `cargo clippy --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-clippy.log`, `slice11-clippy.log`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` (`slice11-fmt.log`) |
| SDK gate | `cargo horizon-mod check` | `TREE_CHECK_EXIT:0` — *"…uses 537 items of the API (5 Part kinds, 2 capabilitys, 147 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 167 methods, 29 traits, 115 types). Made for 0a875da."* (`slice11-tree-check.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **91,952 B**, SHA-256 `7820eeccb3d40fba47ef72041bce5c39a216d700dd1cd43756c10af7ba4979a8`, **`cmp`-identical** (`slice11-build-1.log`, `slice11-build-2.log`) |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.11 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **91,952 bytes**, `7820eecc…79a8`, `FILE_CHECK_EXIT:0` (`sdk-check.log`, `slice11-check.log`) |
| Entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`2417ab53a2fb`) |
| Fits the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, `FITS_PIN_EXIT:0` (`sdk-fits.log`, `slice11-fits.log`) |
| Fits another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, `FITS_OTHER_EXIT:0` (`sdk-fits.log`, `slice11-fits.log`) |
| Launcher build | `BUILD_LOCAL=1 launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, build tag `native-0a875da-a6e0d2c73be2`, `LAUNCHER_BUILD_EXIT:0` (`slice11-launcher-build.log`) |
| In game | `build-slot.sh run --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` (`CONTROL_GAME=`) | **PASS** + `EXIT:0` (`e2e-sort.log`) |
| Control | the same rig on the Nightly without the package | **FAIL** + `EXIT:1` (`e2e-sort-control.log`) |
| Selftest | `SORT_SELFTEST=1` (the `Amount-` reply must not start with stone) | **FAIL** + `EXIT:1` (`e2e-sort-selftest.log`) |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=28  ok=28  missing-exit=0  bad-exit=0`, exit 0 |
| Package gate | `verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 |
| Skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS |

**0.2.10 — the scan's search filter (2026-11-23, pin `0a875da`)**

| Step | Command | Result |
|---|---|---|
| Red proof | `cargo test` with `matches_search`/`search_namespace`/`search_name` stubbed | **FAILED. 39 passed; 4 failed**, `TEST_EXIT:101` (`slice10-red.log`); quoted above |
| Runtime + glue tests | `cargo test` | **43 passed**, 0 failed (`sdk-test.log`, `slice10-test.log`, `TEST_EXIT:0`) — 39 runtime + 4 glue |
| Lints | `cargo clippy --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-clippy.log`, `slice10-clippy.log`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` (`slice10-fmt.log`) |
| SDK gate | `cargo horizon-mod check` | `TREE_CHECK_EXIT:0` — *"…uses 537 items of the API (5 Part kinds, 2 capabilitys, 147 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 167 methods, 29 traits, 115 types). Made for 0a875da."* (`slice10-tree-check.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **88,702 B**, SHA-256 `1aadf54d967054073d7b0898e1c6b60005829440d54105bb69296543d494f868`, **`cmp`-identical** (`slice10-build-1.log`, `slice10-build-2.log`) |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.10 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **88,702 bytes**, `1aadf54d…f868`, `FILE_CHECK_EXIT:0` (`sdk-check.log`, `slice10-check.log`) |
| Entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`be8bc239…20a3`) (`sdk-sha256.txt`) |
| Fits the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, `FITS_PIN_EXIT:0` (sha `1aadf54d…f868` in the log) (`sdk-fits.log`, `slice10-fits.log`) |
| Fits another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, `FITS_OTHER_EXIT:0` (`sdk-fits.log`, `slice10-fits.log`) |
| Launcher build | `BUILD_LOCAL=1 launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, build tag `native-0a875da-7094a7593c52`, `LAUNCHER_BUILD_EXIT:0` (`slice10-launcher-build.log`) |
| In game | `build-slot.sh run --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` (`CONTROL_GAME=`) | **PASS** + `EXIT:0` (`e2e-scanfilter.log`) |
| Control | the same rig on the Nightly without the package | **FAIL** + `EXIT:1` (`e2e-scanfilter-control.log`) |
| Selftest | `SCAN_SELFTEST=1` (the `dirt` query must not have listed the stone) | **FAIL** + `EXIT:1` (`e2e-scanfilter-selftest.log`) |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=25  ok=25`, exit 0 |
| Package gate | `verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 |
| Skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS |

**0.2.9 — the scanner's insert-from-screen path (2026-11-22, pin `0a875da`)**

| Step | Command | Result |
|---|---|---|
| Red proof | `cargo test` with `insert_price`/`insert_amount`/`insert_charge` and the four lines stubbed | **FAILED. 37 passed; 2 failed**, `TEST_EXIT:101` (`slice9-red.log`); quoted above |
| Runtime + glue tests | `cargo test` | **39 passed**, 0 failed (`sdk-test.log`, `slice9-test.log`, `TEST_EXIT:0`) — 35 runtime + 4 glue |
| Lints | `cargo clippy --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-clippy.log`, `slice9-clippy.log`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` (`slice9-fmt.log`) |
| SDK gate | `cargo horizon-mod check` | `TREE_CHECK_EXIT:0` — *"…uses 533 items of the API (5 Part kinds, 2 capabilitys, 147 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 163 methods, 29 traits, 115 types). Made for 0a875da."* (`slice9-tree-check.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **86,850 B**, SHA-256 `f6db89ae783957109af86c926f3a8db592401b77f39cce7eed5a2763189e46ef`, **`cmp`-identical** (`slice9-build-1.log`, `slice9-build-2.log`) |
| Package, reproduced | a third build of the delivered tree, `cmp` against the delivered file (2026-11-23) | same **86,850 B** / SHA-256 `f6db89ae…46ef`, `CMP:identical` (`slice9-rebuild-cmp.log`) |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.9 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **86,850 bytes**, `f6db89ae…46ef`, `FILE_CHECK_EXIT:0` (`sdk-check.log`, `slice9-check.log`) |
| Entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`sdk-sha256.txt`) |
| Fits the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, `FITS_EXIT:0` (sha `f6db89ae…46ef` in the log) (`sdk-fits.log`, `slice9-fits.log`) |
| Fits another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, `FITS_EXIT:0` (`sdk-fits.log`, `slice9-fits.log`) |
| Launcher build | `BUILD_LOCAL=1 launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, build tag `native-0a875da-aa79a9d1a5ef` (the log's tag; an earlier build of the same sources reported `dec52470e705`), `LAUNCHER_BUILD_EXIT:0` (`slice9-launcher-build.log`) |
| In game | `build-slot.sh run --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` (`CONTROL_GAME=`) | **PASS** + `EXIT:0` (`e2e-insert.log`) |
| Control | the same rig on the Nightly without the package | **FAIL** + `EXIT:1` (`e2e-insert-control.log`) |
| Selftest | `INSERT_SELFTEST=1` (the insert must not have moved the stone) | **FAIL** + `EXIT:1` (`e2e-insert-selftest.log`) |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=22  ok=22`, exit 0 |
| Package gate | `verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 |
| Skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS |

**0.2.8 — the scanner's give-from-screen path (2026-11-22, pin `0a875da`)**

| Step | Command | Result |
|---|---|---|
| Red proof | `cargo test` with `give_price`/`give_amount`/the two lines stubbed | **FAILED. 35 passed; 2 failed**, `TEST_EXIT:101` (`slice8-red.log`); quoted above |
| Runtime + glue tests | `cargo test` | **37 passed**, 0 failed (`sdk-test.log`, `slice8-test.log`, `TEST_EXIT:0`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-clippy.log`, `slice8-clippy.log`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` (`slice8-fmt.log`) |
| SDK gate | `cargo horizon-mod check` | `TREE_CHECK_EXIT:0` — *"…uses 531 items of the API (5 Part kinds, 2 capabilitys, 146 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 162 methods, 29 traits, 115 types). Made for 0a875da."* (`slice8-tree-check.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **83,938 B**, SHA-256 `150e9784930d2ce7e5ee010219587338bee217b188f37d77ec63e4a6da4ddec8`, **`cmp`-identical** (`slice8-build-1.log`, `slice8-build-2.log`) |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.8 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **83,938 bytes**, `150e9784…ddec8`, `CHECK_EXIT:0` (`sdk-check.log`, `slice8-check.log`) |
| Entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`2bf370d9a5a9`) (`sdk-sha256.txt`) |
| Fits the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, `FITS_PIN_EXIT:0` (`sdk-fits.log`, `slice8-fits.log`) |
| Fits another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, `FITS_OTHER_EXIT:0` (`sdk-fits.log`, `slice8-fits.log`) |
| Launcher build | `BUILD_LOCAL=1 launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, build tag `native-0a875da-6ba7644ca185`, `LAUNCHER_BUILD_EXIT:0` (`slice8-launcher-build.log`) |
| In game | `build-slot.sh run --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` | **PASS** + `EXIT:0` (`e2e-give.log`) |
| Control | the same rig on the Nightly without the package | **FAIL** + `EXIT:1` (`e2e-give-control.log`) |
| Selftest | `GIVE_SELFTEST=1` (the give must not have handed the stone over) | **FAIL** + `EXIT:1` (`e2e-give-selftest.log`) |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=19 ok=19`, exit 0 |
| Package gate | `verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 |
| Skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS |

**0.2.6 — the scanner's item slots and its inject tick (2026-11-22, pin `0a875da`)**

| Step | Command | Result |
|---|---|---|
| Red proof | `cargo test` with `inject_plan`/`injectable_at` stubbed | **FAILED. 29 passed; 2 failed**, `TEST_EXIT:101` (`slice6-red.log`); quoted above |
| Runtime + glue tests | `cargo test` | **32 passed**, 0 failed (`sdk-test.log`, `TEST_EXIT:0`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-clippy.log`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` (`sdk-test.log`) |
| SDK gate | `cargo horizon-mod check` | EXIT 0 — *"…uses 524 items of the API (… 18 hooks …). Made for 0a875da."* (`sdk-check.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **78,799 B**, SHA-256 `5a034f0ac023bb31dd107c98c439943842a8a722a6f94d9917006df43156996b`, **`cmp`-identical**; 67 entries, no `Cargo.lock` (`slice6-build.log`, `sdk-check.log`) |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.6 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **78,799 B**, `5a034f0a…996b`, `FILE_CHECK_EXIT:0` (`sdk-check.log`) |
| Fits the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da91dbb…` | `0a875da  made for`, `FITS_PIN_EXIT:0`, sha `5a034f0a…996b` (`sdk-fits.log`) |
| Fits another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8acfd…` | `0b14f8a  fits`, `FITS_OTHER_EXIT:0`, sha `5a034f0a…996b` (`sdk-fits.log`) |
| Launcher build | `launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, EXIT 0 (`slice6-launcher-build.log`) |
| In game | `build-slot.sh run --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` | **PASS** + `EXIT:0` (`e2e-inject.log`) |
| Control | the same rig on the Nightly without the package | **FAIL** + `EXIT:1` (`e2e-inject-control.log`) |
| Counterfactual | a vanilla barrel where the scanner would be | **PASS** + `EXIT:0` (`e2e-inject-counterfactual.log`) |
| Selftest | `SELFTEST=1` (asserts the tick did not move the stone) | **FAIL** + `EXIT:1` (`e2e-inject-selftest.log`) |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=12 ok=12`, exit 0 |
| Package gate | `verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 |
| Skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS |

**0.2.5 — the storage scanner (2026-11-21, pin `0a875da`)**

| Step | Command | Result |
|---|---|---|
| Red proof | `cargo test` with the scanner's new rules stubbed | **FAILED. 24 passed; 5 failed**, `TEST_EXIT:101` (`slice5-red.log`); quoted above |
| Runtime + glue tests | `cargo test` | **29 passed**, 0 failed (`sdk-test.log`, `TEST_EXIT:0`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-clippy.log`) |
| Formatting | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` (`sdk-test.log`) |
| SDK gate | `cargo horizon-mod check` | EXIT 0 — *"…uses 498 items of the API (… 16 hooks …). Made for 0a875da."* (`sdk-check.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **74,447 B**, SHA-256 `48a86037…6776`, **`cmp`-identical**; 67 entries, no `Cargo.lock` (`sdk-build.log`, `sdk-sha256.txt`) |
| Package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.5 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **74,447 B**, `48a86037…6776`, EXIT 0 (`sdk-fits.log`) |
| Fits the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da91dbb…` | `0a875da  made for`, EXIT 0, sha `48a86037…6776` (`sdk-fits.log`) |
| Fits another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8acfd…` | `0b14f8a  fits`, EXIT 0, sha `48a86037…6776` (`sdk-fits.log`) |
| In game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` | **PASS** + `EXIT:0` (`e2e-scanner.log`) |
| Control | the same rig on the Nightly without the package | **FAIL** + `EXIT:1` (`e2e-scanner-control.log`) |
| Counterfactual | a vanilla barrel where the scanner would be | **PASS** + `EXIT:0` (`e2e-scanner-counterfactual.log`) |
| Selftest | `SELFTEST=1` (asserts the scan did not find the stone) | **FAIL** + `EXIT:1` (`e2e-scanner-selftest.log`) |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=8 ok=8`, exit 0 |
| Package gate | `verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 |
| Skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS |

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
