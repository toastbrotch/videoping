# Chroma key

`chroma-key.fs` is an ISF effect from the videoping shader pack. Makes one color transparent, like a green screen: in the mixer (mix type A) the layer below shows there. Spill removal takes the green (or blue) cast off the edges; the edge can shrink or grow; the matte view shows what is kept.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Key color** | color | #1abf40 | The color that becomes transparent: the green (or blue) of the screen. Pick it from the middle of the screen. |
| **Tolerance** | 0 – 1 | 0.25 | How far a color may differ from the key color and still go: 0.25 = a green screen with its shadows, 1 = almost everything but white, black and strong opposite colors. |
| **Softness** | 0 – 1 | 0.1 | How soft the edge between kept and keyed is: 0 = a hard cut, higher = a soft fade (hair, motion blur). |
| **Brightness counts** | 0 – 1 | 0.15 | How much brightness counts besides the color: 0 = a dark and a bright green key out alike, 1 = only the same brightness keys out. |
| **Spill removal** | 0 – 1 | 0.6 | Takes the key color's tint off what is kept, so edges do not glow green. |
| **Edge (shrink .. grow, px)** | -12 – 12 | 0 | Shrinks (negative) or grows (positive) what is kept, in pixels: negative removes a remaining fringe. |
| **Show** | choice | keyed picture | **keyed picture**; **matte** (white = kept, black = transparent: to set tolerance and softness); **only the key color** (the inverse: keeps the screen, removes the rest). |

## Tips

- Use mix type A in the mixer: the layer below shows where the color was.
- Set it up with Show = matte: raise Tolerance until the screen is black, raise Softness for soft edges, then back to the keyed picture.
- A remaining green fringe: Edge -2 to -4, or more Spill removal.
- It keys any color, not only screens: a blue sky, a colored background in a generator.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
