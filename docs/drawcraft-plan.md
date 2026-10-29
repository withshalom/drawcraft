# DRAWcraft: plan for a CorelDRAW-class product on VectorCraft

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** minor (`main` is now the DRAWcraft app and publishes the web build; two repositories: product here, features on `withshalom/vectorcraft`; DRAWcraft spelling; `web` branch; previously: section 7, what's built: branding, icon, `.drawcraft` saving, pages first slice, web build; previously: the page model is an artboard plus a side table, not a second rectangle type; owner's decisions D1 and D2 recorded; P0 rules added; the "no version bump" format rule and our own command names corrected from the first version)

Branch `drawcraft` of `withshalom/drawcraft`. Written 2026-10-28. A plan and a list of decisions for the owner.

**Two repositories (owner, 2026-10-28), the same branches in each.** The features are built conservatively, in
VectorCraft's own terms and with `.vectorcraft` files, on `withshalom/vectorcraft`, so they can go to VectorCraft
as pull requests: `draw-mode` (pages, master layers, draw-mode tools and layout, dimensioning, from `main`),
`cdr` (`.cdr` reading, then writing, from `draw-mode`) and `web` (`draw-mode` published). Their plan is
`docs/draw-mode.md` there. `withshalom/drawcraft` is the product, the same work under DRAWcraft's identity:

| Branch | What it is |
|---|---|
| `main` | the DRAWcraft app: upstream's `main` plus DRAWcraft's identity (name, logo, icon, `.drawcraft` files, README) and the Pages workflow; published at <https://withshalom.github.io/drawcraft/> |
| `drawcraft` | the identity commits on their own, on top of upstream's `main` (`main` was moved up to here) |
| `draw-mode` | `drawcraft` with the vectorcraft repo's `draw-mode` merged |
| `cdr` | `draw-mode` here with the vectorcraft repo's `cdr` merged |
| `web` | `draw-mode` here plus the Pages workflow: <https://withshalom.github.io/drawcraft/> | Claims about the `.cdr` format and about libcdr come from public
package listings and general knowledge and are marked **unverified** until checked against a primary source.

## 1. Goal and what it does not mean

Goal: a vector application in the CorelDRAW class (page-based layout, interactive object effects, production
output for print, signage, engraving and cutting) that opens `.cdr` files, keeps everything VectorCraft already
reads and writes (SVG, PDF, `.ai`, EPS, EMF/WMF, DXF, PSD, Affinity), and adds to it.

It does not mean copying Corel's look, icons, wording or presets, and it does not mean writing native `.cdr`
files (section 5). "Opens CorelDRAW files" is accurate and safe wording; "open-source CorelDRAW" suggests an
affiliation and uses Corel's trademark as a product name. Corel's names appear in docs and as file-format names
only. Everything a user reads in the app (command labels, menus, panels, params docs) uses our own words, and
`cargo xtask brands` enforces it, as it does for Adobe's names. So the features in section 3 get our own labels
(for example "Contain in Shape" for PowerClip); the Corel names are used there only to say what is meant.

## 2. Decisions needed before any code

