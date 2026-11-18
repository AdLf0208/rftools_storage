# RFTools Storage for Horizon — validation

Slice 1 (the `modular_storage` block with a tier-1 storage module).

**Version 0.2.0 is the SDK package (2026-11-18):** one flat `.hmod`, SDK nightly
`0b14f8acfd97fa2c750e42795249dcc981178ca4`, mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`. §0 is this migration's evidence.
§1–§6 are the retired 0.1.0 `.hnmod` run (pin `1d63e0a5…` / `5a64af93…`), kept
because they are the behaviour evidence: the logic behind every 0.1.0 assertion
is `src/runtime.rs`, and that file is byte-identical in both packages. The
staged ladder and the embedded/control legs no longer exist; the 0.2.0
`tools/e2e.sh` keeps the scenario, the counterfactual and the selftest, and waits
for a launcher-built game.

Every command below ran through `ports/tools/build-slot.sh` (a lease per step);
in 0.1.0 compile steps forwarded to the second machine and game legs ran locally
because the script uses `--offscreen`.

## 0. The SDK package (0.2.0) — what ran here

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

**Not run here:** the in-game e2e. There is no `horizon-launcher` on this machine
(`which horizon-launcher` is empty), so no game carrying the package can be
built; `tools/e2e.sh` is adapted and is the next run on a launcher machine. The
0.1.0 logs in §4 remain the behaviour evidence, and the rules they exercise are
the byte-identical `src/runtime.rs`.

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
