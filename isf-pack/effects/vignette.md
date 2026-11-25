# Vignette

`vignette.fs` is an ISF effect from the videoping shader pack. Darkens the edges of the picture like an old lens, from a gentle frame to a small spotlight in the dark. The color can be anything, and the shape round or following the frame.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Amount** | 0 – 1 | 0.6 | How dark the edges get: 1 = fully the vignette color. |
| **Size** | 0 – 2 | 0.75 | Size of the clear middle: 0 = a small spotlight. |
| **Softness** | 0 – 2 | 0.6 | How soft the transition is. |
| **Round (0 = frame shape)** | 0 – 1 | 0.5 | 1 = a circle, 0 = an oval that follows the frame. |
| **Center** | position | 0.5, 0.5 | Where the clear middle is. |
| **Color** | color | #000000 | Color of the edges (black like a lens; white for a dreamy look). |
| **Edge (0 = transparent)** | 0 – 1 | 1 | How much of the edge color shows. 0 = the edges become transparent. |

## Tips

- Size 0.1 + softness 0.3 + amount 1 = a spotlight; move Center to follow something.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