| # | Decision | Recommendation |
|---|---|---|
| D1 | **Name.** DrawCraft was upstream's name for this app until 2026-10-01; `.drawcraft` files and `"format": "drawcraft"` headers still open there (`LEGACY_EXTENSION`). | **Decided (owner, 2026-10-28): the product is DRAWcraft and its extension is `.drawcraft`.** It reuses upstream's legacy name on purpose. Consequences are in D2 and D7. The GitHub repository keeps its name (`withshalom/vectorcraft`) until the owner says otherwise. |
| D2 | **Native format.** | **Decided: `.drawcraft` is the same format as `.vectorcraft`**, not a second one. Upstream's reader opens both names at every version and decides by content, so one reader handles upstream's old `.drawcraft` files (no pages: artboards mode) and ours. **Never bump `VERSION` (4):** upstream's reader refuses a file whose version is newer than it knows, so a bump would lock upstream's VectorCraft out of our files. DRAWcraft's additions are optional document fields that are written only when used; a document without pages saves exactly as before, and upstream's reader keeps what it doesn't know in `Document::extra` instead of dropping it. Open details: which `format` header and `generator` string our files carry (`"drawcraft"` makes upstream treat the file as old and rewrite it as `.vectorcraft` on save), and the default Save As extension. |
| D3 | **Crate and branch layout.** | Don't rename `vectorcraft-*` crates (the CADCraft rename to `cadcraft-model` was cancelled for the same reason: it split names across branches and made every upstream merge conflict). New work goes in new crates and new files beside upstream's. |
| D4 | **`.cdr` source of truth and licence.** | See section 4. Owner's call. |
| D5 | **Pages.** VectorCraft has `Document.artboards` (Illustrator model). CorelDRAW has pages with master layers. | Add pages beside artboards, as a document-level mode, not a rewrite. See section 3. |
| D6 | **Upstream.** | Anything that is not specific to Corel (pages, Shaping commands, dimension tools, `.cdr` reader if its licence allows) is built on `withshalom/vectorcraft` (`draw-mode`, `cdr`) and offered as its own PR; `drawcraft` here is the integration branch that merges it. |
| D7 | **Branding of builds.** The ArtCraft name, wordmark and logos in `docs/brand/` are trademarks, not open source (`docs/brand/LICENSE-brand.txt`): they may not be modified or used outside VectorCraft. | A released DRAWcraft build can't carry them. Before the first release: our own name, icon and About window, and a decision on "built by the ArtCraft team" wording. The packaging identifiers (`ai.storyteller.vectorcraft`, MIME type, file associations) need DRAWcraft's own, or installing both apps makes them fight over `.drawcraft`. Not blocking P1. |

## 3. Product gaps against a CorelDRAW-class app

What VectorCraft has today (from `docs/file-format-parity.md` and the crate list): Bézier and Pathfinder
(`pathops`), type (`text`), gradients and meshes, effects (`effects`), image trace (`trace`), PDF/SVG/EPS/EMF/DXF
(`pdf`, `svg`, `eps`, `metafile`, `cad`), a wasm plug-in host, an MCP server and a command-first engine.

Gaps, in the order they define the product:

1. **Pages and master layers.** Multi-page documents with a page sorter, per-page size and orientation, master
   layers shown on every page, a desktop (off-page) layer, guides and grid layers.
2. **Shaping.** Weld, Trim, Intersect, Simplify, Front Minus Back, Back Minus Front, Boundary, as commands over
   `pathops` (mostly renamed Pathfinder operations plus Boundary and "keep original" options).
3. **PowerClip.** A container object whose contents are edited in place and can be extracted; today's clipping
   mask is the nearest equivalent.
4. **Interactive effects as live objects.** Contour, Envelope, Extrude, Distort, Drop Shadow, Lens, Transparency,
   Interactive Fill. Check each against `effects` before building.
5. **Dimension and connector tools.** Parallel, horizontal, vertical, angular and callout dimensions and
   connector lines. This is where `crates/cad` pays off: it already writes DXF and can carry dimensions.
6. **Production output.** Print Merge, bleed and registration marks, imposition, cut contours, hairline strokes,
   contour-cut paths, PLT/HPGL for plotters and cutters, laser and engraving output, barcode and QR objects,
   spot colour and palette handling. Signage, vinyl, laser and engraving shops are CorelDRAW's strongest market;
   a vector tool that is also CAD-aware (DXF, units, precision) is a real differentiator.
7. **Object Manager, Object Data, Styles and Color Styles**, a table tool, text frames and Fit Text to Path.
8. **A "Draw" workspace profile**: menu paths, tool names, shortcuts and panel layout. Every behaviour is a
   command with its own menu path and shortcut (`CLAUDE.md`), so a profile is data that remaps them. The
   Illustrator-style layout stays the default; the Draw profile is opt-in. Shortcuts and tool names are
   functional; icons and artwork are never copied.

