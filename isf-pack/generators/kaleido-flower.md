# Kaleido flower

`kaleido-flower.fs` is an ISF generator from the videoping shader pack. A blooming flower of mirrored noise petals in polar coordinates, breathing and turning.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Petals** | 3 – 24 | 8 | Number of petals. |
| **Symmetry** | choice | mirrored | **mirrored** (every second petal reflected) or **rotated** (all petals turn the same way). |
| **Rotation** | -1 – 1 | 0.1 | Turning speed. |
| **Zoom** | 0.3 – 3 | 1 | Size of the flower. |
| **Bloom speed** | -2 – 2 | 0.3 | Speed of the growth from the center outwards; negative = inwards. |
| **Detail** | 0 – 1 | 0.5 | Fine detail in the petals. |
| **Rings** | 0 – 1 | 0.4 | Ring bands through the petals. |
| **Sharp edges** | 0 – 1 | 0 | From soft, smoky petals to hard cut edges; the colors step into flat bands. |
| **Crisp (fine detail + edge lines)** | 0 – 1 | 0 | Finer detail and thin bright lines along the petal edges and folds. |
| **Dirt (grain, specks, scratches)** | 0 – 1 | 0 | Film grain, light and dark specks, fine scratches and an uneven, faded print. Note: the grain also lifts the black background. |
| **Inner color** | color | #fad999 | Color in the middle. |
| **Outer color** | color | #8c4a87 | Color at the tips. |
| **Background** | color | #000000 | Background. |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the background shows. 0 = transparent: only the pattern stays, and in the mixer (mix type A) the layer below shows through; light parts cover it, dark parts let it through. |

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
