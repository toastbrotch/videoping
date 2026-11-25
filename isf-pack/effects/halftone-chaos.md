# Halftone chaos

`halftone-chaos.fs` is an ISF effect from the videoping shader pack. It redraws the picture as up to four rotated halftone dot grids, one for each ink, and sizes every dot by the brightness under it. The warp and the pulse waves run at a fixed speed that rises with their strength slider.

> ⚠ About 15 ms per frame at 1080p on an Intel Iris Xe (fine on a dedicated GPU).

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Cell size (px)** | 3 – 400 | 14 | Spacing of the dot grid in pixels. Large values give huge dots. |
| **Dot size** | 0 – 3 | 1 | Dot size relative to the grid. Above 1, dots overlap their neighbors, and from about 2 they can grow to 1.5 cells in radius. |
| **Contrast** | 0.2 – 6 | 1 | Contrast of the tones that set dot size. High values push dots to either nothing or full size. |
| **Dot shape (circle .. line)** | 0 – 6 | 0 | Dot shape. 0 circle, 1 diamond, 2 square, 3 ring, 4 star, 5 cross, 6 lines. Values in between blend the two neighboring shapes. |
| **Softness** | 0 – 1 | 0.05 | Edge softness. 0 is crisp; 1 gives blurry, glowing blobs. |
| **Screen angle** | 0 – 1 | 0 | Rotation of all grids. 1 is a full turn. |
| **Screen rotation** | -2 – 2 | 0 | Keeps the grids spinning, in radians per second. Negative values spin the other way. |
| **Ink angle spread** | 0 – 6 | 1 | Multiplies the angle between the colors' grids. At 1 you get the classic print angles, at 0 all grids line up, and higher values give strong moiré patterns. |
| **Color scheme (0-7)** | 0 – 7 | 1 | Color scheme. Rounded to the nearest whole number; see the table below. |
| **Hue shift** | 0 – 1 | 0 | Rotates the hue of all inks. 1 is a full turn around the color wheel. |
| **Ink offset (cells)** | 0 – 6 | 0 | Pushes each color's grid away from the others, up to 6 cells. |
| **Offset spin** | -4 – 4 | 0 | Keeps the color offsets circling over time. It only shows when Ink offset is above 0. |
| **Color smear** | 0 – 1 | 0 | Smears the colors: each color reads the image from a slightly different, drifting spot, up to 15 % of the frame away. |
| **Screen warp** | 0 – 1 | 0 | Wobbles the dot grids. Only the grids move; the image itself stays put. |
| **Pulse waves** | 0 – 1 | 0 | Rings of growing and shrinking dots that roll outward from the center. |
| **Chaos** | 0 – 1 | 0 | Turns up everything below at once. |

## Color schemes

| Value | Name | Look |
|---|---|---|
| 0 | Mono | Black dots on white paper |
| 1 | CMYK | Classic four-color print: cyan, magenta, yellow and black on paper |
| 2 | RGB | Red, green and blue light dots on black |
| 3 | Original | Dots in the image's own colors, boosted, on black |
| 4 | Neon | Three brightness grids colored from a drifting rainbow palette, on dark purple |
| 5 | Acid | CMYK inks that slowly shift hue, on paper that cycles through colors |
| 6 | Thermal | Hot-colored dots in the bright areas, cold blue dots in the shadows |
| 7 | Spectral | Cyan, magenta and yellow light dots on black |

The ink-on-paper schemes (Mono, CMYK, Acid) would turn black where huge dots stack up. To keep that from happening, the inks become opaque between Dot size 1 and 2. Where dots overlap, the one the pixel sits deepest inside wins, so high settings give bold, flat color shapes.

## Chaos

Chaos changes many things at once, and they all re-roll faster as the value rises: from 2 to 18 times per second.

- Horizontal bands of the image tear sideways.
- Areas of the frame get a different grid size (0.4×, 0.7×, 2× or 3.5×).
- Each color's grid angle and offset jump around.
- Single dots flicker in size.
- Some dots pick up their color from random places in the image.
- The colors sometimes swap places.
- Ink hues jitter.
- The whole frame sometimes flashes inverted.

Chaos also strengthens Screen warp and Pulse waves and makes Color smear reach further.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
