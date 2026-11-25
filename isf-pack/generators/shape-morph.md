# Shape morph

`shape-morph.fs` is an ISF generator from the videoping shader pack. Outlines that morph circle to square to triangle to star and back, repeated as rings that travel outwards.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Shape (0 circle - 1 square - 2 triangle - 3 star)** | 0 – 3 | 0 | The shape: 0 circle, 1 square, 2 triangle, 3 star; values in between blend. Good on a fader. |
| **Auto morph speed** | 0 – 2 | 0.2 | Morphs on its own at this speed. 0 = only by hand. |
| **Rings** | 1 – 30 | 8 | Number of rings. |
| **Ring travel** | -2 – 2 | 0.3 | Rings travel outwards (negative: inwards). |
| **Morph lag per ring** | 0 – 1 | 0.15 | Outer rings follow the morph later: a wave of shapes. |
| **Rotation** | -1 – 1 | 0.1 | Turning speed. |
| **Twist per ring** | -0.5 – 0.5 | 0.1 | Each ring is turned a bit more than the one inside it. |
| **Line width (px)** | 0.5 – 12 | 2 | Line width in pixels. |
| **Star points** | 3 – 12 | 5 | Points of the star. |
| **Inner color** | color | #f2ebd9 | Color of the inner rings. |
| **Outer color** | color | #1c6999 | Color of the outer rings. |
| **Background** | color | #000000 | Background. |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the background shows. 0 = transparent: only the pattern stays, and in the mixer (mix type A) the layer below shows through; light parts cover it, dark parts let it through. |

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
