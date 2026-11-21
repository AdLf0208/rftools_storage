# Working on this native Horizon mod (SDK format)

This port is **one flat `.hmod`** in the SDK layout (mod API 0.2, 2026-11-18):
`horizon-mod.toml` is the only manifest, `src/` is the code the launcher compiles
against the pinned game build, `assets/` and `data/` are the carried content.
`mixins/`, `horizon-mixin.toml`, `horizon-native.json`, per-hook helper files,
`compatibility.json`, `rust-toolchain.toml` and the old `.hnmod` were retired
with the container — do not recreate them (the retired tree is preserved in
`../_shared/evidence/rftoolsstorage/retired-hnmod-0.1.0/`).

- `src/runtime.rs` — the pure engine (28 rules' tests: the tier table, the slot
  rules, the grid's `slot_accepts`, the padlock's `lock_press`, the reach rule, the
  carried payload's clamp, the scanner's box/radius/price/readout and the inject
  plan and walk); keep them green. It stopped being byte-identical to the retired
  `mixins/rftoolsstorage/runtime.rs` with the 2026-11-18 review fixes.
- `src/support.rs` — the game glue (the block-entity Parts, the containers, the
  menu and its screen and padlock, the click, the carried payload, the scanner's
  FE buffer, its container and its inject tick, and the readout commands). No
  business rules here: they belong in `runtime.rs` with a test.
- `src/lib.rs` — the one `#[mod_api::horizon_mod] impl Hooks`.

Validation (see `VALIDATION.md` §0; every build/test step goes through the
campaign's `ports/tools/build-slot.sh` as the lease, never a bare `cargo` outside
it):

```sh
cargo test
cargo clippy --all-targets -- -D warnings
rustfmt --edition 2024 --check src/lib.rs src/support.rs src/runtime.rs
cargo horizon-mod check                       # the SDK gate (format + manifest + code)
rm -f Cargo.lock                              # never pack a lock: it changes the artifact
cargo horizon-mod build                       # -> target/horizon-mod/rftoolsstorage.hmod
cp target/horizon-mod/rftoolsstorage.hmod ./rftoolsstorage.hmod
cargo horizon-mod check rftoolsstorage.hmod
cargo horizon-mod fits rftoolsstorage.hmod --build nightly
```

Do **not** run `cargo fmt --all`: it reaches into the vendored `.horizon/sdk`
and rewrites the SDK's own sources. The delivered `rftoolsstorage.hmod` lives in
the port root, never in `target/`. `.horizon/sdk` is fetched per clone (21 MB,
gitignored) and never packed, and it is a **single** path dependency
(`horizon-mod-api`): the launcher's native build resolves only the API's own SDK,
and a second `horizon-core` path made it fail with `No such file or directory`
(2026-11-18). Saved-compound keys go through `Compound::iter` (`support::field`).

The in-game e2e needs a **launcher-built** game carrying the package:

```sh
ports/tools/launcher-game.sh build rftoolsstorage
eval "$(ports/tools/launcher-game.sh env rftoolsstorage)"
ports/tools/build-slot.sh run --job e2e-rftoolsstorage --port ports/rftoolsstorage \
    --step e2e -- bash ports/rftoolsstorage/tools/e2e.sh
```
