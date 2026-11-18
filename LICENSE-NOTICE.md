# Licence notice — RFTools Storage

- **Upstream:** RFTools Storage 1.21-6.0.5, https://github.com/McJtyMods/RFToolsStorage
  (branch `1.21_neo`, commit `fc1b18a7f070d0030671e1db1d3aab90699fd160`).
- **Licence as read from the artefact:** MIT. The jar's
  `META-INF/neoforge.mods.toml` says `license="MIT License
  https://github.com/McJtyMods/RFToolsStorage/blob/1.16/LICENCE"`, and the
  repository's `LICENSE` is the MIT text, `Copyright (c) 2019 McJtyMods`. The
  port's [`LICENSE`](LICENSE) carries that notice. **Licence rank 0.**
- **Assets carried** (from the RFTools Storage jar, MIT): the blockstate, the
  21 block models under `storage/`, the `machinemodularstorage` front texture,
  the five overlay textures, the `storagemodule0` item texture, the item and
  block models, and the lang lines the slice uses.
- **Assets carried from RFToolsBase 1.21-6.0.11** (MIT, `Copyright (c) 2019
  McJtyMods`, same author): the three machine casing textures
  `block/base/{machinebottom,machineside,machinetop}`, copied into this mod's
  own namespace so the block renders without that mod installed. RFToolsBase is
  RFTools Storage's own declared required dependency
  (`[[dependencies.rftoolsstorage]] modId="rftoolsbase"`).
- **The Rust is a clean-room rewrite** from the mod's observed behaviour, its
  own data files and its public source (analysis only, never copied or
  translated).
- **What travels with the artifact:** `LICENSE` and this notice are packaged at
  the root of the distributed `rftoolsstorage.hmod` (one flat SDK package holding
  the code and the content), so an artifact-only download carries them (MIT's
  notice condition). In the retired 0.1.0 `.hnmod` they sat at
  `mixins/rftoolsstorage/LICENSE{,-NOTICE.md}` and inside the embedded content
  archive.
- **Status:** local-only until the maintainer approves publishing
  (ARCHITECTURE.md §8).
