# Color adjust

`color-adjust.fs` is an ISF effect from the videoping shader pack. Brightness, contrast, saturation, hue, gamma and a color gain in one place. At the defaults the picture is unchanged; pushed to the ends it burns out, crushes, turns neon or goes gray.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Brightness** | -1 – 1 | 0 | Adds or takes away light. -1 = black, 1 = white. |
| **Contrast** | 0 – 8 | 1 | 1 = unchanged, 0 = flat gray, high values crush everything into hard colors. |
| **Saturation** | 0 – 8 | 1 | 1 = unchanged, 0 = black and white, high values give neon colors. |
| **Hue shift** | -1 – 1 | 0 | Turns all colors around the color wheel (in turns: 0.5 = opposite colors). |
| **Hue cycle (turns/s)** | -4 – 4 | 0 | Keeps the colors turning on their own, in turns per second. |
| **Gamma** | 0.1 – 10 | 1 | Brightens (above 1) or darkens (below 1) the mid tones; black and white stay. |
| **Exposure (stops)** | -6 – 6 | 0 | Like the exposure of a camera, in stops: +1 = twice as bright. |
| **Color gain** | color | #ffffff | Multiplies the colors: a tint. White = unchanged. |

## Tips

- At the defaults the picture is unchanged: start there and push one slider at a time.
- Saturation 0 + contrast 2 = hard black and white.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
