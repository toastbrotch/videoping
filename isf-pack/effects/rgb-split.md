# RGB split

`rgb-split.fs` is an ISF effect from the videoping shader pack. Pulls the red, green and blue parts of the picture apart: a slight lens fringe, a 3D-glasses look, or three ghost copies flying apart. Sideways, zooming out from the center, or turning.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Distance (px)** | 0 – 500 | 6 | How far red and blue move away from green, in pixels: 6 = a lens fringe, 500 = three ghost pictures. |
| **Mode** | choice | sideways | **sideways** (along the angle), **from the center** (red zooms out, blue in), **turning** (red and blue turn against each other). |
| **Angle** | 0 – 1 | 0 | Direction of the sideways split; in the turning mode an extra turn. |
| **Wobble** | 0 – 1 | 0 | The distance breathes and jumps: a nervous, glitchy look. |
| **Wobble speed** | 0 – 20 | 1 | How fast it wobbles. |
| **Center** | position | 0.5, 0.5 | Center for the modes from the center and turning. |

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
