# Kaleido

`kaleido.fs` is an ISF effect from the videoping shader pack. A kaleidoscope: the picture is cut into mirrored wedges around a center and turned. From a calm six-fold mandala to a hundred flickering slivers.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Segments** | 1 – 64 | 6 | Number of mirrored wedges: 6 = a classic kaleidoscope, 64 = thin flickering slivers. |
| **Spin (turns/s)** | -2 – 2 | 0.03 | Turning speed, in turns per second. |
| **Angle** | 0 – 1 | 0 | Turns the pattern (for a fixed position or animation). |
| **Zoom** | 0.05 – 20 | 1 | Zooms into the picture; above 1 the wedges show less of it, below 1 the edge setting repeats it. |
| **Center** | position | 0.5, 0.5 | Center of the kaleidoscope on the screen. |
| **Source offset** | position | 0, 0 | Which part of the picture goes into the wedges. |
| **Rings** | 0 – 30 | 0 | Folds the distance too, so the pattern repeats outwards in rings. |
| **Outside the picture** | choice | mirror | What shows where the picture runs out: **stretch** (the border pixels), **repeat** (tiles), **mirror** (mirrored tiles, no seams), **black**, **transparent**. |

## Tips

- Animate Source offset slowly: the pattern changes without turning.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
