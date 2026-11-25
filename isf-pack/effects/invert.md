# Invert

`invert.fs` is an ISF effect from the videoping shader pack. Inverts the picture: the colors as a negative, only the brightness (colors stay), only the hue, or single channels. Amount fades through gray in between, which is a look of its own.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Mode** | choice | negative | **negative** (all colors), **brightness** (dark becomes light, the colors stay), **hue** (opposite colors, the brightness stays), **red**, **green**, **blue** (one channel only). |
| **Amount** | 0 – 1 | 1 | How far: 0.5 = everything gray in the middle. |
| **Solarize (only above)** | 0 – 1 | 0 | Only the parts brighter than this flip, like a solarized photo. 0 = everything. |

## Tips

- Map Amount to a button for an inverted flash.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
