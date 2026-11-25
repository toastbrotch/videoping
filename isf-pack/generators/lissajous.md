# Lissajous

`lissajous.fs` is an ISF generator from the videoping shader pack. Glowing oscilloscope curves with afterglow. The phase drifts, so the figure turns slowly in space; the trail buffer keeps the phosphor glow.

> ⚠ Heavy on laptop GPUs: about 10 ms per frame at 1080p on an Intel Iris Xe (fine on a dedicated GPU).

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Frequency X** | 1 – 9 | 3 | Horizontal frequency (whole numbers; the curve then closes). |
| **Frequency Y** | 1 – 9 | 2 | Vertical frequency. |
| **Frequency Z (depth)** | 0 – 9 | 5 | Depth frequency: with spin the figure turns in 3D. 0 = flat. |
| **Phase drift** | 0 – 2 | 0.3 | Phase drift: how fast the figure changes. |
| **Spin** | -2 – 2 | 0.2 | Turning around the vertical axis. |
| **Size** | 0.2 – 1.2 | 0.8 | Size of the figure. |
| **Beam width** | 0.5 – 6 | 1.5 | Width of the beam in pixels. |
| **Glow** | 0 – 2 | 0.6 | Soft light around the beam. |
| **Afterglow** | 0 – 0.98 | 0.85 | How long the phosphor glows on. High = long trails. |
| **Beam color** | color | #73ff99 | Color of the beam. |
| **Background** | color | #000000 | Screen color. |
| **Graticule** | 0 – 1 | 0 | The measuring grid of the oscilloscope screen (off by default). |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the background shows. 0 = transparent: only the pattern stays, and in the mixer (mix type A) the layer below shows through; light parts cover it, dark parts let it through. |

## Tips

- Ratios like 3:2, 5:4 or 3:4 give the classic figures; with afterglow near the top the drift paints ribbons.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
