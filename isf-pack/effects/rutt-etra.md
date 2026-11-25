# Rutt-Etra

`rutt-etra.fs` is an ISF effect from the videoping shader pack. After the **Rutt/Etra video synthesizer** of the 1970s: the picture is drawn only as horizontal scanlines, and each line is pushed upwards by the brightness under it, so the picture becomes a relief of lines. A tilt gives it a pseudo-3D look; there is no real 3D camera.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Scanlines** | 8 – 160 | 60 | Number of lines. |
| **Line width (px)** | 0.5 – 6 | 1.5 | Line width in pixels. |
| **Displacement** | 0 – 3 | 1 | How far bright areas push the lines up. 0 = straight lines. |
| **Perspective / tilt** | 0 – 1 | 0.35 | Tilts the line field backwards, so it looks like a landscape seen at an angle. |
| **Depth fog** | 0 – 1 | 0.3 | Distant lines fade into the background. |
| **Mesh (vertical lines)** | 0 – 1 | 0.3 | Adds vertical lines, turning the scanlines into a wire mesh. |
| **Smoothing** | 0 – 1 | 0.5 | Smooths the brightness before displacement. Low = jagged, noisy lines; high = soft hills. |
| **Glow** | 0 – 1 | 0.3 | Glow around the lines. |
| **Color mode** | 0 – 3 | 0 | 0 = phosphor green, 1 = white, 2 = the picture's own colors, 3 = rainbow by depth. |
| **Background brightness** | 0 – 1 | 0 | Brightness of the background (0 = black). |
| **Invert** | on/off | off | Inverts the brightness that drives the displacement: dark areas rise instead. |
| **Noise / jitter** | 0 – 1 | 0.05 | Analog jitter on the lines. |
| **Rotation (0-180°)** | 0 – 1 | 0 | Rotates the scan direction, up to 180°. 0.5 = vertical lines. |
| **Scroll speed** | -1 – 1 | 0 | Lets the lines scroll through the picture. 0 = still. |
| **Chaos** | 0 – 1 | 0 | More displacement (up to 2.5×), more jitter, thicker lines. |

## Tips

- **Classic Rutt/Etra:** the defaults.
- **Wireframe landscape:** Mesh 1, Perspective 0.7, Displacement 1.5, Color mode 3.
- **Color scan:** Color mode 2, Scanlines 120, Line width 1.
- **Heavy signal:** Chaos 1, Glow 0.8, Scroll speed 0.3.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
