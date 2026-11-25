# Color isolate

`color-isolate.fs` is an ISF effect from the videoping shader pack. Keeps one color and drains the rest to gray; above 1 the rest is over-saturated instead.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Hue** | 0 – 1 | 0 | The color to keep, around the color wheel (0 red, 0.33 green, 0.67 blue). |
| **Range** | 0.01 – 0.5 | 0.08 | How wide a range of hues around it is kept. |
| **Rest saturation** | 0 – 3 | 0 | Saturation of everything else: 0 = gray, 1 = normal, 3 = over-saturated. |

## Tips

- **Red only:** the defaults.
- **Green screen check:** Hue 0.33, Range 0.15.
- **Inverted logic:** Rest saturation 3.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
