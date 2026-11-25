# Edges

`edges.fs` is an ISF effect from the videoping shader pack. Finds the edges in the picture: glowing lines on black like a neon drawing, lines laid over the picture, or dark ink outlines for a comic look.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Style** | choice | lines on black | **lines on black**: glowing lines, like a neon drawing; **lines over the picture**; **ink outlines**: dark lines on the picture, a comic look. |
| **Strength** | 0 – 40 | 3 | How bright / dark the lines are. High values catch faint edges too. |
| **Width (px)** | 0.5 – 30 | 1 | Width of the lines in pixels: big values give thick, smeared outlines. |
| **Threshold** | 0 – 1 | 0.08 | Edges weaker than this are left out: higher = only the strong outlines. |
| **Picture colors** | 0 – 1 | 1 | 1 = the lines take the picture's colors, 0 = the line color only. |
| **Line color** | color | #ffffff | Color of the lines (with Picture colors, a tint). |

## Tips

- Lines on black + mix type A (or the add mix) puts a glowing outline drawing over another layer.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
