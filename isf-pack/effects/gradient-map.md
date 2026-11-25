# Gradient map

`gradient-map.fs` is an ISF effect from the videoping shader pack. Re-colors the brightness through a palette, optionally repeated into false-color contour bands, sliding or cycling.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Palette (0-7)** | 0 – 7 | 0 | 0 Thermal, 1 Synthwave, 2 Sepia, 3 Neon, 4 Ice, 5 Rainbow, 6 Toxic, 7 Blood. |
| **Repeat** | 1 – 12 | 1 | Repeats the palette over the brightness range: false-color contour bands. |
| **Shift** | 0 – 1 | 0 | Slides the palette along the brightness. |
| **Speed** | −1 – 1 | 0 | Keeps the palette cycling; negative = the other way. |

## Tips

- **Thermal camera:** the defaults.
- **Contour bands:** Palette 5, Repeat 8.
- **Flowing neon:** Palette 3, Repeat 3, Speed 0.3.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
