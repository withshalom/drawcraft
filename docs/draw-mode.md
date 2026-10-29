# Draw mode and `.cdr` support

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** minor (section 3: the page-based layout compared with the Illustrator style, from public help pages; Illustrate and Draw modes, D6; previously: first version)

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
| `pages.insert {after?, count?, width?, height?}` | New pages in a row right of the rightmost artboard, placed in the page order, named by their numbers. A document without pages becomes one in the same undo step. |
| `pages.move {page, to}` | Reorder pages; the artboards stay where they are on the canvas. A document without pages becomes one in the same undo step. |
| `pages.setMaster {id?, master?}` | Make a top-level layer a master layer or an ordinary one (toggles without `master`). |

A master layer's art is laid out on the first page and repeats at the same place on every other page, in output
only: `Document::with_master_pages` makes the repeats for every exporter (through `bake_document`), raster export
and the canvas. The document and its file hold the art once. Artboards made or deleted with the artboard commands
join or leave the page order by themselves. Known limit: a clipping layer as a master layer clips its repeats with
the first page's clipping path.

The commands are reachable from the command palette, MCP and the control channel, and from Draw mode's page bar.
Their labels are our own words (`cargo xtask brands` rejects Corel's names in user-visible strings).

**Illustrate and Draw modes** (section 3). `window.mode {mode?}` and Window › Mode, and a switch in the app bar
beside the workspace switcher. Draw mode switches to the built-in Draw workspace (Control bar, rulers, Layers in
front) and adds the page bar above the status bar (page navigation, Add Page, a tab per page: click to go, double-click
to rename, drag to reorder, a menu to insert before or after, rename, move and delete) and the swatch strip at the
window's right edge (the document's solid swatches, None first; click for the fill, right-click for the stroke).
Illustrate brings back the workspace used before. The mode is kept with the preferences. Tests:
`crates/ui-egui/src/modes/tests.rs`, `chrome/page_bar/tests.rs`, `chrome/swatch_strip/tests.rs`.

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

| D6 | Two modes, **Illustrate** (the default: VectorCraft as it is, Illustrator-style) and **Draw** (the page-based layout below), switched from the app bar or Window › Mode. The mode changes the window's arrangement and adds page-based chrome; it never changes the document, the commands, the menus' contents or the shortcuts. | decided (owner, 2026-10-28) |

## 3. Layout: how the page-based style differs from the Illustrator style

Sources are public help pages only (we never run either application). The Illustrator column is VectorCraft as it
is today, which follows Illustrator's public documentation (`docs/ui-parity.md`).

