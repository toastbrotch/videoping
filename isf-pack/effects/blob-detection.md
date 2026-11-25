# Blob detection

`blob-detection.fs` is an ISF effect from the videoping shader pack. It finds **connected regions** ("blobs") in the picture and outlines them like a machine-vision system, in the same visual language as `tracking-hud`.

What counts as a blob depends on the source: bright areas, dark areas, moving areas, or areas of one color. For every blob it draws:
- a **smooth contour line** along the blob's real edge, including holes (a ring is one blob with an inner and outer contour)
- its **own color** (fill and contour)
- a **bounding box** with corner brackets, an **ID** (B01, B02 …) and its size
- a **centroid** cross
- optionally **links** joining the blobs into a network

Telemetry in the top left counts the blobs. Blobs that touch are one blob: two overlapping circles get a single outline and a single box.

The grouping into connected regions runs entirely on the GPU, on a coarse grid: labels and box bounds spread through each blob in alternating row and column sweeps. That's why the boxes snap to grid cells, while the contour lines follow the picture exactly. Up to 24 blobs are labeled.

> ⚠ About 8 ms per frame at 1080p on an Intel Iris Xe (fine on a dedicated GPU).

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Source (0 bright, 1 dark, 2 motion, 3 color)** | 0 – 3 | 0 | What forms a blob: 0 = areas brighter than Threshold, 1 = areas darker than it, 2 = areas that move, 3 = areas of the Target hue. |
| **Threshold** | 0.01 – 1 | 0.55 | The brightness that separates blob from background (bright / dark). For motion: how much change counts (lower = more sensitive). Not used for color. |
| **Target hue** | 0 – 1 | 0 | Color mode: the hue that forms blobs, around the color wheel (0 red, 0.17 yellow, 0.33 green, 0.5 cyan, 0.67 blue, 0.83 magenta). |
| **Grid** | 8 – 36 | 24 | Resolution of the blob grid from top to bottom. Finer = smaller blobs found and tighter boxes; coarser = only big blobs, and nearby small ones merge. |
| **Minimum size** | 1 – 40 | 2 | Smallest blob that gets a box and ID, as the area of its box in grid cells. Raise it to ignore specks and noise. |
| **Contour smoothing** | 0 – 1 | 0.4 | How smooth the contour lines are. 0 = follows every jagged detail; 1 = soft, rounded outlines. |
| **Contours** | 0 – 1 | 1 | Visibility of the contour lines. |
| **Fill** | 0 – 1 | 0.3 | How strongly the blobs are filled with their color. |
| **Boxes** | 0 – 1 | 1 | Visibility of the bounding boxes and centroid crosses. |
| **Links** | 0 – 1 | 0.3 | Lines joining the blobs' centers, from blob to blob in order, like a network. |
| **Color per blob** | 0 – 1 | 0.7 | 0 = everything in the HUD color; 1 = every blob in its own color. |
| **Labels** | 0 – 1 | 1 | IDs, sizes and telemetry on (above 0.5) or off. |
| **Text size** | 1 – 4 | 2 | Size of the pixel font. It also scales with the output resolution. |
| **Darken picture** | 0 – 1 | 0.5 | Darkens the picture; 1 = fully black, only the detection remains. |
| **HUD color** | 0 – 1 | 0.33 | Color of text and (with Color per blob low) the whole overlay. |

## Tips

- **Light blobs:** the defaults, on footage with lights or bright objects on dark.
- **Silhouettes:** Source 1 (dark), Threshold 0.3, on a person against a bright background.
- **Moving objects:** Source 2, Threshold 0.3, Contour smoothing 0.7, Minimum size 4.
- **Color tracking:** Source 3, Target hue on the object's color.
- **Network:** Links 1, Fill 0.6, Darken picture 1.

Blob numbers follow the blobs' position (counted from the bottom row up), not their identity: when blobs move past each other, their numbers can swap.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
