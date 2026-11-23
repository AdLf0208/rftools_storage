# RFTools Storage for Horizon — validation

Slice 1 (the `modular_storage` block with a tier-1 storage module).

**Version 0.2.13 is the current SDK package (2026-11-23):** one flat `.hmod`, SDK
nightly `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`. **§0l is this slice's evidence (the
scanner's routable star list) and is where slice 12's in-game legs are**; **§0m is
slice 13's evidence (the scanner's export choice) and is where this slice's in-game
legs are**; §0k is slice
11's (the scan list's sort choice), §0j is slice 10's
(the scan's search filter), §0i is
slice 9's (the scanner's insert-from-screen path), §0h slice 8's (the
give-from-screen path), §0f slice 7's (the request path), §0e
slice 6's (the scanner's slots and inject tick), §0d slice 5's (the scanner's block
and scan), §0c slice 3's (the card slot), §0b slice 2's (the menu) and §0 the 0.2.0
migration;
§1–§6 are the retired 0.1.0 `.hnmod` run (pin `1d63e0a5…` / `5a64af93…`), kept as
history. The 0.2.x `tools/e2e.sh` runs on a launcher-built game
(`ports/tools/launcher-game.sh`).

Every command below ran through `ports/tools/build-slot.sh` (a lease per step);
in 0.1.0 compile steps forwarded to the second machine and game legs ran locally
because the script uses `--offscreen`.

## 0m. The scanner's export choice — version 0.2.13 (2026-11-23)

Slice 13: the GUI's "Export to current container" choice (`StorageScannerData.exportC`,
the saved `exportc` flag) reached as `/rftoolsstorage export <pos> <current|first>`, plus
the list's selected row (`getInventoryForBlock`'s `lastSelectedInventory`) reached as
`/rftoolsstorage select <pos> <target>`. Under `current` the tick's **GUI-slot** stack goes
only into the selected inventory and, as upstream's `injectStackInternal` returns there,
never falls through to the starred walk; `first` (the default) is that walk. The selection
is saved with the block — upstream's field is transient, so a relaunch would silently
change where items go (notes.md, deviation 11) — under the port's `selected` key, in the
`{c: [I; x, y, z]}` shape a star entry has; the mode uses upstream's own `exportc` key.
The selected row does not change the readout: `scan` still lists the whole box (the list
widget remains a gap). The slice plan, with the gap line and the upstream reading, is
`ports/_shared/evidence/rftoolsstorage/SLICE-13-PLAN.md`. Ladder and package logs:
`ports/_shared/evidence/rftoolsstorage/`; game legs:
`ports/_shared/evidence/rftoolsstorage-e2e/`.

| Step | Command | Result | Log |
|---|---|---|---|
| red proof | `cargo test` with `word`/`from_word`/`export_line`/`selection_line` stubbed, `saved` false, `from_saved` always `First`, `export_route` always `Walk` | **FAILED. 50 passed; 3 failed**, `TEST_EXIT:101` | `slice13-red.log` |
| tests | `cargo test` | `running 54 tests` → **54 passed**, 0 failed, `TEST_EXIT:0` (48 runtime + 6 glue) | `sdk-test.log`, `slice13-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | `Finished`, **CLIPPY_EXIT:0**, 0 warnings | `slice13-clippy.log` |
| format | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` after the import wrapping and one join were formatted | `slice13-fmt.log` |
| SDK gate | `cargo horizon-mod check` | **TREE_CHECK_EXIT:0** — *"… 540 items of the API (5 Part kinds, 2 capabilitys, 149 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 168 methods, 29 traits, 115 types). Made for 0a875da."* | `slice13-tree-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **98,817 B**, SHA-256 `f89f5974f411cf012a7c5a28242185e630bd6c9ffa726939863ceb1197baeb83`, **`cmp`-identical** (the packed README was corrected after the first leg; `slice13-readme-delta.log` proves that rebuild changed only `README.md`, and the three legs were re-run on the delivered file) | `slice13-build-1.log`, `slice13-build-2.log` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.13 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **98,817 bytes**, `f89f5974…eb83`, **FILE_CHECK_EXIT:0** | `slice13-check.log` |
| entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`9af88f59eccb`) | — |
| fits, the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, **FITS_PIN_EXIT:0** | `slice13-fits.log` |
| fits, another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, **FITS_OTHER_EXIT:0** | `slice13-fits.log` |
| launcher build | `BUILD_LOCAL=1 ports/tools/launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, tag `native-0a875da-f2d820a6f1ad`, **LAUNCHER_BUILD_EXIT:0** | `slice13-launcher-build.log` |
| in game | `build-slot.sh run --job e2e-rftoolsstorage --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` (`CONTROL_GAME=`) | **PASS** + `EXIT:0` | `rftoolsstorage-e2e/e2e-export.log` |
| control | the same leg on the Nightly **without** the package | **FAIL** + `EXIT:1` (`{"error":"unknown item rftoolsstorage:storage_scanner"}`) | `e2e-export-control.log` |
| selftest (this slice) | `EXPORT_SELFTEST=1` (asserts the selected chest DID take the stack) | **FAIL** + `EXIT:1` — *"the current-container export sent the 64 stone to the selected B (64) and left A empty (0) for one rfPerInsert (energy 4980) …"* | `e2e-export-selftest.log` |
| e2e gate | `python3 ports/tools/check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=36  ok=36  missing-exit=0  bad-exit=0`, exit 0 | — |
| package gate | `python3 ports/tools/verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 | — |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/rftoolsstorage --strict` | PASS | — |

The default leg's export phase (abridged; the log has every `ctl` reply):

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

The brief's rig: scanner at S, radius 2, two empty chests two blocks away — A at S+2 on x,
B at S+2 on z (B first in the x-major walk), both starred. The flag is absent on a fresh
scanner and `1b` after `export current`; the first tick puts the 64 stone in the selected
B only (A 0) for exactly one `rfPerInsert` (5,000 → 4,980); `select A` sends the next
stack to A while B keeps its 64; the mode and the selection survive quit + re-entry
(`exportc` 1b, `selected` = A) and still route; a **selected full barrel** leaves the 64
in the slot with A and B unchanged (no fall-through — a walk would have taken it); and
`export first` saves `0b` and routes that same stack to the walk's first starred cell
(B 128). `EXPORT_SELFTEST=1` inverts the first assertion and the run ends `FAIL:` +
`EXIT:1`; the first reaching run is kept as `e2e-export-first-run.log` because its extra
buffer assertion found the tick-charge trap.

## 0l. The scanner's routable star list — version 0.2.12 (2026-11-23)

