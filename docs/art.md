# Character art

## Direction

Portrait-led arcade pixel art: individual faces, curved anatomy, shaded materials, and distinct silhouettes. Founders and Bernie use recognizable hairstyles, clothing, and facial structure. Human rivals have tailored outfits and tools; mechanical rivals have articulated joints and curved armor.

`art/source/` contains generated source sheets. Character sheets have eight direction columns and four pose rows: idle, left stride, passing stride, right stride. Columns face down, down-left, left, up-left, up, up-right, right, down-right. Preserve these sources; rebuilding does not generate new images.

## Rebuild

Install Python 3.11+ and uv. Pillow is pinned in the importer.

```sh
pnpm assets
pnpm assets:check
uv run scripts/import-actors.py --check --preview
```

The preview writes `test-results/actor-directions.png`. Check faces, eight distinct views, padding, and silhouettes against a dark background.

```text
art/source/*.png --import_actor(): slice 8×4--> individual poses
poses --clean(): alpha threshold + largest component--> isolated silhouettes
silhouettes --pixelize() + apply_palette()--> aligned 128px cells, 64 colors per actor
cells --main(): page/row/column packing--> assets/actors-{0..4}.png
front pose --import_actor(): upper-body crop--> assets/{name}.png portraits
Renderer::load() --GET assets/actors-{0..4}.png--> five WebGL textures
Renderer::actor() --Facing::sprite_column(frame)--> 1280×720 backing target
```

The importer rejects missing silhouettes, clipped output, duplicate facings, and duplicate pose frames. `--check` rebuilds in memory and rejects stale runtime assets. Image generation is not deterministic; processing the saved sources is.

Alpha thresholds remove colored fringe pixels and the faint background in Elon's sheet. Connected-component cleanup discards stray marks. A common scale per actor preserves animation proportions; torso centering prevents held tools from shifting the body. Humanoid feet share `(64,108)`; crawler and whale bodies use a stable hover center. One palette spans all 32 poses to reduce color shimmer. Portraits retain higher-resolution facial detail from the source.

## Runtime layout

Each RGBA atlas is 2048×1024: 16 columns, eight rows, 128px cells. Four actors occupy two rows each. The last page has three unused actor slots. AOC and Alice Huang extend the roster to 17 actors / 544 frames. `bubble.png` uses a separate cleaned, palette-limited 128px sprite with runtime hovering. For actor ID `a` and `c = direction * 4 + frame`:

```text
page = a / 4
column = c % 16
row = (a % 4) * 2 + c / 16
```

IDs follow `ACTORS` in the importer and the mappings in `founder.rs` / `render.rs`. Actors draw into 64×64 logical units with a `(32,54)` foot anchor. The 1280×720 target retains 640×360 logical coordinates. Nearest-neighbor scaling keeps edges crisp. Collision, movement, idle facing, and walk clocks remain simulation-owned.

`scripts/pixel-art.py` generates props, packs, and logos separately. It cannot overwrite imported actors or portraits. Photo-reference attribution ships in `assets/ART-CREDITS.txt`.

`packs.png` has five 64px columns: Democratic Party banner, GPU voucher, audit sign, stock-crash ticker, and stock-rally ticker. The banner uses the party's circled-D emblem. Matching `democrats.png`, `regulatory_crackdown.png`, `ai_stock_crash.png`, and `ai_stock_rally.png` icons appear in news cards.

## Environment and weapons

`scripts/environment-art.py` shares cleanup helpers with the character importer. It processes `art/source/environment.png`, a four-column, two-row equipment sheet, into 128×160 cells anchored at `(64,148)`. Runtime cabinets occupy 64×80 logical units. Glass/mesh racks, CRAC cooling, UPS, network patch panels, coolant distribution, GPU manifolds, and a service cart share the character shading style. Prop footprints retain the terrain collision grid.

The same script draws deterministic 64px raised-floor panels and a 768×256 projectile atlas with 64px cells. Columns are legislation, subpoena, citation, flag, wrench, coin, bolt, scrap, Cybertruck, leather jacket, cash bundle, and safety rail. Four rows animate flags, coins, wheels, and highlights. Rust rotates projectiles and aims trucks/barriers; jackets spin during their return flight. Wider Context scales friendly artwork with collision radii. Projectile age is simulation time, so pause freezes the animation.