Page model (D5), concretely, after counting the code (2026-10-28: `artboards` appears about 380 times in the
engine, 330 in the UI, 54 in `eps`, 48 in `doc`, 43 in `tools`, 38 in `pdf`; art belongs to an artboard by where
it lies, `Document::art_on_artboard`, not by a parent link): **a page is an artboard.** Don't add a second
rectangle type that all of that code would have to learn. Add one optional side table to `Document`, keyed by the
existing artboard and layer ids (as `artboard_ruler_origins` already is): the page order, which top-level layers
are master layers, and which layers belong to one page only. A master layer's art is drawn and exported on every
page, shifted by that page's offset from the first page. A document without the table is an ordinary artboards
document and saves as before; so the exporters (PDF, SVG, EPS) and every artboard command keep working on day
one, and are then taught what a master layer is. No format version bump (D2): the new fields are optional and written only
when a document uses pages. Files in Pages mode open in upstream's VectorCraft as their art with the page data
kept unread; that has to be proved by a test, not assumed.

## 4. Opening `.cdr` files

**What is known (unverified, check before relying on it).**
- Corel doesn't publish the `.cdr` specification. Older versions (up to roughly X3) are RIFF-based; X4 and later
  use a ZIP container holding RIFF-style data, an XML part and a thumbnail. Check this against libcdr's source
  and Document Liberation Project notes before writing any code.
- The one widely used open reader is **libcdr** (Document Liberation Project; used by LibreOffice, Inkscape and
  others). The Alpine package lists it as licence **MPL-2.0**, "import filter and tools for Corel DRAW files",
  and the project describes itself as reverse engineered. Which versions it reads, and how completely, is
  **unverified**.
- Corel's EULA very likely forbids reverse engineering, as Adobe's does; the exposure is in how the work is
  done, not in the file bytes of a user's own document. This is a question for a lawyer, not this plan.

**Options.**

| Option | What it is | Verdict |
|---|---|---|
| A. Port libcdr's logic to Rust as its own crate | A derivative of MPL-2.0 code: the translated files stay MPL-2.0, the rest of the workspace stays MIT OR Apache-2.0 (MPL is file-level copyleft; credit in `NOTICE`). Same shape as the CADCraft DWG codec port. | **Recommended**, if the owner accepts one MPL-2.0 crate in an otherwise MIT/Apache tree. Upstream may refuse it: ask before proposing it there. |
| B. Link libcdr through FFI | C++ library in the tree. | No: breaks "pure Rust", `forbid(unsafe_code)` and the wasm build. |
| C. Reverse engineer from `.cdr` files, from scratch | Study users' own files with no one else's code. | Possible, slowest, and needs files made by Corel's software. Under the clean-room rule below, only files users send or openly licensed files, never files we make with it. |
| D. Don't read `.cdr`; read Corel's exchange outputs only | Ask users to export to PDF/AI/SVG/EPS. | The fallback today, and what to document in the meantime. |

**Clean-room rules to add to `CLAUDE.md` on this branch before any `.cdr` work** (copy the Adobe rule and swap the
vendor): never run, script or screenshot CorelDRAW or any Corel application; never read anything inside a Corel
install; never copy Corel icons, palettes, presets, clipart, fonts or wording beyond feature names; test inputs
are written from scratch or are openly licensed files with a recorded source and SHA-256, fetched pinned
like the Affinity corpus (`cargo xtask corpus`), never committed as large binaries. One thing to audit: `crates/eps`
has a test (`coreldraw_files_read_as_vectors`) built around a `wCorel3Dict` procset. Check that it was written from
the PostScript spec and not from a Corel prolog, because Adobe-style "no prologs" rules apply by analogy.

**Staged reader** (each stage ends with a short report and a corpus count, like the Affinity audit):
1. **C1: identify.** Sniff the container and version, read metadata, extract the embedded thumbnail, and show a
   clear "unsupported version" message otherwise. Safe to ship early.
2. **C2: geometry.** Pages, layers, paths, rectangles, ellipses, polygons, fills (uniform, then gradients),
   strokes, dash and arrowheads, transforms. Anything unread is kept as a warning, not a crash.
