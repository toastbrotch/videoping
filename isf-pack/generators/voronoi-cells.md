# Voronoi cells

`voronoi-cells.fs` is an ISF generator from the videoping shader pack. Drifting, pulsing cells with glowing edges. From stained glass to a living tissue to a pure wire net.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Cells** | 0.5 – 30 | 8 | Number of cells per screen height. Below 1 only a few huge cells are left. |
| **Jitter** | 0 – 1 | 0.9 | How irregular the cells are. 0 = a regular grid. |
| **Drift** | 0 – 2 | 0.4 | How fast the cell centers move. |
| **Edge width** | 0.005 – 0.3 | 0.04 | Width of the edges. |
| **Edge glow** | 0 – 2 | 0.6 | Light spilling from the edges into the cells. |
| **Cell fill** | 0 – 1 | 0.5 | How much the cells are filled with color. 0 = a pure wire net. |
| **Pulse** | 0 – 1 | 0.5 | How strongly the cells pulse. |
| **Pulse speed** | 0 – 5 | 1 | Speed of the pulsing. |
| **Edge color** | color | #f2bff2 | Edge color. |
| **Cell color A** | color | #1c6999 | Cell color A (each cell picks a mix of A and B). |
| **Cell color B** | color | #8c4a87 | Cell color B. |
| **Background** | color | #000000 | Background. |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the background shows. 0 = transparent: only the pattern stays, and in the mixer (mix type A) the layer below shows through; light parts cover it, dark parts let it through. |

## Tips

- fill 0, glow 1: a glowing net; fill 1, glow 0: stained glass.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