Slice 12: the GUI's per-inventory star (`GuiStorageScanner.addStorageLine`'s
"Routable: Yes/No") reached as `/rftoolsstorage routable <pos> <target> <on|off>`.
The set is upstream's `routable` list in the scanner's saved compound, one
`{c: [I; x, y, z]}` compound a starred cell, and it starts **empty**
(`StorageScannerTileEntity`'s `routable = new HashSet<>()`): a fresh scanner routes
nothing until an inventory is starred, while `scan` still lists every inventory in
the box (`getInventoryForBlock`/`countItems` pass `starred = false`). Every routing
walk (`scan_targets`: the tick, the request, the give and the insert's
`checkForRoutableInventories`) filters by the set. The XNet per-path
`InventoryAccessSettings` stay a gap. The slice plan, with the gap line and the
upstream reading, is `ports/_shared/evidence/rftoolsstorage/SLICE-12-PLAN.md`.
Ladder and package logs: `ports/_shared/evidence/rftoolsstorage/`; game legs:
`ports/_shared/evidence/rftoolsstorage-e2e/`.

| Step | Command | Result | Log |
|---|---|---|---|
| red proof | `cargo test` with `is_routable`/`star`/`routable_line` stubbed and `routing_target` ignoring the star | **FAILED. 46 passed; 3 failed**, `TEST_EXIT:101` | `slice12-red.log` |
| tests | `cargo test` | `running 50 tests` → **50 passed**, 0 failed, `TEST_EXIT:0` (45 runtime + 5 glue) | `sdk-test.log`, `slice12-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | `Finished`, **CLIPPY_EXIT:0**, 0 warnings | `sdk-clippy.log`, `slice12-clippy.log` |
| format | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` after the two wrapping diffs the first check listed were formatted | `slice12-fmt.log` |
| SDK gate | `cargo horizon-mod check` | **TREE_CHECK_EXIT:0** — *"… 540 items of the API (5 Part kinds, 2 capabilitys, 149 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 168 methods, 29 traits, 115 types). Made for 0a875da."* | `slice12-tree-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **95,274 B**, SHA-256 `d194c73b31d2d3e2b3bf60e9ed3b727a2feac90e26ff33cf2234fbd3df47991b`, **`cmp`-identical** | `slice12-build-1.log`, `slice12-build-2.log` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.12 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **95,274 bytes**, `d194c73b…991b`, **FILE_CHECK_EXIT:0** | `slice12-check.log` |
| entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`7522c53c…a7c`) | — |
| fits, the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, **FITS_PIN_EXIT:0** | `slice12-fits.log` |
| fits, another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, **FITS_OTHER_EXIT:0** | `slice12-fits.log` |
| launcher build | `BUILD_LOCAL=1 ports/tools/launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, tag `native-0a875da-d4df6925a64a`, **LAUNCHER_BUILD_EXIT:0** | `slice12-launcher-build.log` |
| in game | `build-slot.sh run --job e2e-rftoolsstorage --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` (`CONTROL_GAME=`) | **PASS** + `EXIT:0` | `rftoolsstorage-e2e/e2e-routable.log` |
| control | the same leg on the Nightly **without** the package | **FAIL** + `EXIT:1` (`{"error":"unknown item rftoolsstorage:storage_scanner"}`) | `e2e-routable-control.log` |
| selftest (this slice) | `ROUTABLE_SELFTEST=1` (asserts the un-starred chest DID take the stack) | **FAIL** + `EXIT:1` — *"the un-starred chest A stayed empty (0) and the starred B took the 64 stone (energy 4980) …"* | `e2e-routable-selftest.log` |
| e2e gate | `python3 ports/tools/check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=32  ok=32  missing-exit=0  bad-exit=0`, exit 0 | — |
| package gate | `python3 ports/tools/verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 | — |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/rftoolsstorage --strict` | PASS | — |

The default leg's star phase (abridged; the log has every `ctl` reply):

```
saved routable (default): Found no elements matching routable
no star: in-slot 64, A 0, B 0, energy 0
star B: rftoolsstorage:storage_scanner: routable 12 102 3 on
star A: rftoolsstorage:storage_scanner: routable 14 102 1 on
un-star A: rftoolsstorage:storage_scanner: routable 14 102 1 off
saved routable (B only): 12, 102, 1 has the following block data: [{c: [I; 12, 102, 3]}]
B starred, A unstarred: in-slot '', A 0, B 64, energy 4980
scan with A unstarred: rftoolsstorage:storage_scanner: minecraft:dirt 32, minecraft:stone 64 (4900/50000 FE)
A re-starred, B unstarred: in-slot '', A 96, B 64, energy 4980
saved routable after the relaunch: 12, 102, 1 has the following block data: [{c: [I; 14, 102, 1]}]
after the relaunch: in-slot '', A 160, B 64, energy 4980
```

The brief's rig: scanner at S, radius 2, two empty chests two blocks away — A at
S+2 on x, B at S+2 on z (different axes, so they do not join into a double chest;
B is first in the x-major walk). The default set is empty, so the tick's one paid
attempt (`energy: 20` → 0) moves nothing; starring B and un-starring A routes the
very stack that stayed to B (64) for one `rfPerInsert` (5,000 → 4,980); A's 32
dirt, seeded after the walk, is still in the scan (the star filters routing, not
the list); re-starring A and un-starring B makes A the recipient (96) while B stays
64; and the saved list (`[{c: [I; 14, 102, 1]}]`, A only) and the walk both
survive a relaunch (A 160). `ROUTABLE_SELFTEST=1` inverts the routing assertion and
the run ends `FAIL:` + `EXIT:1`.

## 0k. The scan list's sort choice — version 0.2.11 (2026-11-23)

Slice 11: `SortingMode` reached as `/rftoolsstorage sort <pos> <mode>` — the GUI
list's sort, saved in the scanner's compound under upstream's `sortMode` key and
applied by `scan`. `Amount+`/`Amount-` sort by the merged count and are stable
(equal counts keep the walk's first-seen order); `Mod` groups the namespace then
the item id; `Name` is the id order and the default; `Tag` is
`blocked(no item-tag enumeration)` — the SDK's `Tags` service answers only
item-in-named-tag membership, not an item's tags, which `findLargestTag` needs.
The slice plan, with the gap line and the upstream reading, is
`ports/_shared/evidence/rftoolsstorage/SLICE-11-PLAN.md`. Ladder and package
logs: `ports/_shared/evidence/rftoolsstorage/`; game legs:
`ports/_shared/evidence/rftoolsstorage-e2e/`.

| Step | Command | Result | Log |
|---|---|---|---|
| red proof | `cargo test` with `description`/`serialized_name`/`from_description`/`from_saved`/`available`/`sort_found` stubbed | **FAILED. 42 passed; 4 failed**, `TEST_EXIT:101` | `slice11-red.log` |
| tests | `cargo test` | `running 46 tests` → **46 passed**, 0 failed, `TEST_EXIT:0` (42 runtime + 4 glue) | `sdk-test.log`, `slice11-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | `Finished`, **CLIPPY_EXIT:0**, 0 warnings | `sdk-clippy.log`, `slice11-clippy.log` |
| format | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` | `slice11-fmt.log` |
| SDK gate | `cargo horizon-mod check` | **TREE_CHECK_EXIT:0** — *"… 537 items of the API (5 Part kinds, 2 capabilitys, 147 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 167 methods, 29 traits, 115 types). Made for 0a875da."* | `slice11-tree-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **91,952 B**, SHA-256 `7820eeccb3d40fba47ef72041bce5c39a216d700dd1cd43756c10af7ba4979a8`, **`cmp`-identical** | `slice11-build-1.log`, `slice11-build-2.log` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.11 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **91,952 bytes**, `7820eecc…79a8`, **FILE_CHECK_EXIT:0** | `sdk-check.log`, `slice11-check.log` |
| entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`2417ab53a2fb`) | — |
| fits, the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, **FITS_PIN_EXIT:0** | `sdk-fits.log`, `slice11-fits.log` |
| fits, another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, **FITS_OTHER_EXIT:0** | `sdk-fits.log`, `slice11-fits.log` |
| launcher build | `BUILD_LOCAL=1 ports/tools/launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, tag `native-0a875da-a6e0d2c73be2`, **LAUNCHER_BUILD_EXIT:0** | `slice11-launcher-build.log` |
| in game | `build-slot.sh run --job e2e-rftoolsstorage --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` (`CONTROL_GAME=`) | **PASS** + `EXIT:0` | `rftoolsstorage-e2e/e2e-sort.log` |
| control | the same leg on the Nightly **without** the package | **FAIL** + `EXIT:1` (`{"error":"unknown item rftoolsstorage:storage_scanner"}`) | `e2e-sort-control.log` |
| selftest (this slice) | `SORT_SELFTEST=1` (asserts the `Amount-` reply did start with stone) | **FAIL** + `EXIT:1` — *"the Amount- scan started with minecraft:dirt 64 and not the stone 32 …"* | `e2e-sort-selftest.log` |
| e2e gate | `python3 ports/tools/check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=28  ok=28`, exit 0 | — |
| package gate | `python3 ports/tools/verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 | — |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/rftoolsstorage --strict` | PASS | — |

The default leg's sort phase (abridged; the log has every `ctl` reply):

```
== THE SORT CHOICE: the GUI list's sort — Amount+, Amount-, Mod, Name (Tag is blocked at this pin) ==
  chest at 18,102,-6: 32 x minecraft:stone
  chest at 19,102,-6: 64 x minecraft:dirt
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

The brief's rig: a chest at S+2 holding 32 stone, one at S+3 holding 64 dirt, the
buffer at 5,000, radius 3. The same two chests answer dirt-first under `Amount-`
and stone-first under `Amount+` (the order is the mode, not the rig); `Name`
answers the dirt first again (the differential against `Amount+`); and the third
chest's `rftoolsstorage:storage_module0` sorts after both `minecraft:` kinds under
`Mod`. The saved compound reads `AMOUNT_DESCENDING`, and the relaunch leg sets
`Amount-` on the restored search rig, quits and re-enters, and reads
`minecraft:stone 64, minecraft:dirt 32` back with `sortMode` still
`AMOUNT_DESCENDING`.

## 0j. The scan's search filter — version 0.2.10 (2026-11-23)

Slice 10: `StorageScannerTileEntity.getMatcher` / `makeSearchPredicate` reached as
`/rftoolsstorage scan <pos> [query]` — the GUI list's search box. Every
whitespace-separated word must match (upstream ANDs the predicates): `@modid` is a
**prefix** test on the item's namespace (`startsWith`), anything else a
case-insensitive contains on the item's name; an empty query matches everything.
The price is unchanged — one `rfPerRequest`, charged before the walk and also when
the query matches nothing (`empty`). The slice plan, with the gap line and the
upstream reading, is `ports/_shared/evidence/rftoolsstorage/SLICE-10-PLAN.md`.
Ladder and package logs: `ports/_shared/evidence/rftoolsstorage/`; game legs:
`ports/_shared/evidence/rftoolsstorage-e2e/`.

| Step | Command | Result | Log |
|---|---|---|---|
| red proof | `cargo test` with `matches_search`, `search_namespace`, `search_name` stubbed | **FAILED. 39 passed; 4 failed**, `TEST_EXIT:101` | `slice10-red.log` |
| tests | `cargo test` | `running 43 tests` → **43 passed**, 0 failed, `TEST_EXIT:0` (39 runtime + 4 glue) | `sdk-test.log`, `slice10-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | `Finished`, **CLIPPY_EXIT:0**, 0 warnings | `sdk-clippy.log`, `slice10-clippy.log` |
| format | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` | `slice10-fmt.log` |
| SDK gate | `cargo horizon-mod check` | **TREE_CHECK_EXIT:0** — *"… 537 items of the API (5 Part kinds, 2 capabilitys, 147 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 167 methods, 29 traits, 115 types). Made for 0a875da."* | `slice10-tree-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **88,702 B**, SHA-256 `1aadf54d967054073d7b0898e1c6b60005829440d54105bb69296543d494f868`, **`cmp`-identical** | `slice10-build-1.log`, `slice10-build-2.log` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.10 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **88,702 bytes**, `1aadf54d…f868`, **FILE_CHECK_EXIT:0** | `sdk-check.log`, `slice10-check.log` |
| entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`be8bc239…20a3`) | `sdk-sha256.txt` |
| fits, the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, **FITS_PIN_EXIT:0** (sha printed in the log) | `sdk-fits.log`, `slice10-fits.log` |
| fits, another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, **FITS_OTHER_EXIT:0** | `sdk-fits.log`, `slice10-fits.log` |
| launcher build | `BUILD_LOCAL=1 ports/tools/launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, tag `native-0a875da-7094a7593c52`, **LAUNCHER_BUILD_EXIT:0** | `slice10-launcher-build.log` |
| in game | `build-slot.sh run --job W1-rftoolsstorage --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` (`CONTROL_GAME=`) | **PASS** + `EXIT:0` | `rftoolsstorage-e2e/e2e-scanfilter.log` |
| control | the same leg on the Nightly **without** the package | **FAIL** + `EXIT:1` (`{"error":"unknown item rftoolsstorage:storage_scanner"}`) | `e2e-scanfilter-control.log` |
| selftest (this slice) | `SCAN_SELFTEST=1` (asserts the `dirt` query did list the stone) | **FAIL** + `EXIT:1` — *"the 'dirt' query listed only minecraft:dirt 32 and not the stone 64 two blocks away …"* | `e2e-scanfilter-selftest.log` |
| e2e gate | `python3 ports/tools/check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=25  ok=25`, exit 0 | — |
| package gate | `python3 ports/tools/verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 | — |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/rftoolsstorage --strict` | PASS | — |

The default leg's filter phase (abridged; the log has every `ctl` reply):

```
== THE SCAN FILTER: the GUI list's search box — every word is '@modid' (a namespace prefix) or a case-insensitive name contains ==
scan @minecraft: rftoolsstorage:storage_scanner: minecraft:dirt 32, minecraft:stone 64 (4900/50000 FE); buffer now 4900
scan dirt: rftoolsstorage:storage_scanner: minecraft:dirt 32 (4800/50000 FE); buffer now 4800
scan stone: rftoolsstorage:storage_scanner: minecraft:stone 64 (4700/50000 FE); buffer now 4700
scan @nomatch: rftoolsstorage:storage_scanner: empty (4600/50000 FE); buffer now 4600
== the unfiltered scan still lists both — the differential for the filter ==
scan (no query): rftoolsstorage:storage_scanner: minecraft:dirt 32, minecraft:stone 64 (4500/50000 FE); buffer now 4500
```

The rig is the brief's: the scanner at S with radius 3, a chest at S+2 holding 64
stone, a chest at S+3 holding 32 dirt, the buffer reset to 5,000. `@minecraft`
lists both kinds while `dirt` and `stone` each list one, the buffer falls by
exactly 100 for every query (including the `empty` one), and the unfiltered scan
after them lists both again — the differential that proves the filter did the
removal. The fail-capable assertion is the inverted leg:

```
== SCAN_SELFTEST: assert the 'dirt' query DID list the stone (must fail) ==
FAIL: the 'dirt' query listed only minecraft:dirt 32 and not the stone 64 two blocks away (rftoolsstorage:storage_scanner: minecraft:dirt 32 (4800/50000 FE)), so this inverted assertion fails as it must
EXIT:1
```

## 0i. The scanner's insert-from-screen path — version 0.2.9 (2026-11-22)

Slice 9: `StorageScannerTileEntity.injectStackFromScreen` reached as
`/rftoolsstorage insert <pos> <item> [count]` — the player's stack pushed into the
scanned inventories in scan order, `rfPerInsert` (20 FE) charged **only when the
stack emptied**, `checkForRoutableInventories` refusing a box with no inventory,
the win checked before the walk and **no out slot at all** (the path is the player's
inventory). The slice plan, with the gap line and the upstream reading, is
`ports/_shared/evidence/rftoolsstorage/SLICE-9-PLAN.md`. Ladder and package logs:
`ports/_shared/evidence/rftoolsstorage/`; game legs:
`ports/_shared/evidence/rftoolsstorage-e2e/`.

| Step | Command | Result | Log |
|---|---|---|---|
| red proof | `cargo test` with `insert_price`, `insert_amount`, `insert_charge` and the four reply lines stubbed | **FAILED. 37 passed; 2 failed**, `TEST_EXIT:101` | `slice9-red.log` |
| tests | `cargo test` | `running 39 tests` → **39 passed**, 0 failed, `TEST_EXIT:0` (35 runtime + 4 glue) | `sdk-test.log`, `slice9-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | `Finished`, **CLIPPY_EXIT:0**, 0 warnings | `sdk-clippy.log`, `slice9-clippy.log` |
| format | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` | `slice9-fmt.log` |
| SDK gate | `cargo horizon-mod check` | **TREE_CHECK_EXIT:0** — *"… 533 items of the API (5 Part kinds, 2 capabilitys, 147 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 163 methods, 29 traits, 115 types). Made for 0a875da."* | `slice9-tree-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **86,850 B**, SHA-256 `f6db89ae783957109af86c926f3a8db592401b77f39cce7eed5a2763189e46ef`, **`cmp`-identical** | `slice9-build-1.log`, `slice9-build-2.log` |
| package, reproduced | a third build of the delivered tree `cmp`ed against the delivered file (2026-11-23) | same **86,850 B** / `f6db89ae…46ef`, `CMP:identical` | `slice9-rebuild-cmp.log` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.9 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **86,850 bytes**, `f6db89ae…46ef`, **FILE_CHECK_EXIT:0** | `sdk-check.log`, `slice9-check.log` |
| entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`72720fac…d9cb`) | `sdk-sha256.txt` |
| fits, the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, **FITS_EXIT:0** (sha printed in the log) | `sdk-fits.log`, `slice9-fits.log` |
| fits, another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, **FITS_EXIT:0** | `sdk-fits.log`, `slice9-fits.log` |
| launcher build | `BUILD_LOCAL=1 ports/tools/launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, tag `native-0a875da-aa79a9d1a5ef`, **LAUNCHER_BUILD_EXIT:0** | `slice9-launcher-build.log` |
| in game | `build-slot.sh run --job W1-rftoolsstorage --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` (`CONTROL_GAME=`) | **PASS** + `EXIT:0` | `rftoolsstorage-e2e/e2e-insert.log` |
| control | the same leg on the Nightly **without** the package | **FAIL** + `EXIT:1` (`{"error":"unknown item rftoolsstorage:storage_scanner"}`) | `e2e-insert-control.log` |
| selftest (this slice) | `INSERT_SELFTEST=1` (asserts the insert did not move the stone) | **FAIL** + `EXIT:1` — *"the insert moved 64 stone from the player into the chest (player 64 -> 0, chest 0 -> 64) and charged one rfPerInsert (5000 -> 4980) …"* | `e2e-insert-selftest.log` |
| e2e gate | `python3 ports/tools/check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=22  ok=22`, exit 0 | — |
| package gate | `python3 ports/tools/verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 | — |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/rftoolsstorage --strict` | PASS | — |

The default leg's insert phases (abridged; the log has every `ctl` reply):

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

The fail-capable assertion is the inverted leg:

```
== INSERT_SELFTEST: assert the insert did NOT move the stone into the chest (must fail) ==
FAIL: the insert moved 64 stone from the player into the chest (player 64 -> 0, chest 0 -> 64) and charged one rfPerInsert (5000 -> 4980), so this inverted assertion fails as it must
EXIT:1
```

## 0h. The scanner's give-from-screen path — version 0.2.8 (2026-11-22)

Slice 8: `StorageScannerTileEntity.giveToPlayerFromScreen` reached as
`/rftoolsstorage give <pos> <item> [single]` — the **whole** `rfPerRequest` for one
item or a stack, the item's own stack size (or exactly one with `single`), the scan
walk's pull, and the stack handed straight to the player with **no out slot at all**
(no guard, no fallback). The slice plan, with the gap line and the upstream reading,
is `ports/_shared/evidence/rftoolsstorage/SLICE-8-PLAN.md`. Ladder and package logs:
`ports/_shared/evidence/rftoolsstorage/`; game legs:
`ports/_shared/evidence/rftoolsstorage-e2e/`.

| Step | Command | Result | Log |
|---|---|---|---|
| red proof | `cargo test` with `give_price`, `give_amount` and the two reply lines stubbed | **FAILED. 35 passed; 2 failed**, `TEST_EXIT:101` | `slice8-red.log` |
| tests | `cargo test` | `running 37 tests` → **37 passed**, 0 failed, `TEST_EXIT:0` | `sdk-test.log`, `slice8-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | `Finished`, **CLIPPY_EXIT:0**, 0 warnings | `sdk-clippy.log`, `slice8-clippy.log` |
| format | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` | `slice8-fmt.log` |
| SDK gate | `cargo horizon-mod check` | **TREE_CHECK_EXIT:0** — *"… 531 items of the API (5 Part kinds, 2 capabilitys, 146 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 162 methods, 29 traits, 115 types). Made for 0a875da."* | `slice8-tree-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **83,938 B**, SHA-256 `150e9784930d2ce7e5ee010219587338bee217b188f37d77ec63e4a6da4ddec8`, **`cmp`-identical** (`CMP:identical`) | `slice8-build-1.log`, `slice8-build-2.log` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.8 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **83,938 bytes**, `150e9784…ddec8`, **CHECK_EXIT:0** | `sdk-check.log`, `slice8-check.log` |
| entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`2bf370d9a5a9`) | `sdk-sha256.txt` |
| fits, the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, **FITS_PIN_EXIT:0** | `sdk-fits.log`, `slice8-fits.log` |
| fits, another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, **FITS_OTHER_EXIT:0** | `sdk-fits.log`, `slice8-fits.log` |
| launcher build | `BUILD_LOCAL=1 ports/tools/launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, tag `native-0a875da-6ba7644ca185`, **LAUNCHER_BUILD_EXIT:0** | `slice8-launcher-build.log` |
| in game | `build-slot.sh run --job W1-rftoolsstorage --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` | **PASS** + `EXIT:0` | `rftoolsstorage-e2e/e2e-give.log` |
| control | the same leg on the Nightly **without** the package | **FAIL** + `EXIT:1` (`{"error":"unknown item rftoolsstorage:storage_scanner"}`) | `e2e-give-control.log` |
| selftest (this slice) | `GIVE_SELFTEST=1` (asserts the give did not hand the stone over) | **FAIL** + `EXIT:1` — *"the give handed 64 stone to the player (chest 64 -> 0, player holds 64) and charged 100 FE (5000 -> 4900) …"* | `e2e-give-selftest.log` |
| e2e gate | `python3 ports/tools/check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=19  ok=19`, exit 0 | — |
| package gate | `python3 ports/tools/verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 | — |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/rftoolsstorage --strict` | PASS | — |

The default leg's give phases (abridged; the log has every `ctl` reply):

```
== THE GIVE PATH: /rftoolsstorage give <pos> minecraft:stone hands the stack straight to the player ==
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

