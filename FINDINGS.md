# RFTools Storage — findings from the artefact

Ported from **RFToolsStorage 1.21-6.0.5** (`McJtyMods/RFToolsStorage`, **MIT**),
the ATM10 jar `ports/_shared/licence-cache/rftoolsstorage-1.21-6.0.5.jar`
(301 entries, 398,718 B). Upstream was read from branch `1.21_neo`, commit
`fc1b18a7f070d0030671e1db1d3aab90699fd160` (2026-06-16, `gradle.properties`
`version=1.21-6.0.5` — the jar's exact version) — behaviour only, this is a
clean-room Rust rewrite. The clone is deleted once these findings are recorded.

Everything below is cited: the jar's own data files first, the mod's source
second. Where the brief and the artefact disagree, the artefact wins (noted
under "Brief vs artefact").

## Licence

`META-INF/neoforge.mods.toml`: `license="MIT License
https://github.com/McJtyMods/RFToolsStorage/blob/1.16/LICENCE"`; the repo's
`LICENSE` is the MIT text, `Copyright (c) 2019 McJtyMods`. **Rank 0.**

Dependencies (same file): `neoforge`, **`mcjtylib`** `[1.21-9.0.21,)` and
**`rftoolsbase`** `[1.21-6.0.11,)`, both mandatory. `mcjtylib` is a library
(class `library`, `not-needed` in the ledger); `rftoolsbase` is a content parent
that this campaign has ported (`ports/rftoolsbase`, `ported-partial`).

## Jar inventory (301 entries)

| Family | Count | Names |
|---|---|---|
| blockstates | 3 | `modular_storage` (multipart), `storage_scanner` (6 variants), `crafting_manager` (single part) |
| block models | 3 + 26 | `modular_storage`, `storage_scanner`, `crafting_manager`; `storage/` holds `modular_storage` + 3 type overlays (`overlaynone`, `overlayore`, `overlaygeneric`) + 8 amount overlays (`overlayamount0..7`) + 8 remote ones (`overlayamount_remote0..7`) + `overlayamount_empty` |
| item models | 11 | `modular_storage`, `storage_scanner`, `crafting_manager`, `storage_module0..3`, `storage_module6`, `storage_control_module`, `dump_module`, `tablet_scanner` |
| textures | 11 block + 14 item + 5 gui | `machinemodularstorage`, `machinestoragescanner`, `machinecraftingmanager(+_top)`, `overlayamount(+remote)`, `overlaygeneric`, `overlaynone`, `overlayore`, `item/storage/storagemodule0..3(+6,+6active,+tablet*)`, `gui/modularstorage(+top)`, … |
| items registered in code | 12 | the 11 models above plus `filter_module`, `oredict_module`, `generic_module` (they reuse models) |
| recipes | 8 | `modular_storage`, `storage_module0..3`, `storage_scanner`, `storage_control_module`, `dump_module` |
| loot tables | 2 | `modular_storage`, `storage_scanner` (the crafting manager has none) |
| advancements | 8 | `recipes/misc/*` (recipe unlocks) |
| tags | 2 | `minecraft:mineable/pickaxe`, `minecraft:needs_iron_tool` — all three blocks |
| Patchouli manual | 7 | `assets/rftoolsbase/patchouli_books/manual/en_us/...` (categories + 7 entries) — the mod's own documentation |
| code | ~110 classes | `modules/{modularstorage,scanner,craftingmanager}`, `storage/`, `craftinggrid/`, `commands/`, `compat/` |

`modular_storage`'s blockstate is a multipart over **three** properties:
`facing` (6: north/south/west/east/up/down, the main model rotated),
`type` (3: `none`/`ore`/`generic`, a full-face overlay) and `amount` (18:
`none`, `empty`, `g0..g7`, `r0..r7`, a 1×8 bar on the front). The loot table
copies three components from the block entity: `mcjtylib:item_energy`,
`mcjtylib:item_inventory` and `rftoolsstorage:modular_storage_data`.

## Behaviour (the slice: `modular_storage`)

### Capacity — from the artefact, not the name

`StorageModuleItem` (`modules/modularstorage/items/StorageModuleItem.java:43`):

```java
public static final int[] MAXSIZE = new int[]{100, 200, 300, 500, 0, 0, -1};
```

with `STORAGE_TIER1=0 … STORAGE_TIER4=3`, `STORAGE_REMOTE=6`. So
`storage_module0` = tier 1 = **100**, `_1` = 200, `_2` = 300, `_3` = 500; the
remote module is `-1` ("legacy/internal, not part of normal progression",
per its manual entry). The manual agrees (`storagemodules.json`: "Tier 1 holds
100 stacks, Tier 2 holds 200, Tier 3 holds 300, and Tier 4 holds 500").

A "stack" is a **slot**: `GlobalStorageItemWrapper.getSlotLimit` returns 64 and
`getStackLimit = min(64, stack.getMaxStackSize())`, and `insertItem` refuses a
slot whose `ItemStack.isSameItemSameComponents` differs. So a tier-1 storage is
**100 slots, one item kind per slot, at most 64 per slot** — 6,400 items of a
64-stack item, 1,600 of a 16-stack one, 100 of a 1-stack one.

### The storage is inert until a module is in and the block is locked

`ModularStorageTileEntity` keeps the storage behind `GlobalStorageItemWrapper`,
and **both** its `insertItem` and `extractItem` start with
`if (!isLocked()) return …` — unlocked means "you may swap the module, the
storage does nothing". The module slot (`SLOT_STORAGE_MODULE = 0`) is the
mirror image: `mayPlace` is `if (isLocked()) return false`. `ModularStorageData`
defaults to `locked = false` (`ModularStorageModule`: `new ModularStorageData(false)`),
and the GUI's padlock button is what calls `setLocked(true)`.

So a fresh Modular Storage accepts no items: the player inserts a storage module
(unlocked), locks the block, and only then does it store. The block's own
tooltip says it: *"You must first insert a storage module item before you can
use it"*.

### Where the contents live

`GlobalStorageItemWrapper.createStorage()` resolves the module item's UUID
against a **world-level** `StorageHolder` (`StorageHolder.get(overworld)`), i.e.
the contents are a global map keyed by the module's UUID, saved with the
Overworld's data, not in the block entity. The block entity saves only the three
card slots (`slot0..2`), `sortMode`, `viewMode`, `groupMode`, `filter` and the
crafting grid (`ModularStorageTileEntity.loadAdditional`/`saveAdditional`).
`StorageEntry` holds `NonNullList<ItemStack> stacks` of `slots` entries, each
`Slot` + item, and `resize()` **truncates** ("when resizing to a smaller size
items might be lost").

This port keeps the same observable behaviour with a simpler representation: the
stacks are the block entity's own container slots (the pin's `Items` list, saved
with the chunk), so a relaunch keeps them, and the block's state (`module`,
`locked`) is in its compound. A module removed and re-inserted finds its contents
again (upstream: the UUID's entry is untouched). Recorded as a deviation below.

### Insert / extract / readout

Upstream's only player paths are the GUI's storage grid (a slot view over
`GlobalStorageItemWrapper`, shift-click and click) and hoppers through the
`CapType.ITEMS` capability; there is no right-click path on the block — a
`MenuProvider` block opens its menu on any face. `getNumStacks()` counts the
non-empty slots; `setLocked` writes `infoAmount = getNumStacks()` onto the module
item for its tooltip.

### The fill overlay is dead at 1.21.1

`ModularStorageBlock` declares `TYPEMODULE` and `AMOUNT`
(`ModularStorageBlock.java:28-29`) and the blockstate ships 19 overlay models,
but **nothing in 1.21-6.0.5 writes either property**:
`ModularStorageTileEntity.getRenderLevel()` is

```java
public int getRenderLevel() {
    // @todo 1.14
    return 0;
//        if (numStacks == -1 || maxSize == 0) { return -1; }
//        return (numStacks+6) * 7 / maxSize;
}
```

and there is no `setBlock`/`setValue` for `AMOUNT`/`TYPEMODULE` anywhere in the
source (only `DataGenHelper` names them, to emit the blockstate). So the overlay
never shows in 1.21.1 — the 1.12 fill bar was not ported. The port declares both
properties and carries the models (the jar's own blockstate must bind) but does
not write them: restoring the commented formula would be inventing a mechanic.
Recorded as a gap, not a "defect fixed" (the comment says it was a deliberate
`@todo`).

### Placement and block facts

`ModularStorageBlock extends BaseBlock` (McJtyLib) with no `getStateForPlacement`
of its own; the campaign's own RFToolsBase port measured the shared defaults as
hardness 2.0 / blast resistance 2.0 / correct tool / metal sound
(`ports/rftoolsbase/FINDINGS.md`, `machine_infuser`), and RFToolsStorage calls the
same `RFToolsStorage.setup.defaultProperties()` for its block item and modules.
`ironPickaxeTags()` in its datagen adds `mineable/pickaxe` + `needs_iron_tool`.
The modules are `stacksTo(1)` items.

### The rest of the mod (out of this slice)

- **`storage_scanner`** — scans nearby inventories, searches them, needs FE
  ("Infusing bonus: reduced power consumption"), has a remote/tablet variant.
- **`crafting_manager`** — the jar's own lang calls it "WIP! … This is not
  functional yet".
- **Storage tiers 2-4** (`storage_module1..3`), the **remote** module and block,
  the **type modules** (`generic`/`oredict`), the **filter module**, the **dump
  module**, the **storage control module**, the **tablet**, the **crafting
  grid**, the JEI/TOP/XNet/RFToolsUtility compat, the two commands
  (`/rftoolsstorage list|restore`) and the config.
- The GUI (search/sort/compact/cycle, the padlock button) — the pin has no
  windowed click path in a headless harness, so this slice's player surface is
  right-click, see `notes.md`.

### The GUI (read for slice 2)

`ModularStorageContainer.java` and `client/GuiModularStorage.java`:

- the container has a **storage module card slot** (`SLOT_STORAGE_MODULE = 0`, `mayPlace` is
  `if (isLocked()) return false` then "a `StorageModuleItem`"), a type-module slot
  (`SLOT_TYPE_MODULE = 1`, `@todo 1.14` — no longer supported), a filter slot (2) and the storage
  grid from index 3 (`MAXSIZE_STORAGE = 500`, "`@todo` should be max of all possible storages");
- the grid's slots are placed at `(-500, -500)` — dummy positions — because
  `GuiModularStorage` draws a **scrollable item list** (`WidgetList itemList`) with a search
  field, view/sort/group mode choices, a `ToggleButton lockButton`, a cycle and a compact
  button. Each grid `Slot.mayPlace` is `if (!isLocked()) return false;` and then
  `getItemHandler().isItemValid(index - 3, stack)`;
- `clicked()` on the module slot carries a `// @todo 1.14` for the module copy, and
  `broadcastChanges` only pushes the sort/view/group/filter/lock state to the client.

So the pin can carry the grid and the padlock (a slot layout plus a drawn button), not the list
widget: the port's deviation 2 records that.

## Brief vs artefact

| Brief says | Artefact says | Resolution |
|---|---|---|
| "4 blockstates" | **3** (`modular_storage`, `storage_scanner`, `crafting_manager`) | 3; the slice ships only `modular_storage` |
| "12 items" | 12 registered ids, 11 models | confirmed (the three type/filter modules share models) |
| "16 recipes" | **8** in `data/rftoolsstorage/recipe/` (+ 8 recipe advancements) | 8; the slice ships 2 (`modular_storage`, `storage_module0`) |
| capacity "from the jar" | `MAXSIZE = {100,200,300,500,0,0,-1}` slots × 64 | 100 slots × 64 = 6,400 for tier 1 |
| "insert/extract through right-click" | upstream opens a GUI; the only automation path is the item capability | right-click is this slice's substitution (documented in `notes.md`) |

## What the port does differently (deviations)

1. **The storage is the block entity's own slots** (100 × 64) instead of a
   world-global map keyed by the module's UUID. Same observable behaviour for
   the slice (contents survive a relaunch and a module swap); a *second* block
   holding the same module item does not share contents, which upstream's UUID
   model would.
2. **Right-click replaces the GUI**: a storage module in hand installs it, an
   empty hand extracts a stack, any other held stack inserts, sneak + empty hand
   toggles the lock, and every action answers with an action-bar readout.
3. **`type`/`amount` are declared but never written** (see above).
4. A broken block **spills** its contents as loose drops (the pin's
   `PreRemove::Contents` for a mod container) instead of carrying them in the
   item: `minecraft:copy_components` covers only vanilla block entities at this
   pin (ARCHITECTURE §3), and upstream's list names `mcjtylib:*` components this
   port does not have.
5. The loot table is a plain self-drop (no component copy, for the same reason).

## Files read

- Jar: `META-INF/neoforge.mods.toml`, `assets/rftoolsstorage/{blockstates,models,lang}/*`,
  `data/rftoolsstorage/{loot_table,recipe,tags}/*`, `data/minecraft/tags/block/*`,
  `assets/rftoolsbase/patchouli_books/manual/en_us/**`.
- Source (`1.21_neo` @ `fc1b18a7`): `ModularStorageBlock.java`,
  `ModularStorageTileEntity.java`, `ModularStorageContainer.java`,
  `client/GuiModularStorage.java`,
  `ModularAmountOverlay.java`, `ModularTypeModule.java`,
  `StorageModuleItem.java`, `StorageModuleData.java`, `ModularStorageData.java`,
  `ModularStorageModule.java`, `storage/GlobalStorageItemWrapper.java`,
  `storage/StorageEntry.java`, `storage/StorageInfo.java`, `setup/Registration.java`,
  `setup/ModSetup.java`, `RFToolsStorage.java`.
