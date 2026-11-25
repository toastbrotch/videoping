# Threshold

`threshold.fs` is an ISF effect from the videoping shader pack. Splits the picture into two colors at a brightness threshold: hard black and white, a soft two-tone print, or per color channel for a pop-art look.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Threshold** | 0 – 1 | 0.5 | The brightness where dark turns into light. |
| **Softness** | 0 – 1 | 0.03 | 0 = a hard cut, high values a soft two-tone gradient. |
| **Mode** | choice | brightness | **brightness**: two colors; **per channel**: red, green and blue cut each on their own (pop art with up to 8 colors). |
| **Noise** | 0 – 1 | 0 | Moving grain on the threshold: the edges crawl like a photocopy. |
| **Dark color** | color | #000000 | Color of the dark parts. |
| **Light color** | color | #ffffff | Color of the light parts. |

## Tips

- Animate the threshold slowly for shapes that grow and shrink.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
