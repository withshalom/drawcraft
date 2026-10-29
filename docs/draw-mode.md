# Draw mode and `.cdr` support

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version)

Work on the fork `withshalom/vectorcraft`, meant to go to VectorCraft as pull requests:

- **`draw-mode`** (from `main`): pages and master layers, better dimensioning, and an optional layout and set of
  tools for people coming from CorelDRAW-style apps.
- **`cdr`** (from `draw-mode`): opening CorelDRAW `.cdr` files, then writing them.

The rules for both: VectorCraft stays VectorCraft. Its name, its icon, its file format and extension
(`.vectorcraft`, format version 4) and its default Illustrator-style workspace don't change. Every change is
additive and sits beside upstream's code, so the branches merge `main` easily and each piece can become its own
pull request. (DRAWcraft, the product built on this work, lives in its own repository, `withshalom/drawcraft`,
and is never proposed upstream.)

## 1. What exists today (2026-10-28)

**Pages and master layers, first slice.** A page is an artboard. `Document::pages` (`crates/doc/src/pages.rs`) is
an optional table of the page order and the master layers, written only when a document uses pages, so a document
that doesn't saves exactly as before and no format version bump is needed. A VectorCraft without this code keeps
the table it doesn't know in `Document::extra`.

| Command | What it does |
|---|---|
| `pages.enable` / `pages.disable` | A document becomes a pages document and back (one undo step each). |
| `pages.list` | The pages in order, and the master layers. |
| `pages.insert {after?, count?, width?, height?}` | New pages in a row right of the rightmost artboard, placed in the page order, named by their numbers. |
| `pages.move {page, to}` | Reorder pages; the artboards stay where they are on the canvas. |
| `pages.setMaster {id?, master?}` | Make a top-level layer a master layer or an ordinary one (toggles without `master`). |

A master layer's art is laid out on the first page and repeats at the same place on every other page, in output
only: `Document::with_master_pages` makes the repeats for every exporter (through `bake_document`), raster export
and the canvas. The document and its file hold the art once. Artboards made or deleted with the artboard commands
join or leave the page order by themselves. Known limit: a clipping layer as a master layer clips its repeats with
the first page's clipping path.

The commands are reachable from the command palette, MCP and the control channel; there is no menu or panel yet.
Their labels are our own words (`cargo xtask brands` rejects Corel's names in user-visible strings).

Tests: `crates/doc/src/pages/tests.rs`, `crates/engine/src/cmd/pages/tests.rs`.

**Clean-room rules for Corel software** in `CLAUDE.md` and `AGENTS.md`, and Corel's names in `cargo xtask brands`.

## 2. Decisions

| # | Decision | Status |
|---|---|---|
| D1 | Draw mode is opt-in. The Illustrator-style layout, menus and shortcuts stay the default. | decided |
| D2 | No new file format and no format version bump: additions are optional document fields. | decided |
| D3 | No renaming of crates, commands or files; new work in new files beside upstream's. | decided |
| D4 | Source of `.cdr` knowledge. libcdr (Document Liberation Project) is the one widely used open reader; its licence is listed as MPL-2.0. Porting it means one MPL-2.0 crate in an MIT OR Apache-2.0 tree, credited in `NOTICE`, which upstream may refuse. The alternative is a reader written only from users' own files and openly licensed sample files. | **open: the owner decides, and asks upstream before proposing either** |
| D5 | Ask upstream before proposing pages, draw mode or `.cdr` support, and send each as a pull request of its own. | decided |

## 3. Draw mode

1. **Pages, the rest of P1:** a Pages panel (page sorter: thumbnails, drag to reorder, add, delete), a Layout
   menu in draw mode, master layers marked in the Layers panel, layers shown on one page only, page numbers as a
   text field master art can hold, the canvas laying pages out in order, multi-page PDF in page order.
2. **Shaping:** weld, trim, intersect, simplify, front minus back, back minus front and boundary as one-step
   commands over `pathops`, each with "keep originals".
3. **Contain in shape:** art placed inside a shape and edited there (the nearest today is a clipping mask).
4. **Dimensioning and connectors:** VectorCraft's Dimension tool and DXF output are the base. Add parallel,
   horizontal, vertical, angular, radius and callout dimensions with styles and units, dimensions that stay
   attached to the art they measure, connector lines that follow the objects they join, and DXF round trips of
   all of them.
5. **The draw-mode workspace:** menus, tool names, shortcuts and panel layout as data that remaps the existing
   commands (every behaviour is a command already). Our own names and icons; functional parity only.
6. **Production output:** cut contours, hairlines, PLT/HPGL for plotters and cutters, laser and engraving output,
   barcodes, print merge.

## 4. `.cdr` support (`cdr`)

Corel doesn't publish the format. What is said about it (older files RIFF, newer ones a ZIP container holding
RIFF-style data, XML and a thumbnail) is unverified until checked against a primary source. Stages, each ending
with a short report and a corpus count, as the Affinity audit does (`docs/affinity-validation.md`):

1. **C1, identify:** sniff the container and version, read the metadata and the embedded thumbnail, and refuse
   versions not read yet with a clear message. Safe to ship early.
2. **C2, geometry:** pages, layers, paths and shapes, fills, strokes, dashes, arrowheads and transforms; anything
   unread becomes a warning, never a crash.
3. **C3, content:** text, bitmaps, contained art, symbols, palettes and spot colours.
4. **C4, effects:** blends, contours, envelopes, extrusions, shadows and lenses, live where VectorCraft has the
   effect and as their drawn look otherwise.
5. **C5, writing,** after reading is solid. No open `.cdr` writer exists to learn from, so this is original work
   built from what reading teaches, and a file can look valid yet open wrongly. It is its own pull request, with
   round-trip tests (VectorCraft → `.cdr` → VectorCraft) and files people open in CorelDRAW themselves and report
   on (we never run Corel software).

Every stage is fuzzed (`engine/tests/import_fuzz.rs`) and never panics. Test files are openly licensed with a
recorded source and SHA-256, fetched pinned, never committed (`CLAUDE.md`, Shared test corpora).

## 5. Rules

- `CLAUDE.md` applies in full: clean-room for Adobe and Corel software, no panics, everything a command, layering,
  assets in `ASSETS.md`, `cargo xtask ci` before every commit.
- Merge `main` into `draw-mode`, and `draw-mode` into `cdr`, every few days.
- The existing `.ai`, EPS, PDF and SVG tests pass unchanged on both branches.

## Revision history

| Date | Change |
|---|---|
| 2026-10-28 | First version: what exists (pages and master layers, Corel clean-room rules), decisions, draw mode, `.cdr` stages with writing last |
