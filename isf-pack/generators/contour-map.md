# Contour map

`contour-map.fs` is an ISF generator from the videoping shader pack. Animated noise drawn as topographic height lines, like a hiking map. Every fifth line is an index line.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Lines** | 3 – 60 | 16 | Number of height lines over the whole range. More = denser map. |
| **Line width** | 0.3 – 5 | 1.2 | Line width in pixels. |
| **Scale** | 0.5 – 8 | 2 | Zoom of the landscape: higher = more, smaller hills. |
| **Speed** | 0 – 2 | 0.15 | How fast the landscape changes shape. |
| **Warp** | 0 – 2 | 0.6 | How strongly the terrain is folded and twisted. |
| **Flow (lines move)** | -1 – 1 | 0.1 | Moves the lines up or down the slopes without changing the terrain; negative = the other way. |
| **Height shading** | 0 – 1 | 0.2 | Shading by height between the lines. 0 = lines on a flat background. |
| **Line color** | color | #f2e6cc | Color of the lines. |
| **Low color** | color | #000000 | Background, and the color of the low ground. |
| **High color** | color | #592e54 | Color of the high ground (with height shading). |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the background shows. 0 = transparent: only the pattern stays, and in the mixer (mix type A) the layer below shows through; light parts cover it, dark parts let it through. |

## Tips

- Every fifth line is drawn thicker, like the index lines of a real map.
- Slow speed and some flow give a calm, breathing background; high warp and speed a boiling one.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
