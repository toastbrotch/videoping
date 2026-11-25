# YOLO detect

`yolo-detect.fs` is an ISF effect from the videoping shader pack. It has the look of an **AI object detector** (the style of YOLO-type neural networks): class-colored boxes with a filled label bar reading something like `PERSON 0.93`, and optionally the segmentation mask and its outline.

There is no neural network inside, since a shader can't run one. Underneath is the same method as in `blob-detection.fs`: connected regions (bright, dark, moving or of one color) are found on the GPU. Each region is then given a class from what it looks like:

| Class | Color | When |
|---|---|---|
| **PERSON** | red | tall and narrow (box at least 1.6× as high as wide) |
| **VEHICLE** | blue | wide and flat, not tiny |
| **LIGHT** | yellow | bright and compact |
| **SIGN** | orange | strongly colored, red/orange hue |
| **PLANT** | green | strongly colored, yellow/green hue |
| **SCREEN** | cyan | strongly colored, cyan/blue/magenta hue |
| **STRUCTURE** | violet | sparse: fills less than half its box (frames, rings, lattices) |
| **OBJECT** | white | anything else |

The **confidence** comes from how clearly a region fits its class, and it flickers slightly from frame to frame like a real detector's. Detections below the confidence threshold are dropped.

> ⚠ About 8 ms per frame at 1080p on an Intel Iris Xe (fine on a dedicated GPU).

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Source (0 bright, 1 dark, 2 motion, 3 color)** | 0 – 3 | 0 | What forms a region: bright areas, dark areas, moving areas, or areas of the Target hue. |
| **Threshold** | 0.01 – 1 | 0.55 | Brightness that separates region from background; for motion, how much change counts. |
| **Target hue** | 0 – 1 | 0 | Color mode: the hue that forms regions. |
| **Grid** | 8 – 36 | 24 | Resolution of the detection grid. Finer = smaller objects and tighter boxes. |
| **Minimum size** | 1 – 40 | 2 | Smallest region that gets detected (box area in grid cells). |
| **Mask smoothing** | 0 – 1 | 0.4 | How smooth the segmentation mask and its outline are. |
| **Confidence threshold** | 0 – 0.99 | 0.5 | Detections with a lower confidence are not shown. Raise it for fewer, "surer" detections. |
| **Mask (segmentation)** | 0 – 1 | 0.35 | How strongly each region is filled with its class color. 0 = boxes only. |
| **Mask outline** | 0 – 1 | 0.6 | Visibility of the outline around each region, in its class color. |
| **Detector jitter** | 0 – 1 | 0.4 | How much boxes and confidence values jitter from frame to frame, like a real detector. 0 = rock steady. |
| **Box width** | 1 – 6 | 2 | Line width of the boxes. |
| **Labels** | 0 – 1 | 1 | Label bars with class and confidence on (above 0.5) or off. |
| **Text size** | 1 – 4 | 2 | Size of the label font. It also scales with the output resolution. |
| **Darken picture** | 0 – 1 | 0.2 | Darkens the picture; 1 = fully black, only the detections remain. |

## Tips

- **Classic detector:** the defaults, on footage with people and lights.
- **People against a bright background:** Source 1 (dark), Threshold 0.35.
- **Motion detector:** Source 2, Threshold 0.3, Mask smoothing 0.7, Minimum size 4.
- **Instance segmentation:** Mask 0.8, Mask outline 1, Darken picture 0.5.
- **Clean boxes:** Mask 0, Mask outline 0, Detector jitter 0.

As in blob detection, very winding shapes (rings, spirals) can be split into several detections. In this look that tends to read as plausible detector behavior.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