The out slot is the differential: the same dirt that makes a `request` answer *"the out
slot holds minecraft:dirt; request refused"* is not even consulted by the give, which is
the `giveToPlayerFromScreen` path the slice carries. The inverted assertion:

```
== GIVE_SELFTEST: assert the give did NOT hand the stone over (must fail) ==
FAIL: the give handed 64 stone to the player (chest 64 -> 0, player holds 64) and charged 100 FE (5000 -> 4900), so this inverted assertion fails as it must
EXIT:1
```

## 0f. The scanner's request path — version 0.2.7 (2026-11-22)

Slice 7: `StorageScannerTileEntity.requestStack` reached as
`/rftoolsstorage request <pos> <item> [<count>]` — the price (an explicit count a
tenth of `rfPerRequest`, a whole stack the full price), the out slot's guard, the
scan walk's pull, the charge only when something came out, and the reference's
straight-to-inventory default (`give_stack`, as `/give`). Ladder and package logs:
`ports/_shared/evidence/rftoolsstorage/`; game legs:
`ports/_shared/evidence/rftoolsstorage-e2e/`. The slice plan, with the gap line
and the upstream reading, is
`ports/_shared/evidence/rftoolsstorage/SLICE-7-PLAN.md`.

| Step | Command | Result | Log |
|---|---|---|---|
| red proof | `cargo test` with the four request rules stubbed | **FAILED. 32 passed; 3 failed**, `TEST_EXIT:101` | `slice7-red.log` |
| tests | `cargo test` | `running 35 tests` → **35 passed**, 0 failed, `TEST_EXIT:0` | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | `Finished`, **CLIPPY_EXIT:0**, 0 warnings | `sdk-clippy.log` |
| format | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` | `slice7-fmt.log` |
| SDK gate | `cargo horizon-mod check` | **TREE_CHECK_EXIT:0** — *"… 526 items of the API (5 Part kinds, 2 capabilitys, 142 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 162 methods, 29 traits, 114 types). Made for 0a875da."* | `slice7-tree-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **82,134 B**, SHA-256 `bf1094cef2e24b7aba521f74b103b672836eddc3209d5664167942c911391057`, **`cmp`-identical** (`CMP:identical`) | `slice7-build-1.log`, `slice7-build-2.log` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.7 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **82,134 bytes**, `bf1094ce…1057`, **CHECK_EXIT:0** | `sdk-check.log` |
| entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`0b9899ac95c6`) | `sdk-sha256.txt` |
| fits, the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, **FITS_PIN_EXIT:0** | `sdk-fits.log` |
| fits, another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, **FITS_OTHER_EXIT:0** | `sdk-fits.log` |
| launcher build | `ports/tools/launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, **LAUNCHER_BUILD_EXIT:0** | `slice7-launcher-build.log` |
| in game | `build-slot.sh run --job W1-rftoolsstorage --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` | **PASS** + `EXIT:0` | `rftoolsstorage-e2e/e2e-request.log` |
| control | the same leg on the Nightly **without** the package | **FAIL** + `EXIT:1` (`{"error":"unknown item rftoolsstorage:storage_scanner"}`) | `e2e-request-control.log` |
| selftest (slice 6) | `SELFTEST=1` (asserts the tick did not move the stone) | **FAIL** + `EXIT:1` | `e2e-request-selftest.log` |
| selftest (this slice) | `REQUEST_SELFTEST=1` (asserts the request did not pull) | **FAIL** + `EXIT:1` — *"the request pulled 32 stone out of the chest (128 -> 96) and charged 10 FE (5000 -> 4990) …"* | `e2e-request-selftest-request.log` |
| e2e gate | `python3 ports/tools/check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=16  ok=16`, exit 0 | — |
| package gate | `python3 ports/tools/verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 | — |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/rftoolsstorage --strict` | PASS | — |

The default leg's transcript (abridged; the log has every `ctl` reply):

```
== THE REQUEST PATH: /rftoolsstorage request <pos> minecraft:stone 32 pulls from the chest ==
before the request: chest 128 stone, energy 5000, player stone 0
request 32 stone: rftoolsstorage:storage_scanner: pulled 32 minecraft:stone (4990/50000 FE)
after the request: chest 96 stone, energy 4990, player stone 32
== a request for a kind nothing holds refuses and charges nothing ==
request 32 dirt: rftoolsstorage:storage_scanner: nothing to pull for minecraft:dirt (4990/50000 FE)
after the refused dirt request: chest 96 stone, energy 4990, player dirt 0
== with the buffer below the price the same request is refused with the price line ==
request 32 stone at 9 FE: rftoolsstorage:storage_scanner: not enough power for 32 minecraft:stone: costs 10, have 9/50000 FE
after the 9-FE refusal: chest 96 stone, energy 9, player stone 32
== the same request through ctl command goes straight to the player too (the reference's default) ==
command request 16 stone: {"chat":["rftoolsstorage:storage_scanner: pulled 16 minecraft:stone (4990/50000 FE)"],"ok":true}
after the command request: out slot reply 'Found no elements matching Items', chest 80 stone, energy 4990, player stone 48
== counterfactual: an out slot holding another kind refuses the request and leaves the chest alone ==
request 32 stone with a dirt out slot: rftoolsstorage:storage_scanner: the out slot holds minecraft:dirt; request refused (5000/50000 FE)
after the out-slot counterfactual: chest 80 stone, energy 5000, player stone 48
```

The inverted assertion that proves the slice's own family can fail:

```
== REQUEST_SELFTEST: assert the request did NOT pull the stone (must fail) ==
FAIL: the request pulled 32 stone out of the chest (128 -> 96) and charged 10 FE (5000 -> 4990), so this inverted assertion fails as it must
EXIT:1
```

## 0e. The scanner's item slots and its inject tick — version 0.2.6 (2026-11-22)

Slice 6: `StorageScannerContainer`'s three slots (in 0, out 1, auto-in 2) and the
`StorageScannerTileEntity.tickServer` path that pushes the two input slots into the scanned
inventories, one `rfPerInsert` attempt each. Ladder and package logs:
`ports/_shared/evidence/rftoolsstorage/`; game legs: `ports/_shared/evidence/rftoolsstorage-e2e/`.
The slice plan, with the gap line and the upstream reading, is
`ports/_shared/evidence/rftoolsstorage/SLICE-6-PLAN.md`.

| Step | Command | Result | Log |
|---|---|---|---|
| red proof | `cargo test` with `inject_plan`/`injectable_at` stubbed | **FAILED. 29 passed; 2 failed**, `TEST_EXIT:101` | `slice6-red.log` |
| tests | `cargo test` | `running 32 tests` → **32 passed**, 0 failed, `TEST_EXIT:0` | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | `Finished`, **CLIPPY_EXIT:0**, 0 warnings | `sdk-clippy.log` |
| format | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` | `sdk-test.log` |
| SDK gate | `cargo horizon-mod check` | **CHECK_EXIT:0** — *"RFTools Storage uses 524 items of the API (5 Part kinds, 2 capabilitys, 141 fields or variants, 18 hooks, 52 hook arguments, 2 macros, 161 methods, 29 traits, 114 types). Made for 0a875da."* | `sdk-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **78,799 B**, SHA-256 `5a034f0ac023bb31dd107c98c439943842a8a722a6f94d9917006df43156996b`, **`cmp`-identical** (`CMP_EXIT:0`) | `slice6-build.log`, `sdk-check.log` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.6 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **78,799 bytes**, `5a034f0a…996b`, **FILE_CHECK_EXIT:0** | `sdk-check.log` |
| entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package (`1336fe99312c`) | `sdk-check.log` |
| fits, the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da91dbb…` | `0a875da  made for`, **FITS_PIN_EXIT:0**, sha `5a034f0a…996b` before and after | `sdk-fits.log` |
| fits, another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8acfd…` | `0b14f8a  fits`, **FITS_OTHER_EXIT:0**, sha `5a034f0a…996b` | `sdk-fits.log` |
| launcher build | `ports/tools/launcher-game.sh build rftoolsstorage` | `Compiling rftoolsstorage (1 of 1)`, **LAUNCHER_BUILD_EXIT:0** | `slice6-launcher-build.log` |
| in game | `build-slot.sh run --job W1-rftoolsstorage --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh` | **PASS** + `EXIT:0` | `rftoolsstorage-e2e/e2e-inject.log` |
| control | the same leg on the Nightly **without** the package | **FAIL** + `EXIT:1` (`{"error":"unknown item rftoolsstorage:storage_scanner"}`) | `e2e-inject-control.log` |
| counterfactual | a vanilla barrel where the scanner would be | **PASS** + `EXIT:0` — `scan`/`radius` answer `not a storage scanner` | `e2e-inject-counterfactual.log` |
| selftest | `SELFTEST=1` (asserts the tick did not move the stone) | **FAIL** + `EXIT:1` — *"the tick moved 64 stone from the in-slot into the chest two blocks away (energy 4980) …"* | `e2e-inject-selftest.log` |
| e2e gate | `python3 ports/tools/check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=12  ok=12`, exit 0 | — |
| package gate | `python3 ports/tools/verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 | — |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS | — |

The default leg's transcript (abridged; the log has every `ctl` reply):

```
== THE INJECT TICK: 64 stone in the in-slot goes to the chest two blocks away, for one rfPerInsert ==
radius: rftoolsstorage:storage_scanner: radius 2
before the seed: in-slot '' chest total 0 energy 5000
after the seed: in-slot  energy 4980
after 10 ticks: in-slot '', chest Items[0] = 64 x minecraft:stone, energy 4980
== counterfactual: buffer 0, another 64 stone in the in-slot -> the chest keeps 64 and the slot keeps the stone ==
buffer 0: in-slot 64, chest total 64, energy 0
== recharged, the same stack goes: the stall was the power, not the walk ==
recharged: in-slot '', chest total 128, energy 4980
== the automation input (slot 2) routes through the same tick ==
auto slot: '', chest total 160, energy 4960
== items the game puts in the container are routed too (ctl container -> the tick) ==
container path: chest 160 -> 163, energy 4960 -> 4940
== a stack nothing accepts still costs its attempt: 20 FE pays one, 19 pays none ==
the full barrel holds 64 x 27 slots of stone
20 FE: in-slot 64 dirt, energy 0
19 FE: in-slot 64 dirt, energy 19
== the walk passes a refusing inventory for the one behind it (a full barrel at S+1, a chest at S+2) ==
walk: in-slot '', barrel 64..64 stone, chest 64 x minecraft:dirt
== the out slot (1) is declared and never written by the tick ==
out slot reply: Found no elements matching Items
scanner container: {"slots":[null,null,null]}
```

The selftest keeps the scenario and inverts the slice's own assertion:

```
== THE INJECT TICK: 64 stone in the in-slot goes to the chest two blocks away, for one rfPerInsert ==
after 10 ticks: in-slot '', chest Items[0] = 64 x minecraft:stone, energy 4980
== SELFTEST: assert the tick did NOT push the stone into the chest (must fail) ==
FAIL: the tick moved 64 stone from the in-slot into the chest two blocks away (energy 4980), so this inverted assertion fails as it must
EXIT:1
```

The control leg (the same rig on the Nightly without the package) fails on the game's own
refusal, so the PASS is not vacuous:

```
control give: {"error":"unknown item rftoolsstorage:storage_scanner"}
FAIL: without the package there is no rftoolsstorage:storage_scanner to place or scan with, so nothing of this scenario can run
EXIT:1
```

## 0d. The storage scanner — version 0.2.5 (2026-11-21)

Slice 5: `rftoolsstorage:storage_scanner` — the block, its block entity, its 50,000 FE buffer and
`/rftoolsstorage scan <pos>` (the inventories in a `pos ± radius` box, one entry a kind) with
`/rftoolsstorage radius <pos> <n>` as the GUI's radius slider. Ladder and package logs:
`ports/_shared/evidence/rftoolsstorage/`; game legs: `ports/_shared/evidence/rftoolsstorage-e2e/`.

| Step | Command | Result | Log |
|---|---|---|---|
| red proof | `cargo test` with the scanner's new rules stubbed | **FAILED. 24 passed; 5 failed**, `TEST_EXIT:101` | `slice5-red.log` |
| tests | `cargo test` | `running 29 tests` → **29 passed**, 0 failed, `TEST_EXIT:0` | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | `Finished`, **CLIPPY_EXIT:0**, 0 warnings | `sdk-clippy.log` |
| format | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` | `sdk-test.log` |
| SDK gate | `cargo horizon-mod check` | **CHECK_EXIT:0** — *"RFTools Storage uses 498 items of the API (5 Part kinds, 2 capabilitys, 135 fields or variants, 16 hooks, 47 hook arguments, 2 macros, 152 methods, 28 traits, 111 types). Made for 0a875da."* | `sdk-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **74,447 B**, SHA-256 `48a860378bfafddcf78e4e5d6fcc2eb055de475bfd91713aa0bcfe4600546776`, **`cmp`-identical** | `sdk-build.log`, `sdk-sha256.txt` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.5 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **74,447 bytes**, `48a86037…6776`, **FILE_CHECK_EXIT:0** | `sdk-fits.log` |
| entries | `unzip -l rftoolsstorage.hmod` | **67 files**, no `Cargo.lock`; `src/runtime.rs` byte-identical in tree and package | `sdk-sha256.txt` |
| fits, the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da91dbb…` | `0a875da  made for`, **FITS_PIN_EXIT:0**, sha `48a86037…6776` before and after | `sdk-fits.log` |
| fits, another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8acfd…` | `0b14f8a  fits`, **FITS_OTHER_EXIT:0**, sha `48a86037…6776` | `sdk-fits.log` |
| in game | `build-slot.sh run --job e2e-rftoolsstorage --step e2e -- bash tools/e2e.sh` | **PASS** + `EXIT:0` | `rftoolsstorage-e2e/e2e-scanner.log` |
| control | the same leg on the Nightly **without** the package | **FAIL** + `EXIT:1` (`{"error":"unknown item rftoolsstorage:storage_scanner"}`) | `e2e-scanner-control.log` |
| counterfactual | a vanilla barrel where the scanner would be | **PASS** + `EXIT:0` — `scan`/`radius` answer `not a storage scanner` | `e2e-scanner-counterfactual.log` |
| selftest | `SELFTEST=1` (asserts the scan did not find the stone) | **FAIL** + `EXIT:1` — *"the scan found minecraft:stone 64 in the chest two blocks away …"* | `e2e-scanner-selftest.log` |
| e2e gate | `python3 ports/tools/check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=8  ok=8`, exit 0 | — |
| package gate | `python3 ports/tools/verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 | — |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS | — |

The default leg's transcript (abridged; the log has every `ctl` reply):

```
== a fresh scanner has no power: the scan refuses and reads nothing ==
scan: rftoolsstorage:storage_scanner: no power (0/50000 FE)
== a chest two blocks away holds 64 stone; one three away holds 32 dirt ==
  chest at 15,102,1: 64 x minecraft:stone
  chest at 16,102,1: 32 x minecraft:dirt
scan: rftoolsstorage:storage_scanner: empty (4900/50000 FE); buffer now 4900
== /rftoolsstorage radius 2 widens the box (the GUI's radius slider) ==
radius: rftoolsstorage:storage_scanner: radius 2
saved radius: 2
== THE SLICE'S ASSERTION: the scan answers the chest two blocks away (minecraft:stone 64) ==
scan: rftoolsstorage:storage_scanner: minecraft:stone 64 (4800/50000 FE); buffer now 4800
== radius 3 brings the chest three away too: both kinds, one entry each ==
scan: rftoolsstorage:storage_scanner: minecraft:dirt 32, minecraft:stone 64 (4700/50000 FE); buffer now 4700
== an emptied buffer refuses the scan with no power, and reads nothing ==
scan: rftoolsstorage:storage_scanner: no power (0/50000 FE); buffer now 0
== charged again, the same scan answers again ==
scan: rftoolsstorage:storage_scanner: minecraft:dirt 32, minecraft:stone 64 (4900/50000 FE); buffer now 4900
== a scan of a block that is not a scanner is refused, and changes nothing ==
scan of the chest: minecraft:chest: not a storage scanner
scan of the modular storage: rftoolsstorage:modular_storage: not a storage scanner
== the radius and the buffer survive a relaunch, and the scan still answers ==
relaunch: buffer 4900 -> 4900, radius 3; scan: rftoolsstorage:storage_scanner: minecraft:dirt 32, minecraft:stone 64 (4800/50000 FE)
== the scanner's loot table drops its own item on a survival break ==
break: {"drops":{"rftoolsstorage:storage_scanner":1},"picked_up":{},"items_left_on_ground":1}
== the carried storage-scanner recipe is in the table; its frame is not in this build ==
recipe give rftoolsstorage:storage_scanner: {"chat":["Unlocked 1 recipe(s) for Player83869"],"ok":true}
the scanner recipe with a stone block where rftoolsbase:machine_frame belongs: 'empty'
give rftoolsbase:machine_frame: {"error":"unknown item rftoolsbase:machine_frame"}
```

**The one miss:** the brief asked for a second leg that *crafts* the scanner's recipe. The jar's
recipe needs `rftoolsbase:machine_frame`, and this harness builds the Nightly plus **one** package;
the campaign's `rftoolsbase` port pins `0b14f8a`, so both cannot be built into one `0a875da` game.
The leg proves the recipe is loaded and parsed and that the frame's cell is what blocks it; the
coverage row is `blocked(rftoolsbase:machine_frame)`.

## 0. The SDK package (0.2.0) — the migration (history)

All steps in one lease (`--job sdk-rfs --step check`), then a repeat
`--step build`; logs in `ports/_shared/evidence/rftoolsstorage/sdk-*.log`.

| Step | Command | Result | Log |
|---|---|---|---|
| tests | `cargo test` | `running 14 tests` → **14 passed**, 0 failed (`TEST_EXIT:0`) — 13 runtime + 1 glue readout | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | `Finished`, **CLIPPY_EXIT:0**, 0 warnings | `sdk-clippy.log` |
| format | `rustfmt --edition 2024 --check src/lib.rs src/support.rs src/runtime.rs` | `FMT_EXIT:0` | `sdk-fmt.log` |
| SDK gate | `cargo horizon-mod check` | **CHECK_EXIT:0** — *"RFTools Storage uses 244 items of the API (5 Part kinds, 1 capability, 47 fields or variants, 6 hooks, 7 hook arguments, 2 macros, 83 methods, 21 traits, 72 types). Made for 0b14f8a."* + the `ModContainer` compatibility note | `sdk-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/rftoolsstorage.hmod`, **BUILD_EXIT:0** | `sdk-build.log` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.0 (native-mod, format native-1/0b14f8a/e59da90, for 26.3)` / **45,351 bytes**, SHA-256 `d3f57e1e…e1de`, unsigned / **FILE_CHECK_EXIT:0** | `sdk-file-check.log` |
| uses | `cargo horizon-mod check rftoolsstorage.hmod --uses` | 244 entries, **USES_EXIT:0** | `sdk-uses.json` |
| fits | `cargo horizon-mod fits rftoolsstorage.hmod --build nightly` | `RFTools Storage:` / `0b14f8a  made for`, **FITS_EXIT:0** | `sdk-fits.log` |
| package hash | `sha256sum rftoolsstorage.hmod` | `d3f57e1e2b7b728e6dedaacc2729704f107f54f106f1fcfbcb3b0ea62d65e1de`; a second build of the same tree produced the same bytes | `sdk-sha256.txt` |
| runtime identical | `unzip -p rftoolsstorage.hmod src/runtime.rs \| sha256sum` | `01691fc1bd47ebd4d3c2f12112068820a59123a1cfd7f12598e2924e118615f4` = the retired `mixins/rftoolsstorage/runtime.rs` (`cmp` clean) | `sdk-sha256.txt` |
| package entries | `unzip -l rftoolsstorage.hmod` | **57 files**: 48 content (`assets/`+`data/`), `horizon-mod.toml`, `Cargo.toml`, `src/{lib,runtime,support}.rs`, `uses.json`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`; no `Cargo.lock` | `sdk-sha256.txt` |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, **SKILLS GATE: PASS**, exit 0 | — |

**Superseded:** this 0.2.0 run had no in-game leg (no `horizon-launcher` was
installed then). The launcher wrapper `ports/tools/launcher-game.sh` later built
and exported a game carrying the package, and §0b has the in-game legs for
0.2.2. The 0.1.0 logs in §4 stay as history.

## 0c. The card slot — version 0.2.3 (2026-11-21)

Slice 3: the menu's **card slot** (`ModularStorageContainer.SLOT_STORAGE_MODULE`) — the module
item lives in container slot 100, so it can be put in and taken back out. Ladder and package
logs: `ports/_shared/evidence/rftoolsstorage/`; game legs:
`ports/_shared/evidence/rftoolsstorage-e2e/`.

| Step | Command | Result | Log |
|---|---|---|---|
| red proof | `cargo test` with `slot_accepts`/`card_accepts` stubbed | **FAILED. 18 passed; 3 failed**, `TEST_EXIT:101` | `slice3-red.log` |
| red proof, the detach/hide rules | the same, first design | **FAILED. 16 passed; 5 failed**, `TEST_EXIT:101` | `slice3-red-first-design.log` |
| tests | `cargo test` | `running 21 tests` → **21 passed**, 0 failed, `TEST_EXIT:0` | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | `Finished`, **CLIPPY_EXIT:0**, 0 warnings | `sdk-clippy.log` |
| format | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` | `sdk-test.log` |
| SDK gate | `cargo horizon-mod check` | **CHECK_EXIT:0** — *"RFTools Storage uses 395 items of the API (5 Part kinds, 1 capability, 101 fields or variants, 13 hooks, 33 hook arguments, 2 macros, 119 methods, 25 traits, 96 types). Made for 0a875da."* | `sdk-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **57,648 B**, SHA-256 `7e0709577e4e29761207c95b163ceb8a1e30ff0bf829bfb52f54fc3aacba2f3f`, **`cmp`-identical** | `sdk-sha256.txt` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.3 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **57,648 bytes**, `7e070957…2f3f`, **FILE_CHECK_EXIT:0** | `sdk-check.log` |
| entries | `unzip -l rftoolsstorage.hmod` | **58 files**, no `Cargo.lock`; the packaged `src/lib.rs`, `src/support.rs`, `src/runtime.rs` and `data/.../menu/modular_storage.json` are byte-identical to the tree | `sdk-sha256.txt` |
| fits, the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, **FITS_PIN_EXIT:0** | `sdk-check.log` |
| fits, another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, **FITS_OTHER_EXIT:0** | `sdk-check.log` |
| in game | `build-slot.sh run --job W1-rftoolsstorage --step e2e -- bash tools/e2e.sh` | **PASS** + `EXIT:0` | `rftoolsstorage-e2e/e2e-cardslot.log` |
| control | the same leg on the Nightly **without** the package | **PASS** + `EXIT:0` (`{"error":"unknown item rftoolsstorage:modular_storage"}`) | `e2e-cardslot-control.log` |
| counterfactual | a vanilla barrel in the same rig | **PASS** + `EXIT:0` — 27 slots, no card slot at 400, its first slot takes the stack | `e2e-cardslot-counterfactual.log` |
| selftest | `SELFTEST=1` (asserts a 27-slot menu on the storage) | **FAIL** + `EXIT:1` — *"the storage's menu shows 101 slots, not 27: the card-slot assertion is alive"* | `e2e-cardslot-selftest.log` |
| negative | `NEGATIVE=1` (asserts the card slot took the stone) | **FAIL** + `EXIT:1` — *"the card slot refused the stone (…), so this inverted assertion fails as it must"* | `e2e-cardslot-negative.log` |
| e2e gate | `python3 ports/tools/check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=11  ok=11`, exit 0 | — |
| package gate | `python3 ports/tools/verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 | — |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS | — |

The default leg's transcript (abridged; the log has every `ctl` reply):

```
== a fresh block: no module, and the readout says so ==
readout: rftoolsstorage:modular_storage: 0/0 empty no module
== the plain click opens the storage's menu: 100 storage slots and the card slot ==
panel origin 30,15; the menu shows 101 container slots; the card slot: 'empty'
== CONTROL A: the card slot refuses a stack that is not a storage module ==
the click left the open menu showing 'minecraft:stone'; the server reads: … 0/0 empty no module; stone in the inventory: 64
reopened, the card slot is 'empty'
== a storage module goes into the card slot (upstream's SLOT_STORAGE_MODULE) ==
readout: … 0/6400 empty unlocked; modules in the inventory: 0
the reopened menu's card slot: '1 rftoolsstorage:storage_module0'
== …and it comes back out through the card slot (nothing could take it out before) ==
the card slot after taking the module out: 'empty'
readout: … 0/0 empty no module; modules in the inventory: 1
== a storage module in the hand still installs (the sneak gesture, now into the slot) ==
action bar: 'Modular Storage: module installed, 100 slots'; readout: … 0/6400 empty unlocked; modules in the hand: 0
the gesture left the module in the card slot: 'rftoolsstorage:storage_module0'
== CONTROL B: unlocked, the menu's grid still refuses a stack ==
the click left the open menu showing '0' in slot 300; the server reads: … 0/6400 empty unlocked
== the panel's padlock seals it (upstream's lockButton) ==
action bar: 'Modular Storage: locked'; readout: … 0/6400 empty locked
== a locked storage stores the stack put into a grid slot ==
stored: 64 (the server); stone in the hand: 0
the re-synced grid's slot 300: '64 minecraft:stone'
== while locked, the card slot takes no module (the sneak gesture says so) ==
action bar: 'Modular Storage: locked — unlock it first'; …; modules in the hand: 1
== the module comes out of the card slot, and the block keeps its contents ==
the card slot after taking the module out: 'empty'
readout with the module out: … 0/0 empty no module; modules in the inventory: 1
== …and the module back in the card slot finds the 64 stone it left ==
readout after the module came back: … 64/6400 minecraft:stone locked
== one kind a slot: dirt is refused by the stone's slot ==
after the dirt click on the stone's slot: … 64/6400 minecraft:stone locked
…and takes the grid's last one: the re-synced grid's slot 399 holds: '64 minecraft:dirt'
…
== the contents, the lock, the grid and the module's slot survive a relaunch ==
relaunch: 6400 -> 6400 (state locked); the reopened grid shows 64 in slot 300, the card slot 'rftoolsstorage:storage_module0'
== the carried storage-module recipe crafts in a real crafting table ==
the grid's result: 'rftoolsstorage:storage_module0 x1'
the module recipe with a stick instead of iron: 'empty'
the storage recipe with a stone block where rftoolsbase:machine_frame belongs: 'empty'
PASS: … EXIT:0
```

**Not run:** a client screenshot of the card slot (`--offscreen` + `ctl screenshot`: the panel is
asserted through `ctl screen`/`ctl ui`, not by pixels), and the 0.2.2 module migration in a real
world (it needs a 0.2.2-saved block; its rule is unit-tested, see `notes.md` § Slice 3 coverage).

## 0d. The contents travel on the item — version 0.2.4 (2026-11-21)

Slice 4: a player's break that drops the block moves the container's stacks aside before the
removal and the block's own item carries them as vanilla `minecraft:block_entity_data`; placing it
back restores the contents, the lock and the card slot's module, clamped. Ladder and package logs:
`ports/_shared/evidence/rftoolsstorage/`; game legs: `ports/_shared/evidence/rftoolsstorage-e2e/`.
The 0.2.3 logs are in `retired-0.2.3/` (both evidence directories).

| Step | Command | Result | Log |
|---|---|---|---|
| red proof | `cargo test` with `carried_slot_in`/`restored_count` stubbed | **FAILED. 21 passed; 1 failed**, `TEST_EXIT:101` | `slice4-red.log` |
| tests | `cargo test` | `running 24 tests` → **24 passed**, 0 failed, `TEST_EXIT:0` | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | **CLIPPY_EXIT:0**, 0 warnings | `sdk-clippy.log` |
| format | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` | `sdk-test.log` |
| SDK gate | `cargo horizon-mod check` | **CHECK_EXIT:0** — *"RFTools Storage uses 458 items of the API (5 Part kinds, 1 capability, 123 fields or variants, 16 hooks, 47 hook arguments, 2 macros, 135 methods, 26 traits, 103 types). Made for 0a875da."* | `sdk-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | **64,513 B**, SHA-256 `a2d6add422d7442671aa9ae79d9d66a678e88141b7ed20305a8cc794f7d90d60`, **`cmp`-identical**, 58 entries, no `Cargo.lock` | `sdk-build.log`, `sdk-sha256.txt` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.4 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **64,461 bytes**, **FILE_CHECK_EXIT:0** | `sdk-check.log` |
| fits, the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da91dbb…` (the pin, by its full commit: the short name needs GitHub's release list, rate-limited at the re-run) | `0a875da  made for`, **FITS_PIN_EXIT:0**, sha `a2d6add4…d90d60` | `sdk-fits.log` |
| fits, another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8acfd…` | `0b14f8a  fits`, **FITS_OTHER_EXIT:0**, sha `a2d6add4…d90d60` | `sdk-fits.log` |
| in game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` | **PASS** + `EXIT:0` | `e2e-carried.log` |
| control | the same rig on the Nightly **without** the package | **FAIL** + `EXIT:1` (*"without the package there is no rftoolsstorage:modular_storage…"*) | `e2e-carried-control.log` |
| counterfactual | a vanilla barrel in the same rig | **PASS** + `EXIT:0` — 27 slots, its first slot takes the stack | `e2e-carried-counterfactual.log` |
| selftest | `SELFTEST=1` (asserts the break *spilled* the contents) | **FAIL** + `EXIT:1` — *"the survival break carried the storage's contents on the block's item (1 drop(s) on the ground, 0 stone spilled), so this inverted assertion fails as it must"* | `e2e-carried-selftest.log` |
| e2e gate | `check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=4  ok=4`, exit 0 | — |
| package gate | `verify-sdk-package.py rftoolsstorage` | `1/1 package(s) verified`, exit 0 | — |
| skills gate | `check-skills.py --notes ports/rftoolsstorage --strict` | `Skills line present`, PASS | — |

The default leg's new transcript (abridged; the log has every `ctl` reply):

```
== a survival break carries the storage's contents on its own item ==
readout before the break: rftoolsstorage:modular_storage: 64/6400 minecraft:stone locked
  break: {"drops":{"rftoolsstorage:modular_storage":1},"picked_up":{},"items_left_on_ground":1}
the break's drops: {"rftoolsstorage:modular_storage":1}; on the ground: 1
== the dropped storage's item carries the contents, and never Items ==
  the dropped rftoolsstorage:modular_storage: Player… has the following entity data: {components:
  {"minecraft:block_entity_data": {carried: [{count: 64, Slot: 0b, id: "minecraft:stone"},
  {count: 1, Slot: 100b, id: "rftoolsstorage:storage_module0"}], module: -1,
  id: "rftoolsstorage:storage_block_entity", locked: 1b}}, count: 1, id: "rftoolsstorage:modular_storage"}
== placing it back restores the 64 stone, the lock and the card slot's module ==
the placed block's saved data: Items[0] = 64 x minecraft:stone, Items[1] (slot 100) = rftoolsstorage:storage_module0, carried = []
readout after placing it back: rftoolsstorage:modular_storage: 64/6400 minecraft:stone locked
the reopened menu: slot 300 holds 64, the card slot 'rftoolsstorage:storage_module0'
== a doctored payload comes back clamped to the slot ==
a payload naming 200 stone places 64 and reads: rftoolsstorage:modular_storage: 64/6400 minecraft:stone locked
== counterfactual: a level-side break spills the contents and carries nothing ==
the level-side break spilled: stone 64, modules 1, storage items 1
  the spilled storage item: Player… has the following entity data: {count: 1, id: "rftoolsstorage:modular_storage"}
the placed block after a level-side break: rftoolsstorage:modular_storage: 0/0 empty no module
the same block with its module back: rftoolsstorage:modular_storage: 0/6400 empty unlocked
PASS: … EXIT:0
```

**Not run:** a real 0.2.3-saved block that a refused removal leaves with a stash (`adopt_carried`):
the recovery path shares the 0.2.2 migration's code and its rule is unit-tested; forcing a refused
`Level.removeBlock` needs a harness the campaign does not have.

## 0b. The menu — version 0.2.2 (2026-11-21)

Slice 2: the block's own menu (a 100-slot grid opened by a right click), the screen
and the padlock. Ladder and package logs: `ports/_shared/evidence/rftoolsstorage/`;
game legs: `ports/_shared/evidence/rftoolsstorage-e2e/`.

| Step | Command | Result | Log |
|---|---|---|---|
| red proof | `cargo test` with `slot_accepts`/`lock_press` stubbed | **FAILED. 15 passed; 2 failed**, `TEST_EXIT:101` | `slice2-red.log` |
| tests | `cargo test` | `running 17 tests` → **17 passed**, 0 failed, `TEST_EXIT:0` | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | `Finished`, **CLIPPY_EXIT:0**, 0 warnings | `sdk-test.log` |
| format | `rustfmt --edition 2024 --check src/{lib,support,runtime}.rs` | `FMT_EXIT:0` | `sdk-test.log` |
| SDK gate | `cargo horizon-mod check` | **CHECK_EXIT:0** — *"RFTools Storage uses 372 items of the API (5 Part kinds, 1 capability, 93 fields or variants, 12 hooks, 31 hook arguments, 2 macros, 113 methods, 24 traits, 91 types). Made for 0a875da."* | `sdk-check.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build`, twice | 53,878 B, SHA-256 `fd5d7213bc29cd1ee3fb6bacc3b16a9d5115d58f36d9793ceaa9777125c8fc82`, **`cmp`-identical** | `sdk-sha256.txt` |
| package check | `cargo horizon-mod check rftoolsstorage.hmod` | `rftoolsstorage 0.2.2 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **53,878 bytes**, **FILE_CHECK_EXIT:0** | `sdk-check.log` |
| entries | `unzip -l rftoolsstorage.hmod` | **58 files**, no `Cargo.lock`; `src/runtime.rs` in the tree and in the package are the same bytes (`3445c570…`) | `sdk-sha256.txt` |
| fits, the pin | `cargo horizon-mod fits rftoolsstorage.hmod --build 0a875da` | `0a875da  made for`, **FITS_PIN_EXIT:0** | `sdk-check.log` |
| fits, another build | `cargo horizon-mod fits rftoolsstorage.hmod --build 0b14f8a` | `0b14f8a  fits`, **FITS_OTHER_EXIT:0** | `sdk-check.log` |
| in game | `build-slot.sh run --job e2e-rftoolsstorage --step e2e -- bash tools/e2e.sh` | **PASS** + `EXIT:0` | `rftoolsstorage-e2e/e2e-menu.log` |
| control | the same leg on the Nightly **without** the package | **PASS** + `EXIT:0` (`{"error":"unknown item rftoolsstorage:modular_storage"}`) | `e2e-menu-control.log` |
| counterfactual | a vanilla barrel in the same rig | **PASS** + `EXIT:0` — a 27-slot chest menu, its slot takes the stack | `e2e-menu-counterfactual.log` |
| selftest | `SELFTEST=1` (asserts a 27-slot menu on the storage) | **FAIL** + `EXIT:1` — *"the storage's menu shows 100 slots, not 27: the grid assertion is alive"* | `e2e-menu-selftest.log` |
| e2e gate | `python3 ports/tools/check-e2e-logs.py rftoolsstorage-e2e --strict` | `logs=6  ok=6`, exit 0 | — |

The default leg's transcript (abridged; the log has every `ctl` reply):

```
== CONTROL A: the plain click opens the storage's menu, and with no module it stores nothing ==
panel origin 30,15; the menu shows 100 container slots
slot 300 after the click with no module: '0' (the stack stays on the cursor)
action bar: 'Modular Storage: no storage module'; stored: 0; stone in the hand: 64
== a storage module installs from the hand (sneak click) ==
action bar: 'Modular Storage: module installed, 100 slots'; readout: … 0/6400 empty unlocked
== CONTROL B: unlocked, the menu's grid still refuses a stack ==
slot 300 after the unlocked click: '0'
== the panel's padlock seals it (upstream's lockButton) ==
action bar: 'Modular Storage: locked'; readout: … 0/6400 empty locked
== a locked storage stores the stack put into a grid slot ==
slot 300 now: '64 minecraft:stone'
== one kind a slot: dirt is refused by the stone's slot and takes the grid's last one ==
slot 300 after the dirt click: 'minecraft:stone' (stone keeps its slot)
slot 399 (the last of the grid) now holds 64 dirt
== a click on a filled grid slot takes the stack back ==
stored: 64; stone in the inventory: 64
== fill the storage to its tier-1 capacity (6,400) and watch the grid refuse more ==
the harness filled 99 slots
readout: … 6400/6400 minecraft:stone locked
slot 300 with the storage full: '64 minecraft:stone' (nothing fits, the stack stays on the cursor)
== …and takes it again once one stack is withdrawn through the grid ==
the grid gave 64 (6400 -> 6336); the gesture took it back: stored 6400
== the contents, the lock and the grid survive a relaunch ==
relaunch: 6400 -> 6400 (state locked)
the reopened grid shows 64 in slot 300
== the carried storage-module recipe crafts in a real crafting table ==
the grid's result: 'rftoolsstorage:storage_module0 x1'
crafted modules in the inventory: 1
== control: a stick in the iron's cell, and a block where the machine frame goes, craft nothing ==
the module recipe with a stick instead of iron: 'empty'
the storage recipe with a stone block where rftoolsbase:machine_frame belongs: 'empty'
PASS: … EXIT:0
```

**Not run:** a *client* screenshot of the panel (`--offscreen` + `ctl screenshot`) —
the layout is asserted through `ctl screen`/`ctl ui` (100 container slots, the panel
origin from the window size), not by pixels. The screen's drawn padlock is proven by
the press it answers, not by its colour.

## 1. The pure runtime (TDD)

Red first: the two core rules (`Storage::insert`, `Storage::extract_first`) were
removed on purpose and the suite run — **8 of 13 failed** (`FAILED. 4 passed; 8
failed`), so the tests exercise the rules and not their absence. Restored, then:

```
cd ports/rftoolsstorage && cargo test --offline
running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored
```

```
cargo clippy --all-targets --offline -- -D warnings   # Finished, no warnings
cargo fmt --check                                    # clean
```

The run is kept at `ports/_shared/evidence/rftoolsstorage/ladder-runtime.log`
(`running 13 tests` / `test result: ok. 13 passed; 0 failed` / clippy
`Finished` / `FMT_OK`).

## 2. The staged ladder

| Step | Command | Result |
|---|---|---|
| stage | `python3 ports/rftoolsstorage/tools/horizon.py stage` | `staging_seconds` 94-102 s, 7 injections, 2 modules, sha-verified pin |
| check | `… check` | `cargo_exit 0`, **0 warnings** (`evidence/…/ladder-check.log`) |
| test | `… test` (`--lib`, filter `game::rftoolsstorage`) | `test result: ok. 17 passed; 0 failed; 1240 filtered out` (13 runtime + 4 glue; `ladder-test.log`) |
| build | `… build` (`--profile fast`) | `cargo_exit 0` (`ladder-build.log`) |

The first `check` run had four warnings (`unused import`, `unused variable`, a
test-only constructor, three unused rules); all four were fixed in the source
(`Kind::plain` moved into the tests, the `is_full_for` rule wired into the
refusal readout, the other two annotated with their reason) and the re-run is
clean.

## 3. Packages

```
content/rftoolsstorage.hmod  size=20762 sha256=51cfef738e560ca955f8e6f7c38ecee7b7dc53c85957d6042638a8ef7292f274 entries=49
rftoolsstorage.hnmod         size=29549 sha256=8c43840a410d5f2d0f75f734ccd7ddbc0826f7082f53f1e1b435301a411e8d82 entries=14
```

`cargo horizon-mod check content/rftoolsstorage.hmod` →
`rftoolsstorage 0.1.0 (mod, API 0.2, both) … no code: content only, nothing to
run`. The `.hnmod` carries `horizon-native.json`, `mixins/rftoolsstorage/`
(plan + 9 `.rs` + `LICENSE` + `LICENSE-NOTICE.md`) and
`mixins/rftoolsstorage/data/rftoolsstorage-content.hmod`, the 20,762 B built
content — one file installs both halves (the AE2 pattern).

## 4. The in-game e2e (five legs, all in `ports/_shared/evidence/rftoolsstorage/`)

Each leg is a fresh scratch home, a seeded Overworld platform at y=100, and the
block placed from its item. The scale of the assertion is the jar's own tier
table: 100 slots × 64 = 6,400.

| Leg | Log | Verdict |
|---|---|---|
| default (`--dev-mod content/`) | `e2e.log` | **PASS** + `EXIT:0` |
| embedded (`.hnmod` alone, no `--dev-mod`) | `e2e-embedded.log` | **PASS** + `EXIT:0` |
| control (embedded leg with `.horizon-cache` a plain file) | `e2e-control.log` | **PASS** + `EXIT:0` |
| counterfactual (a vanilla barrel in the same phases) | `e2e-counterfactual.log` | **PASS** + `EXIT:0` |
| `SELFTEST=1` (asserts the barrel's 64-item capacity on the storage) | `e2e-selftest.log` | **FAIL** + `EXIT:1` |

What the default leg asserts, in order:

1. the block places (`rftoolsstorage:modular_storage[amount=none,facing=south,type=none]`),
   and reads `0/0 empty no module`;
2. **control A** — with no module a stack is refused: `stored 0`, 64 stone still
   in the hand, the action bar says `no storage module`;
3. a `storage_module0` in hand installs (`module installed, 100 slots`), the
   readout becomes `0/6400 empty unlocked`, the module leaves the hand;
4. **control B** — unlocked, the same stack is refused (upstream's
   `if (!isLocked()) return stack`), still `stored 0`, 64 in the hand;
5. an empty-hand click seals it (`0/6400 empty locked`), and a right-click then
   stores the whole 64-stack (`64/6400 minecraft:stone`, hand empty);
6. `/rftoolsstorage lock` unlocks it (`unlocked`) and locks it again (`locked`),
   contents unchanged — the module-swap path;
7. a second stack takes a second slot (`128`), which a barrel cannot;
8. dirt takes its own slot (`192/6400 minecraft:stone`, 2 kinds);
9. an empty-hand click gives a stack of the first kind back
   (`took 64 minecraft:stone (128/6400 minecraft:stone)`), the stone lands in the
   inventory and the dirt is untouched;
10. filled to `6400/6400` (the harness fills the block entity's own container
    slots), a stack is refused — `full, nothing fits (6400/6400 …)`, `stored
    6400`, 64 still in the hand — and **after one stack is withdrawn**
    (`6400 → 6336`) the same click is accepted again (`6400`): the control is
    inside the same block;
11. quitting and relaunching the same world keeps `6400` and `locked`;
12. the counterfactual leg puts a vanilla barrel in the same rig: the readout is
    `minecraft:barrel: not a modular storage`, the click opens a chest menu
    instead of storing, and all 64 stone stay in the hand — so the behaviour is
    this port's code, not the scenario.

The embedded leg launches the bundle **without `--dev-mod`** and asserts the
written `.horizon-cache/rftoolsstorage-embedded-content.hmod` is byte-identical
to `content/rftoolsstorage.hmod` (20,762 B) before running the same scenario.
The control leg replaces `.horizon-cache` with a plain file so the helper cannot
write its content: the game then has **no** storage item and the block id is
unknown (`minecraft:air`), which is the positive proof that the embedded file is
what supplies them.

The selftest keeps the scenario and asserts the barrel's 64-item capacity:

```
== SELFTEST: assert the barrel's 64-item capacity on the storage (must fail) ==
FAIL: the storage holds 6400, not 64: the capacity assertion is alive
EXIT:1
```

## 5. Gates

```
python3 ports/tools/check-e2e-logs.py rftoolsstorage --strict
ok       rftoolsstorage  e2e-control.log          PASS + EXIT:0
ok       rftoolsstorage  e2e-counterfactual.log   PASS + EXIT:0
ok       rftoolsstorage  e2e-embedded.log         PASS + EXIT:0
ok       rftoolsstorage  e2e-selftest.log         FAIL + EXIT:1
ok       rftoolsstorage  e2e.log                  PASS + EXIT:0
logs=5  ok=5  missing-exit=0  bad-exit=0        # exit 0

python3 ports/tools/check-skills.py --notes ports/rftoolsstorage
ok  rftoolsstorage/notes.md: Skills line present   # SKILLS GATE: PASS, exit 0

python3 ports/tools/upstream_ready.py rftoolsstorage --include-blocked --ignore-parents
READY    rftoolsstorage    MIT                     # exit 0
```

The plain `upstream_ready.py rftoolsstorage` reports two blockers that are not
this port's to fix (and that the brief forbids editing): the ledger row is
still `pending` (the orchestrator records the status), and `rftoolsbase` — its
own upstream-declared required parent, ported here as `ported-partial` — is not
in `ports/_shared/horizon-mods/registry-ids.txt`. With those two acknowledged
(`--include-blocked --ignore-parents`), every other gate passes: licence
reviewable (jar), the `.hnmod` delivered in the port root, built after its
sources, `notes.md` with the `Skills:` line, pinned to `1d63e0a5`, and an e2e
log with `PASS + EXIT:0`.

> **0.2.0 note (2026-11-18):** `upstream_ready.py` still inspects the retired
> layout (`content/*.hmod`, `*.hnmod`, `compatibility.json`), so it reports
> `blocked` for this port even though the SDK gate is green (§0). The tool is
> campaign-level, outside this port's write ownership; the gate that applies now
> is `cargo horizon-mod check` + `fits`.

## 6. Harness facts learned here

- `right_click_block` fires **once per hand**, and the off hand's empty call
  follows the main hand's in the same click: the port ignores the off-hand call
  (0.1.0: `Game::hand == Hand::Off`; 0.2.0: `event.click.hand != Hand::Main`),
  otherwise a module install was immediately followed by a "seal" from the empty
  off hand.
- `--game-dir` makes the game its own folder (`set_current_dir`), so a mod's
  `.horizon-cache/` is inside the game dir, not the launch directory — the
  control leg has to block *that* path.
- Sneak held through `horizon-ctl` (`key shift down`) did not reach the player's
  sneak state in an offscreen run, so the lock gesture is the empty-hand click
  (and the command), not sneak-click.
- `ctl script --keep-going` still exits with the last failure; a fill script
  that runs past "container is full" needs `|| true` and an assertion on the
  readout instead.
- `ctl container X Y Z item count` writes into the first empty slot of a mod
  block entity's container — the cheapest honest way to fill 100 slots.
- **`Flow::Cancel` alone is not "handled".** The SDK's cancellation answers with
  `cancellation_result`, and the default is `Pass` — the block's own use is
  skipped but the held item's still runs (a block in hand would be placed behind
  the insert). Set `InteractionResult::Success` as well, on the client's copy too
  (the hook fires on both sides, once per hand). This port's 0.2.0 click does;
  `extradisks`, `enderstorage` and `tiab` are the same shape.
- Building anywhere but the port's own scratch (`--game-dir`) leaves the game's
  cache where the helper expects it; `find`-based probes of the cache path are
  wrong as soon as `--game-dir` is used.
