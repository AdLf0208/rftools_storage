# RFTools Storage for Horizon — validation

Slice 1 (the `modular_storage` block with a tier-1 storage module).

**Version 0.2.5 is the current SDK package (2026-11-21):** one flat `.hmod`, SDK
nightly `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`. **§0d is this slice's evidence (the
storage scanner) and is where the in-game legs are**; §0c is slice 3's (the
card slot), §0b slice 2's (the menu) and §0 the 0.2.0 migration; §1–§6 are the
retired 0.1.0 `.hnmod` run (pin `1d63e0a5…` / `5a64af93…`), kept as history. The
0.2.x `tools/e2e.sh` runs on a launcher-built game
(`ports/tools/launcher-game.sh`).

Every command below ran through `ports/tools/build-slot.sh` (a lease per step);
in 0.1.0 compile steps forwarded to the second machine and game legs ran locally
because the script uses `--offscreen`.

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