3. **C3: content.** Text (artistic and paragraph), bitmaps, PowerClip, symbols, colour palettes and spot colours.
4. **C4: effects.** Blend, Contour, Envelope, Extrude, Drop Shadow, Lens, as live objects when the matching
   VectorCraft effect exists and as baked art otherwise.
5. **C5: newer versions** as they appear.

Every stage is fuzzed (`import_fuzz.rs`, `prop_*` tests) and returns errors, never panics (`CLAUDE.md`).

## 5. Writing `.cdr`

Don't, at least not natively. There is no specification, a wrong file can look valid and open wrongly, and the
legal exposure is higher for writing than reading. Instead make export to Corel-readable formats excellent
(PDF, SVG, EPS, EMF, DXF, `.ai`) and add "for CorelDRAW" export presets. Revisit only if a primary source for the
format appears.

## 6. Order of work

| Phase | Content | Size | Status (2026-10-28) |
|---|---|---|---|
| P0 | Decisions D1 to D6; clean-room rules for Corel; corpus plan; branch rules | S | done but the corpus plan |
| P1 | Pages and master layers in `vectorcraft-doc`; page sorter; multi-page PDF, SVG and print | L | first slice: see section 7 |
| P2 | `.cdr` C1 and C2 (identify, geometry) | L | not started |
| P3 | Shaping commands and PowerClip | M | not started |
| P4 | Draw workspace profile | M | not started |
| P5 | `.cdr` C3 and C4 (content, effects) | XL | not started |
| P6 | Dimension and connector tools; Object Manager, Object Data, styles | L | not started |
| P7 | Production output: cut contours, PLT/HPGL, laser, Print Merge, barcodes | L | not started |

P1 comes first because the `.cdr` reader's target is pages: reading Corel files into artboards would throw away
structure that is hard to recover. P2 can start in parallel once D4 is decided. Sizes are estimates, not
measurements.

## 7. What's built

**Branding (2026-10-28).** The product is DRAWcraft in what people see: the README, the web page, the window
title, the app bar, the About box, the macOS app menu, and every interface string that says the product's name.
That last one is one function, `vectorcraft_ui_egui::i18n::brand`: the English source strings and the 25
translation catalogs stay upstream's word for word ("VectorCraft"), and the name is swapped as text is looked up,
whole words only (a file called `VectorCraft-logo.svg` keeps its name). So upstream's string and translation
changes keep merging cleanly. The About box's credit line says the app is built on VectorCraft by the ArtCraft
team (upstream's "Part of ArtCraft" isn't true of a fork; its 24 translations were removed, so the line is
English everywhere until it is translated). Crates, binaries, packages, app ids, the preferences folder, the MCP
server's name and the PDF producer string keep upstream's names for now (D7).

**Icon.** An engraved knight's helm on DRAWcraft blue (`#2d5c9e`), generated in code by
`packaging/knight-icon.py`, in the same three-colour template as upstream's dragon; every icon file was regenerated
at its old path, so packaging is unchanged (`assets/app-icon/README.md`).

**`.drawcraft` files.** Native files save as `.drawcraft` (`vectorcraft_format::SAVE_EXTENSION`; `.vectorcraft`
still opens and can be chosen). Inside they are VectorCraft files: `"format": "vectorcraft"`, format version 4,
`"generator": "DRAWcraft <version>"`. Upstream reads them as current files. Opening a `.drawcraft` file in DRAWcraft
treats it as old only when its contents are old (version below 4, or upstream's old `"format": "drawcraft"`
header), not because of its name.

**Pages, first slice (P1).** A page is an artboard; `Document::pages` (`vectorcraft_doc::pages`) is an optional
table of the page order and the master layers, written only when a document uses pages:

- `pages.enable` / `pages.disable`: a document becomes a pages document and back.
- `pages.list`: the pages in order and the master layers.
- `pages.insert {after?, count?, width?, height?}`: new pages in a row right of the rightmost artboard, placed
  in the page order.
