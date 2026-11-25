# Dot matrix

`dot-matrix.fs` is an ISF generator from the videoping shader pack. An LED wall of round dots with glow. 14 patterns: waves, noise, rings, bars, plasma, lines, diagonals, scanner, spiral, checker, rain, sparkle, radar, squares. Unlit dots stay faintly visible.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Pattern** | choice | waves | What the wall shows: waves, noise, rings, bars, plasma, lines (scrolling up), diagonals, scanner (one column sweeping with a trail), spiral, checker (flipping), rain (falling drops), sparkle (random flashes), radar (sweeping beam with blips), squares (a tunnel of outlines). |
| **Dots across** | 8 – 160 | 48 | Number of LEDs across. |
| **Dot size** | 0.2 – 1 | 0.7 | Size of an LED in its cell. |
| **Speed** | 0 – 4 | 1 | Speed of the pattern. |
| **Pattern scale** | 0.2 – 4 | 1 | Size of the pattern. |
| **Brightness steps** | 2 – 16 | 6 | Brightness steps, like real LED drivers. Few = harsh, many = smooth. |
| **Glow** | 0 – 1.5 | 0.5 | Light around lit LEDs. |
| **Unlit dots** | 0 – 0.4 | 0.08 | How visible the unlit LEDs are. |
| **LED color** | color | #ff8c1a | LED color. |
| **Hot color** | color | #fff2cc | Color of the brightest LEDs. |
| **Background** | color | #000000 | Background between the LEDs. |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the background shows. 0 = transparent: only the pattern stays, and in the mixer (mix type A) the layer below shows through; light parts cover it, dark parts let it through. |

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
