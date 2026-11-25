# Hex pulse

`hex-pulse.fs` is an ISF generator from the videoping shader pack. A honeycomb grid lit by waves that ripple out from moving points. Cells light up, hold and fade, like a hex LED floor.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Hex size** | 0.015 – 0.25 | 0.06 | Size of the hexagons. |
| **Gap** | 0 – 0.6 | 0.12 | Gap between the cells. |
| **Wave sources** | 1 – 6 | 3 | Number of points the waves start from. |
| **Wave speed** | 0 – 4 | 1 | Speed of the waves. |
| **Wave length** | 0.05 – 1.5 | 0.35 | Distance between the waves. |
| **Wave sharpness** | 0 – 1 | 0.6 | Soft swells or thin sharp rings. |
| **Sources wander** | 0 – 1 | 0.2 | How fast the wave sources move. |
| **Outline** | 0 – 1 | 0.25 | A thin outline on every cell. |
| **Color A** | color | #388f3d | Color of weakly lit cells (and the outlines). |
| **Color B** | color | #f2f299 | Color of strongly lit cells. |
| **Background** | color | #000000 | Background and gaps. |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the background shows. 0 = transparent: only the pattern stays, and in the mixer (mix type A) the layer below shows through; light parts cover it, dark parts let it through. |

## Tips

- Each cell lights as one, like a real LED floor; a light flicker keeps it alive.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
