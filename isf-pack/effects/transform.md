# Transform

`transform.fs` is an ISF effect from the videoping shader pack. Zoom, turn, stretch and move the picture, with a continuous spin and drift; outside the picture it stretches the border, tiles, mirrors or turns transparent.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Zoom** | 0.02 – 30 | 1 | 1 = unchanged, 0.02 = tiny (with repeat: a wall of tiles), 30 = a close-up. |
| **Stretch (wide .. tall)** | -1 – 1 | 0 | Squeezes the picture wide (negative) or tall (positive). |
| **Angle** | -0.5 – 0.5 | 0 | Turns the picture (0.25 = a quarter turn). |
| **Spin (turns/s)** | -4 – 4 | 0 | Keeps it turning, in turns per second. |
| **Position** | position | 0, 0 | Moves the picture. |
| **Drift (screens/s)** | position | 0, 0 | Keeps it moving, in screens per second (endless with repeat or mirror). |
| **Outside the picture** | choice | black | What shows where the picture runs out: **stretch** (the border pixels), **repeat** (tiles), **mirror** (mirrored tiles, no seams), **black**, **transparent**. |

## Tips

- Zoom 0.25 + repeat + spin = a turning tile wall.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
