# Posterize

`posterize.fs` is an ISF effect from the videoping shader pack. Reduces the picture to a few color steps, like a screen print or an old game. Dither breaks the steps into a fine pattern of dots.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Steps** | 2 – 32 | 5 | Number of steps per color: 2 = hard poster, 32 = almost smooth. |
| **Mode** | choice | color | **color**: every channel in steps (colors shift); **brightness**: only the brightness in steps, the hue stays. |
| **Dither** | 0 – 1 | 0 | Breaks the steps into a fine pattern of dots (ordered dither, like old computer graphics). |
| **Dither size (px)** | 1 – 32 | 1 | Size of the dither dots in pixels: big values give a coarse pixel pattern. |
| **Step spread** | 0.2 – 5 | 1 | Where the steps sit: below 1 more steps in the bright parts, above 1 more in the dark parts. |

## Tips

- Steps 2 + dither 1 + dither size 3 = 8-bit computer look.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
