# Tracking HUD

`tracking-hud.fs` is an ISF effect from the videoping shader pack. It turns the picture into a **machine-vision display**: it looks for things in the picture, frames them and tracks them.

What it draws:
- **Object discovery:** the picture is divided into a grid of detector cells. Every cell with enough activity is framed as a numbered detection box, with its strength in percent.
- **Main tracker:** follows the center of all activity with a crosshair and a box sized to how spread out the activity is. It shows **LOCK** while it has something, a blinking **SEARCH** when it has lost it, and live X/Y coordinates. It moves with adjustable inertia, like a real tracking camera.
- **Telemetry:** frame counter, active share of the picture, and the current detection source, in the top-left corner.
- **Detection mask:** optionally, a tint wherever the detector sees activity.

"Activity" can be **motion**, **brightness** or **one chosen color**, so it can follow a moving dancer, the brightest light, or a red object.

> ⚠ About 9 ms per frame at 1080p on an Intel Iris Xe (fine on a dedicated GPU).

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Source (0 motion, 1 bright, 2 color)** | 0 – 2 | 0 | What counts as activity: 0 = motion (change since the last frame), 1 = brightness, 2 = closeness to the Target hue. |
| **Threshold** | 0.01 – 1 | 0.12 | How strong the activity must be. Motion: how much change; brightness: how bright (e.g. 0.6 = only bright areas). Not used for color. |
| **Target hue** | 0 – 1 | 0 | Color mode: the hue to look for, around the color wheel (0 red, 0.17 yellow, 0.33 green, 0.5 cyan, 0.67 blue, 0.83 magenta). Gray and weak colors are ignored. |
| **Grid** | 3 – 24 | 10 | Number of detector cells from top to bottom (across scales with the aspect ratio). More = smaller, more numerous boxes. |
| **Object sensitivity** | 0 – 1 | 0.5 | How easily a cell becomes a detection box: low = only cells that are mostly active, high = cells with just a little activity. |
| **Tracker inertia** | 0 – 1 | 0.4 | How smoothly the main tracker follows: 0 = jumps straight to the target, 1 = drifts after it with over a second of lag. |
| **Show mask** | 0 – 1 | 0.3 | Tints everything the detector sees as active in the HUD color. |
| **Darken picture** | 0 – 1 | 0.3 | Darkens the picture so the HUD stands out; 1 = picture fully black, only the HUD remains. |
| **HUD color** | 0 – 1 | 0.33 | Color of the HUD around the color wheel (0.33 = green, 0.55 = cyan, 0.1 = amber). |
| **Text** | 0 – 1 | 1 | Labels and telemetry on (above 0.5) or off. |
| **Text size** | 1 – 5 | 2 | Size of the pixel font. It also scales with the output resolution. |

## Tips

- **Motion tracker:** the defaults, on footage with people moving.
- **Follow the light:** Source 1, Threshold 0.6.
- **Color tracker:** Source 2, Target hue on the object's color (e.g. 0 for red), HUD color 0.55.
- **Surveillance look:** Darken picture 0.6, Show mask 0.5, Grid 16.
- **Clean tracker only:** Object sensitivity 0 (no cell boxes), Text 0.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
