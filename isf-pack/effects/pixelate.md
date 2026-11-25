# Pixelate

`pixelate.fs` is an ISF effect from the videoping shader pack. Turns the picture into big pixels: squares, round dots like an LED wall, or bars. From a slight retro grain to a few giant blocks.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Pixel size (px)** | 1 – 600 | 12 | Size of the pixels: 12 = retro grain, 600 = a few giant blocks. |
| **Shape** | choice | squares | **squares**, **round dots** (an LED wall), **diamonds**, **horizontal bars**, **vertical bars**. |
| **Gap** | 0 – 0.95 | 0 | Space between the pixels. |
| **Size by brightness** | 0 – 1 | 0 | Dark pixels get smaller: a halftone look. |
| **Gap color** | color | #000000 | Color between the pixels. |
| **Gap (0 = transparent)** | 0 – 1 | 1 | How much of the gap color shows. 0 = transparent gaps. |

## Tips

- Round dots + gap 0.2 + size by brightness 1 = a halftone print.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
