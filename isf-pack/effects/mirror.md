# Mirror

`mirror.fs` is an ISF effect from the videoping shader pack. Mirrors one half of the picture onto the other: left-right, top-bottom or all four quarters. The mirror line can move and turn, so faces and buildings fold into symmetric creatures.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Mirror** | choice | left-right | **left-right**, **top-bottom** or **four quarters**. |
| **Other half** | on / off | off | Mirrors the other half. |
| **Mirror line** | position | 0.5, 0.5 | Where the mirror line is. |
| **Angle** | -0.5 – 0.5 | 0 | Turns the mirror line. |
| **Spin (turns/s)** | -2 – 2 | 0 | Keeps the mirror line turning, in turns per second. |
| **Outside the picture** | choice | mirror | What shows where the picture runs out: **stretch** (the border pixels), **repeat** (tiles), **mirror** (mirrored tiles, no seams), **black**, **transparent**. |

## Tips

- A face or a dancer mirrored left-right turns into a symmetric creature.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
