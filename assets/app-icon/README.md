# DRAWcraft app icon

**Figure:** a medieval knight's helm: a great bascinet with a pointed visor, eye slits and breathing
holes, a plume of ostrich feathers, the gorget and the shoulder plates. DRAWcraft's mascot, the
companion to upstream VectorCraft's dragon.

**Style:** an engraving-style portrait in the Crafting Apps template (as VectorCraft's dragon and the
FilmCraft owl): a tight head-and-shoulders portrait on a full-bleed field, no frame or roundel, with the
armour running off the tile. Tone comes only from engraved lines that swell from the highlight into the
shadow; there are no filled greys.

**Palette** (exactly three colours):

| Colour | Hex | Used for |
|---|---|---|
| Ink | `#0b0b0c` | line work and contour |
| Paper | `#efe9dc` | the figure |
| DRAWcraft colour | `#2d5c9e` | the full-bleed field (heraldic azure) |

**Tile:** `viewBox="0 0 512 512"`, a rounded square with `rx=112`, clipped. The macOS files add Apple's
transparent margin (tile = 824/1024 of the canvas); Windows and Linux use the tile edge to edge.

**Provenance:** original artwork for DRAWcraft, generated in code by
[`packaging/knight-icon.py`](../../packaging/knight-icon.py) (standard library only, deterministic).
`vectorcraft.svg` is its output and the canonical artwork; `vectorcraft-small.svg` is the same file. The
file names stay those of upstream so every build and packaging script keeps working unchanged.
Licence: see `LICENSE.txt`.

## Files

| File | What | Used by |
|---|---|---|
| `vectorcraft.svg` | master vector (the generator's output, about 400 KB) | source for everything below |
| `vectorcraft-small.svg` | the same vector | `hicolor/scalable` |
| `vectorcraft-1024.png` | full tile, 1024 px | docs, README, store listings |
| `vectorcraft-macos-512.png` | tile with Apple margin, 512 px | runtime Dock icon (`apps/vectorcraft/src/main.rs`) |
| `vectorcraft.icns` | macOS icon set, 16 to 1024 px | `cargo xtask bundle` (`CFBundleIconFile`) |
| `vectorcraft.ico` | Windows icon, 16 to 256 px | `apps/vectorcraft/build.rs` (embedded in the `.exe`) |
| `hicolor/<size>/apps/ai.storyteller.vectorcraft.png` | Linux theme icons, 16 to 512 px; the 256 one is also the runtime icon on Windows and Linux, the 128 one the brand mark in the app bar and About box (`crates/ui-egui/src/brand.rs`) | `packaging/linux/ai.storyteller.vectorcraft.desktop` |
| `hicolor/scalable/apps/ai.storyteller.vectorcraft.svg` | Linux scalable icon | as above |

## How it reaches each OS

- **macOS:** the app sets the Dock / app-switcher icon at runtime (`ViewportBuilder::with_icon`); the
  `.app` from `cargo xtask bundle` carries `vectorcraft.icns`.
- **Windows:** `build.rs` embeds `vectorcraft.ico` and VERSIONINFO with `winresource` (taskbar, Start
  menu, Explorer, Alt-Tab); the runtime icon covers the title bar.
- **Linux:** install `packaging/linux/ai.storyteller.vectorcraft.desktop` to `share/applications/` and
  `hicolor/` to `share/icons/`. The app sets its Wayland app ID / X11 class to
  `ai.storyteller.vectorcraft` (DRAWcraft's own identifiers are plan item D7).
- **Web:** the page's favicon and home-screen icon (`apps/vectorcraft-web/index.html`).

## Regenerate

```sh
python3 packaging/knight-icon.py > assets/app-icon/vectorcraft.svg
cp assets/app-icon/vectorcraft.svg assets/app-icon/vectorcraft-small.svg
packaging/icons.sh
```

`packaging/icons.sh` needs `resvg` (and `iconutil` on macOS for the `.icns`). The committed files of
2026-10-28 were rendered with Chromium at the same sizes and viewBoxes, with the `.ico` from
`cargo xtask ico` and the `.icns` written as PNG entries.
