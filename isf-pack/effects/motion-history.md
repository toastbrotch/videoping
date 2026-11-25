# Motion history

`motion-history.fs` is an ISF effect from the videoping shader pack. It shows **when** things moved. Every pixel remembers how long ago it last changed. That age is painted as a color gradient: fresh motion in the hot end of the palette, older motion cooling down, then black. Movement leaves colored **time-trails** behind it, and with Time rings they turn into contour rings, like growth rings in time.

It's a classic computer-vision tool (a "motion history image"), and a close relative of `frame-diff.fs`: frame difference shows *that* something moved, Motion History also shows *how long ago*.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Threshold** | 0.01 – 0.5 | 0.08 | How much a pixel's brightness has to change to count as motion. Low = even noise and flicker leave trails; high = only strong motion. |
| **Duration (s)** | 0.2 – 10 | 2 | How long a trail lasts, in seconds. Short = quick comet tails; long = long-lasting paths of everything that moved. |
| **Palette (0-5)** | 0 – 5 | 0 | Colors from fresh to old: 0 Thermal (white-hot to deep blue), 1 Rainbow, 2 Synthwave, 3 Ice, 4 Mono (white to black), 5 Toxic. |
| **Time rings (0 = smooth)** | 0 – 24 | 0 | Cuts the trail into that many steps, so the gradient becomes contour rings, each ring one moment in time. 0 = smooth gradient. |
| **Color cycle** | -1 – 1 | 0 | Lets the palette run along the trails over time, so colors flow through the rings. 0 = fixed colors. |
| **Original underneath** | 0 – 1 | 0 | Shows the picture underneath the trails. 0 = only the trails on black. |

## Tips

- **Heat trails:** the defaults.
- **Time rings:** Palette 1, Time rings 10, Duration 4.
- **Psychedelic flow:** Palette 2, Time rings 16, Color cycle 0.5.
- **Trails over the picture:** Palette 3, Original underneath 0.5.

Works best on footage with a still background and moving subjects (dancers, people walking, a handheld object). On content where everything moves all the time, the whole frame stays "fresh"; raise Threshold.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
