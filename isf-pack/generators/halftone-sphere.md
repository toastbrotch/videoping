# Halftone sphere

`halftone-sphere.fs` is an ISF generator from the videoping shader pack. A lit, turning 3D shape drawn with halftone dots like a print. The style switch turns it into shaded, wireframe, outline or x-ray.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Style** | choice | halftone | **halftone** (printed with dots), **shaded**, **wireframe**, **outline** (silhouette and edges), **x-ray**. |
| **Shape** | choice | sphere | Sphere, torus, cube or octahedron. |
| **Size** | 0.3 – 1.8 | 1 | Size of the shape. |
| **Rotation** | -2 – 2 | 0.3 | Rotation speed. |
| **Light angle** | 0 – 1 | 0.6 | Where the light comes from. |
| **Light moves** | 0 – 2 | 0.2 | The light circles around the shape. |
| **Dot size (px)** | 3 – 120 | 9 | Size of the halftone cells in pixels, up to big pop-art dots. |
| **Screen angle** | 0 – 1 | 0.25 | Angle of the halftone screen. |
| **Line width (px)** | 0.5 – 24 | 1.2 | Width of the lines (wireframe, outline) in pixels. |
| **Wire density** | 1 – 24 | 8 | How many wire lines per unit: more = a finer mesh. |
| **Fill under lines** | 0 – 1 | 0 | In the line styles: how much of the shaded surface shows under the lines. 0 = pure lines on the background. |
| **Paper (0 = transparent)** | 0 – 1 | 1 | How much of the paper shows. 0 = transparent: only the print (dots, lines, fill) stays, and in the mixer (mix type A) the layer below shows through. Pick a light ink on a dark video. |
| **Ink** | color | #14141a | Ink: dots and lines. |
| **Shadow color** | color | #c7630a | The second ink in the shadows. |
| **Paper** | color | #f0e8d6 | Paper (background). |

## Tips

- A second, finer screen in the shadow color gives the print its depth: set it to the paper color for a plain one-color print.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