- `pages.move {page, to}`: reorder pages (the artboards stay where they are).
- `pages.setMaster {id?, master?}`: make a top-level layer a master layer, or an ordinary one (toggles with no
  `master`). A master layer's art is laid out on the first page and repeats at the same place on every other
  page, in output only: `Document::with_master_pages` makes the repeats for every exporter (through
  `bake_document`), raster export and the canvas; the document and its file hold the art once.
- Artboards deleted or added with the artboard commands leave or join the page order by themselves.

They are commands, so the command palette, MCP and the control channel reach them; they aren't in the menus
yet. Tests: `crates/doc/src/pages/tests.rs`, `crates/engine/src/cmd/pages/tests.rs`.

**Next in P1:** a Pages panel (page sorter: thumbnails, drag to reorder, add and delete), a Layout menu,
master layers in the Layers panel, per-page layers (layers shown on one page only), page numbers as a text
field that master art can hold, and the canvas laying pages out in order.

**Web build.** As on the CADCraft fork: `main` (and the `web` branch, which also carries `draw-mode`) has
`.github/workflows/web-deploy.yml`, which builds the web app with Trunk and publishes it with GitHub Pages at
<https://withshalom.github.io/drawcraft/> on every push to either; the workflow runs only in `withshalom/drawcraft`. Pages' source is GitHub Actions.

## 8. Rules for this repository

- Follow `CLAUDE.md`: clean-room, no panics, everything a command, layering, assets listed in `ASSETS.md`, one
  task id per commit, `cargo xtask ci` before every commit. (`CLAUDE.md` names a `cargo xtask cleanroom` check
  that this checkout's xtask doesn't have; the check that exists is `cargo xtask brands`, now with Corel's names.)
- `main` is DRAWcraft itself: upstream's `main` merged in regularly, plus the identity and the Pages workflow.
  Branch `drawcraft` is the identity on its own; `draw-mode` and `cdr` here merge `main` and their namesakes in
  the vectorcraft repo; `web` merges `draw-mode`. Upstream's code is never edited here except for identity.
- Features are written on the vectorcraft repo first; only DRAWcraft's identity is written here.
- Adobe compatibility is a regression gate: the existing `.ai`, EPS, PDF and SVG tests must pass unchanged on this
  branch, and the Illustrator layout stays the default workspace.
- Update `docs/file-format-parity.md` (the `CorelDRAW .cdr` row), `docs/gaps.md` and `docs/roadmap.md` when a stage lands.

## Revision history

| Date | Change |
|---|---|
| 2026-10-28 | First version: goal, decisions, gaps, `.cdr` options and stages, phases |
| 2026-10-28 | Owner decided D1 (product DRAWcraft, extension `.drawcraft`) and D2 (same format as `.vectorcraft`, no version bump). Added D7 (branding). P0: Corel clean-room rules in `CLAUDE.md` and `AGENTS.md`, Corel names in `cargo xtask brands`. Command names must be our own words. EPS test `coreldraw_files_read_as_vectors`: its procedure names are Corel-style, and the repo's shallow history can't show what it was written from; it is upstream's file, left as it is, to ask upstream about if it matters. |
| 2026-10-28 | Owner moved `main` up to the identity branch plus the Pages workflow (it had been an untouched mirror); `web` kept for `draw-mode`. |
| 2026-10-28 | Page model changed after counting artboard uses: a page is an artboard, plus an optional side table for page order, master layers and per-page layers. |
| 2026-10-28 | Section 7 (what's built): DRAWcraft branding through one name swap at lookup, the knight icon, `.drawcraft` as the save extension, the first pages slice (page table, master layers repeated in output, six commands), the GitHub Pages web build. Phase statuses added. |
| 2026-10-28 | Owner split the work into two repositories: DRAWcraft's identity on `withshalom/drawcraft` (`drawcraft`, `web`), the features on `withshalom/vectorcraft` (`draw-mode`, `cdr`). The name is spelled DRAWcraft. Web build moved to a `web` branch, as on the CADCraft fork. |
