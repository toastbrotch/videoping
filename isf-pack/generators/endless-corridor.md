# Endless corridor

`endless-corridor.fs` is an ISF generator from the videoping shader pack. Flying through an endless corridor of frames, twisting as it goes, fading into fog. Five looks: shaded, wireframe, outline, dots, x-ray. Crazy slider adds warping, pulsing frames, kaleidoscope, camera roll and rainbow colors.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Style** | choice | shaded | Look: **shaded** (lit surface), **wireframe** (lines that stick to the surface), **outline** (silhouette and sharp edges only), **dots** (a halftone screen of the lighting), **x-ray** (see-through glow of the volume). |
| **Frame shape** | choice | square | Shape of the frames: square, round, hexagon. |
| **Speed** | -6 – 6 | 3 | Flying speed; negative flies backwards. |
| **Twist** | -1 – 1 | 0.35 | How much each frame is turned against the one before. |
| **Frame spacing** | 0.3 – 3 | 0.8 | Distance between the frames. |
| **Frame thickness** | 0.02 – 0.4 | 0.12 | Thickness of the frames. |
| **Sway** | 0 – 1.5 | 0.9 | The corridor bends left / right and up / down. |
| **Fog** | 0 – 0.5 | 0.1 | Fog in the distance. |
| **Line width (px)** | 0.5 – 5 | 1.2 | Width of the lines (wireframe, outline) in pixels; in the dots style the size of the dot cells. |
| **Wire density** | 1 – 30 | 8 | How many wire lines per unit: more = a finer mesh. |
| **Fill under lines** | 0 – 1 | 0 | In the line styles: how much of the shaded surface shows under the lines. 0 = pure lines on the background. |
| **Color** | color | #f2ebd9 | Main color: light side, lines, dots. |
| **Shadow color** | color | #8c4a87 | Shadow side. |
| **Background / fog** | color | #000000 | Background and fog color. |
| **Crazy** | 0 – 1 | 0 | Everything gets wild: the corridor warps, frames pulse and change shape, the camera rolls and lurches, the lens bends. |
| **Kaleidoscope** | 0 – 1 | 0 | Folds the view into a kaleidoscope (more = more mirror segments). |
| **Rainbow** | 0 – 1 | 0 | Colors run through the rainbow along the corridor. |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the background shows. 0 = transparent: only the pattern stays, and in the mixer (mix type A) the layer below shows through; light parts cover it, dark parts let it through. |

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
