# Color reduce

`color-reduce.fs` is an ISF effect from the videoping shader pack. It reduces the picture to its **N most common colors** and repaints every pixel in the nearest of them, like the ColorReduce FreeFrame plugin. The palette comes from the picture itself and changes as the picture changes.

How it works:
1. The colors of the frame are counted on a coarse 6 × 6 × 6 grid (216 possible colors), at 128 × 72 points.
2. The N most frequent become the palette.
3. Every pixel takes the closest palette color.

Because the palette is the *dominant* colors, small bright details can disappear at low color counts when they don't cover enough of the picture: a small sun on a big purple sky turns purple at 2 colors.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Colors (64 = off)** | 2 – 64 | 8 | How many colors the picture is reduced to. 2 = a hard two-tone picture in its two most common colors; 64 = effect off (the original picture), as in the plugin. Whole numbers. |
| **Palette rotate** | 0 – 1 | 0 | Shifts every pixel to another color of the palette, so colors swap places: shapes stay, colors scramble. 0 = correct colors. With N colors there are N different shifts across the range. |
| **Dithering** | 0 – 1 | 0 | Ordered dithering: a fine dot pattern that fakes the shades between the palette colors. 0 = flat color areas, as in the plugin. |
| **Pixel size (px)** | 1 – 64 | 1 | Big pixels, each in one palette color: a chunky low-resolution look. |

## Tips

- **Poster look:** Colors 4–8.
- **Two-tone:** Colors 2.
- **Color scramble:** Colors 6, Palette rotate 0.5. Animate Palette rotate for colors stepping through the palette.
- **Retro screen:** Colors 16, Dithering 1, Pixel size 3.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
