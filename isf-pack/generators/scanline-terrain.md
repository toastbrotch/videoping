# Scanline terrain

`scanline-terrain.fs` is an ISF generator from the videoping shader pack. A mountain range made of horizontal ridge lines, flying towards you, with real hidden-line removal. Styles: ridge lines, wire grid, solid shaded, dots.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Style** | choice | ridge lines | **ridge lines** (the classic look), **wire grid** (with lines along the flight), **solid** (shaded faces), **dots** (points on the ridges). |
| **Lines** | 8 – 80 | 40 | Number of ridge lines from front to horizon. |
| **Mountain height** | 0 – 3 | 1 | Height of the mountains. 0 = a flat plane. |
| **Roughness** | 0 – 1 | 0.5 | Smooth hills or jagged peaks. |
| **Speed** | -6 – 6 | 1.5 | Flying speed; negative flies backwards. |
| **Valley in the middle** | 0 – 1 | 0.6 | A flat valley along the middle, mountains on the sides. |
| **Horizon** | 0.3 – 0.95 | 0.62 | Height of the horizon on the screen. |
| **Line width** | 0.5 – 24 | 1.3 | Line width in pixels, up to thick glowing tubes (dot size in the dots style). |
| **Line color** | color | #ebf2ff | Color of lines and light. |
| **Fill color** | color | #000000 | Color of the terrain faces (hidden lines are covered by it). |
| **Sky color** | color | #000000 | Sky above the horizon. |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the background shows. 0 = transparent: only the pattern stays, and in the mixer (mix type A) the layer below shows through; light parts cover it, dark parts let it through. |

## Tips

- Hidden-line removal is real: a front ridge covers the lines behind it, like the famous pulsar plot.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
