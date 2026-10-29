# TraceDraw's `.cdr` reader in DRAWcraft

DRAWcraft opens `.cdr` and CMX files with the reader of
[TraceDraw](https://github.com/igtoth/tracedraw), an open-source vector editor (MIT OR Apache-2.0,
Copyright (c) 2026 Ighor Toth and the TraceDraw contributors). Three of its crates are copied here:

| Folder | Crate | What it is |
|---|---|---|
| `crates/tracedraw-cdr-format` | `tracedraw-cdr-format` | The file's structure: the RIFF and ZIP containers, the chunk tree, embedded fonts, compound files. No document model. |
| `crates/tracedraw-core` | `tracedraw-core` | TraceDraw's document model (pages, layers, shapes, fills, outlines, text, live effects), which the reader builds. |
| `crates/tracedraw-cdr` | `tracedraw-cdr` | The reader (and TraceDraw's writer) between the two. |

The engine maps the document it reads to a VectorCraft document (`crates/engine/src/cmd/fileio/cdr`).

## Where it comes from

- Source: TraceDraw commit `21f22934043b882128da15604d9db84150372efe` (2026-10-29, version
  0.24.0), through the `withshalom/tracedraw` fork, which DRAWcraft's repository also carries as
  the `tracedraw` branch.
- How the format knowledge was gathered: `CLEANROOM.md` in this folder, TraceDraw's own record,
  copied unchanged. It names public format notes, files their owners sent, and fidelity checks
  that people made in their own copies of the original application. DRAWcraft's agents never run
  or inspect that application (`CLAUDE.md`).

## What DRAWcraft changed

Kept as close to TraceDraw as possible, so its later fixes can be brought over by diffing against a
newer TraceDraw commit:

- `Cargo.toml` of each crate: TraceDraw's version, authors and repository; edition 2021, as TraceDraw
  writes; the workspace lints (`[lints] workspace = true`).
- Each `lib.rs`: `#![forbid(unsafe_code)]` (added to `tracedraw-core`), and three style lints allowed
  (`cognitive_complexity`, `too_many_arguments`, `wildcard_imports`). The no-panic lints apply
  unchanged: the shipped code had no `unwrap`, `expect` or `panic` to remove. In test builds only,
  `unreachable!` and four newer style lints are allowed for TraceDraw's tests.
- A `rustfmt.toml` in each folder: TraceDraw formats with rustfmt's defaults.
- Tests that need the Selawik fonts TraceDraw bundles load them from the shared font collection
  (`CRAFT_FONTS_DIR`, `tracedraw_cdr_format::test_fonts`) and skip when they are missing: no font
  file is kept in this repository. They skip until Selawik (SIL Open Font License) is in
  craft-fonts.
- `write.rs`: the file format's own media types and XML namespaces, and the XMP namespace, are
  marked `brand-ok` for `cargo xtask brands`.
- `write/tests.rs.orig`, a leftover file, is not copied.

## Bringing over a newer TraceDraw

1. `git fetch tracedraw main` (the fork) and note the new commit.
2. For each of the three crates, diff TraceDraw's folder between the old commit above and the new
   one, and apply that diff here (`git diff OLD NEW -- crates/core | git apply --directory=…`),
   resolving the few lines listed above by hand.
3. Run `cargo xtask ci`, update the commit in this file and in `Cargo.toml`, and note it in the
   revision history of `docs/draw-mode.md`.
