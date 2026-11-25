# Moire

`moire.fs` is an ISF generator from the videoping shader pack. Two or three line or ring patterns drifting over each other: the interference makes large, slowly moving moire waves.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Pattern** | choice | rings | What each layer is made of: rings, straight lines, or one of each. |
| **Layers** | choice | 2 | Two or three gratings laid over each other. |
| **Frequency** | 20 – 300 | 120 | How many lines per screen height. The moire waves are the slow beat between the gratings. |
| **Drift** | 0 – 1.5 | 0.25 | How fast the gratings move and turn. |
| **Spread** | 0 – 1 | 0.3 | How far the centers of the gratings wander apart. |
| **Line thickness** | 0.1 – 0.9 | 0.45 | Width of the dark lines of each grating. |
| **Color A** | color | #f5f2eb | The light color (where all gratings are clear). |
| **Color B** | color | #000000 | The line color. |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the background shows. 0 = transparent: only the pattern stays, and in the mixer (mix type A) the layer below shows through; light parts cover it, dark parts let it through. |

## Tips

- Moire depends on the output resolution: what looks calm at 1080p can shimmer in a small preview.
- For op-art: rings, 2 layers, slow drift, frequency around 100.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
