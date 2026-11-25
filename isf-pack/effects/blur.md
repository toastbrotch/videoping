# Blur

`blur.fs` is an ISF effect from the videoping shader pack. A smooth gaussian blur, from a soft focus to a picture dissolved into color fields; or a directional blur that smears everything along one angle.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Radius (px)** | 0 – 400 | 6 | Size of the blur in pixels: 6 = soft focus, 400 = only big color fields stay. |
| **Shape** | choice | round | **round**: blurs in all directions; **directional**: smears along one direction (motion blur). |
| **Direction** | 0 – 1 | 0 | The direction of the directional blur (0 = sideways, 0.5 = up and down). |
| **Glow (blur added on top)** | 0 – 4 | 0 | Adds the blur on top of the sharp picture: a bloom / glow look. 0 = plain blur. |

## Tips

- Glow 1-2 with radius 30 makes bright parts shine.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