| Part of the window | Illustrator style (VectorCraft today, Illustrate mode) | Page-based style (CorelDRAW's help) | Draw mode |
|---|---|---|---|
| Document model | Artboards on one shared canvas; no master pages. | Pages in an order, each its own size and orientation; a Master Page holds master layers shown on all, odd or even pages. [Pages][c-pages], [Create layers][c-layers] | Pages are artboards plus a page order and master layers (section 1). |
| Page navigation | Status bar: First, Previous, a list of artboards, Next, Last. | A **document navigator** below the drawing window: page tabs, add-page buttons, drag tabs to reorder; right-click a tab to insert before or after, duplicate, rename, delete. [Document navigator][c-nav], [Add, duplicate, and delete pages][c-add], [Multipage view tutorial][c-multi] | A page bar above the status bar: First, Previous, "Page n of m", Next, Last, Add Page, then a tab per page; a tab's menu inserts before or after, renames, moves and deletes. |
| Page views | All artboards always shown on the canvas. | Single page view (the active page) or multipage view (grid, vertical, horizontal or custom). [Page views][c-views] | All pages shown, as now; clicking a tab fits that page. Single-page view comes later. |
| Context controls | Properties panel in the dock; the Control bar is optional. | A **property bar** under the toolbars, its controls changing with the tool and selection. [Property bar][c-prop], [Application window][c-app] | The Control bar is on. |
| Panels | Panels docked at the right, Properties | Layers | Libraries in front. | **Dockers** at the window's edges, nested as tabs, collapsible. [Dockers][c-dock] | The dock stays; Layers is in front (pages and master layers live there). |
| Colours | Swatches panel; Fill and Stroke chips. | A **colour palette** docked along the right edge of the window: click a swatch for the fill, right-click for the outline; the fill and outline in use also show in the status bar. [Choose colors][c-color], [Restoring the palette][c-kb-palette], [Status bar][c-status] | A swatch strip at the right edge of the window, outside the dock: click sets the fill, right-click the stroke, the first swatch is None. The swatches are the document's own. |
| Rulers | Off by default (Essentials). | Along the drawing window's top and left edges. [Application window][c-app] | On. |
| Workspaces | Essentials, Painting, Typography and others. | Workspaces for different kinds of work, including one that places features where Illustrator users expect them. [Choose a workspace][c-ws], [Illustrator workspace][c-ai-ws] | A built-in **Draw** workspace holds the mode's arrangement and remembers changes made in it, as every workspace does; leaving Draw mode brings back the workspace used before. |

[c-pages]: https://product.corel.com/help/CorelDRAW/540111147/CorelDRAW-en/CorelDRAW-Pages.html
[c-layers]: https://product.corel.com/help/CorelDRAW/540111147/CorelDRAW-en/CorelDRAW-Create-layers.html
[c-nav]: https://product.corel.com/help/CorelDRAW/540111186/Documentation-Windows/CorelDRAW-en/CorelDRAW-Document-navigator.html
[c-add]: https://product.corel.com/help/CorelDRAW/540111147/CorelDRAW-en/CorelDRAW-Add-duplicate-and-delete-pages.html
[c-multi]: https://learn.corel.com/tutorials/coreldraw-multipage-view/
[c-views]: https://product.corel.com/help/CorelDRAW/540111147/CorelDRAW-en/CorelDRAW-Page-views.html
[c-prop]: https://product.corel.com/help/CorelDRAW/540111186/Documentation-Windows/CorelDRAW-en/CorelDRAW-Property-bar.html
[c-app]: https://product.corel.com/help/CorelDRAW/Documentation-Windows/CorelDRAW-en/CorelDRAW-CorelDRAW-application-window.html
[c-dock]: https://product.corel.com/help/CorelDRAW/Documentation-Windows/CorelDRAW-en/CorelDRAW-Dockers.html
[c-color]: https://product.corel.com/help/CorelDRAW/540111147/CorelDRAW-en/CorelDRAW-Choose-colors.html
[c-kb-palette]: https://kb.corel.com/en/128111
[c-status]: https://product.corel.com/help/CorelDRAW/540111186/Documentation-Windows/CorelDRAW-en/CorelDRAW-Status-bar.html
[c-ws]: https://product.corel.com/help/CorelDRAW/540111147/CorelDRAW-en/CorelDRAW-Choose-a-workspace.html
[c-ai-ws]: https://product.corel.com/help/CorelDRAW/540111186/Documentation-Windows/CorelDRAW-en/CorelDRAW-Using-the-Adobe-Illustrator-workspace.html

What this table takes from those pages is the arrangement and the behaviour. The look (icons, colours, wording,
the palette's colours) is VectorCraft's own.

**How the mode is built** (`crates/ui-egui/src/modes.rs`): `UiState::mode` (`illustrate` or `draw`, kept with the
preferences) and the UI command `window.mode {mode}`. Switching to Draw remembers the current workspace and
switches to the built-in Draw workspace; switching back restores the remembered one. While in Draw mode the window
adds the page bar (`chrome/page_bar.rs`) and the swatch strip (`chrome/swatch_strip.rs`). Both act only through
commands (`view.goToArtboard`, `pages.insert`, `pages.move`, `artboard.setProps`, `artboard.delete`,
`paint.setFill`, `paint.setStroke`), so the control channel and MCP reach everything they do. Adding a page to a
document without pages makes it a pages document (`pages.insert` on an artboards document), in the same undo step.

## 4. Draw mode work list

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

## 5. `.cdr` support (`cdr`)

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

## 6. Rules

- `CLAUDE.md` applies in full: clean-room for Adobe and Corel software, no panics, everything a command, layering,
  assets in `ASSETS.md`, `cargo xtask ci` before every commit.
- Merge `main` into `draw-mode`, and `draw-mode` into `cdr`, every few days.
- The existing `.ai`, EPS, PDF and SVG tests pass unchanged on both branches.

## Revision history

| Date | Change |
|---|---|
| 2026-10-28 | Section 3: layout comparison with sources; the Illustrate and Draw modes (D6), the page bar and the swatch strip |
| 2026-10-28 | First version: what exists (pages and master layers, Corel clean-room rules), decisions, draw mode, `.cdr` stages with writing last |
