# Starfield warp

`starfield-warp.fs` is an ISF generator from the videoping shader pack. Stars rushing towards you. At low speed calm points, at high speed long warp streaks; or the stars become spinning wire forms: crosses, rings, squares, triangles, hexagons, sparkles, or all of them mixed.

> ⚠ About 6-10 ms per frame at 1080p on an Intel Iris Xe (fine on a dedicated GPU).

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Form** | choice | streaks | **dots**, **streaks** (warp lines), or wire forms that spin: **crosses**, **rings**, **squares**, **triangles**, **hexagons**, **sparkles**, or **mixed** (every star its own form). |
| **Form size** | 0.3 – 16 | 1.6 | Size of the wire forms. Big forms get a coarser star grid: fewer, bigger forms. |
| **Form spin** | -3 – 3 | 0.5 | How fast the forms turn; every form at its own speed. Negative = the other way. |
| **Speed** | -2 – 8 | 1 | Flying speed; negative flies backwards. |
| **Density** | 0.01 – 1 | 0.3 | How many stars. Low values leave only a few lonely ones. |
| **Streak length** | 0 – 1 | 0.5 | Length of the streaks (grows with the speed). |
| **Star size** | 0.3 – 4 | 1 | Size of the stars. |
| **Roll** | -1 – 1 | 0.05 | The view rolls around the flight direction. |
| **Center** | position | 0.5, 0.5 | Where the stars come from. |
| **Color spread** | 0 – 1 | 0.3 | How much the stars vary towards the tint color. |
| **Star color** | color | #e6f2ff | Star color. |
| **Tint color** | color | #f299e6 | Tint color. |
| **Background** | color | #000000 | Background. |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the background shows. 0 = transparent: only the pattern stays, and in the mixer (mix type A) the layer below shows through; light parts cover it, dark parts let it through. |

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
