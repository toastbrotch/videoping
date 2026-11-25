# Melting blobs

`melting-blobs.fs` is an ISF generator from the videoping shader pack. Soft 3D blobs melting into each other and apart again (raymarched metaballs). Five looks: shaded, wireframe, outline, dots, x-ray.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Style** | choice | shaded | Look: **shaded** (lit surface), **wireframe** (lines that stick to the surface), **outline** (silhouette and sharp edges only), **dots** (a halftone screen of the lighting), **x-ray** (see-through glow of the volume). |
| **Blobs** | 2 – 8 | 5 | Number of blobs. |
| **Melt** | 0.05 – 1 | 0.75 | How far the blobs melt into each other before they touch. |
| **Speed** | 0 – 3 | 0.5 | Speed of the movement. |
| **Rotation** | -1 – 1 | 0.15 | The camera turns around the blobs. |
| **Spread** | 0.2 – 2 | 0.8 | How far the blobs wander apart. |
| **Gloss** | 0 – 1 | 0.6 | Shiny highlight. |
| **Line width (px)** | 0.5 – 5 | 1.2 | Width of the lines (wireframe, outline) in pixels; in the dots style the size of the dot cells. |
| **Wire density** | 1 – 20 | 6 | How many wire lines per unit: more = a finer mesh. |
| **Fill under lines** | 0 – 1 | 0 | In the line styles: how much of the shaded surface shows under the lines. 0 = pure lines on the background. |
| **Color** | color | #f299e6 | Main color: light side, lines, dots. |
| **Shadow color** | color | #1c408c | Shadow side. |
| **Background** | color | #000000 | Background. |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the background shows. 0 = transparent: only the pattern stays, and in the mixer (mix type A) the layer below shows through; light parts cover it, dark parts let it through. |

## Tips

- The wireframe lines are slices through the volume, so they bend and flow with the shape as it melts.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
