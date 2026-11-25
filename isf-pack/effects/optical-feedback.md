# Optical feedback

`optical-feedback.fs` is an ISF effect from the videoping shader pack. It simulates **optical video feedback**: a camera pointed at the monitor that shows the camera's own picture. Every generation, the picture is filmed again, slightly zoomed, turned and shifted. The camera is contrasty and can invert, so every brightness edge multiplies into **zebra stripes**: black-and-white bands that crawl outward from edges and fill the picture. The loop advances at the rate set by **Generations/s** and holds the picture in between: calmer bands, less flicker, the same speed at any frame rate.

On top come VHS artefacts:
- edge sharpening that overshoots, which the loop turns into even more bands
- color that smears sideways
- line jitter, a rolling tracking band and the head-switching tear at the bottom
- noise and white dropout streaks

An optional camcorder **zebra pattern**, the diagonal stripes a camcorder viewfinder shows on overexposed areas, also runs inside the loop, so it gets filmed again and again.

> ⚠ Flashing: with *Invert per generation* at 1, large areas flip between black and white every generation. At the default of 12 generations per second that is a flicker. Lower Generations/s, Invert per generation or Feedback gain if it's too strong for your audience.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Generations/s** | 1 – 60 | 12 | How many feedback generations per second. Lower = slower bands and calmer flicker. Below about 8 you see the individual steps as a stop-motion jerk; the input and VHS noise are held between steps too. 60 = one generation per frame at 60 fps. |
| **Feedback mix** | 0 – 1 | 0.85 | Crossfade between the live input and the camera picture. 0 = input only; near 1 = almost pure feedback (the input only seeds it). |
| **Feedback gain** | 0 – 3 | 1.15 | Brightness of the camera picture each generation. Above 1 the loop grows and the zebra bands spread; too high and it saturates. |
| **Camera zoom** | 0.97 – 1.03 | 1.01 | Zoom per generation. Above 1 = bands flow outward (tunnel), below 1 = inward. |
| **Camera turn (degrees)** | -2 – 2 | 0.4 | Rotation per generation, for spiralling bands. |
| **Camera shift X** | -0.05 – 0.05 | 0 | Sideways shift per generation (fraction of the frame width). Bands stream sideways; positive = to the right. |
| **Camera shift Y** | -0.05 – 0.05 | 0 | The same vertically; positive = up. |
| **Camera focus** | 0 – 1 | 0.8 | Camera focus. Low = soft, wide, glowing bands; 1 = finest stripes. |
| **Black level** | 0 – 0.3 | 0.04 | Subtracted every generation. Higher = dark areas stay black and the loop calms down. |
| **Camera contrast** | 0.5 – 4 | 1.8 | Contrast of the camera around mid-gray. High contrast is what turns soft gradients into hard zebra edges. |
| **Invert per generation** | 0 – 1 | 1 | Inverts the camera picture every generation, so bands alternate black/white as they travel. This is the main zebra maker (see the flashing warning above). 0 = no inversion. |
| **Zebra fold** | 0 – 1 | 0 | Folds brightness back like a triangle wave, so a gradient becomes several contour stripes. Higher = more, finer stripes. |
| **Camcorder zebra** | 0 – 1 | 0 | Strength of the camcorder zebra pattern (moving diagonal stripes) on areas brighter than 0.85. |
| **VHS sharpening** | 0 – 3 | 0.6 | VHS edge enhancement. Its overshoot halos get fed back and add parallel bands next to every edge. 0 = clean edges. |
| **Color** | 0 – 1 | 0.3 | 0 = black and white; higher keeps the input's colors and lets the hue drift every generation, which gives rainbow zebras. |
| **VHS noise** | 0 – 1 | 0.3 | VHS damage: line jitter, a tracking band rolling up the picture, the head-switching tear at the bottom, grain, white dropouts, and stronger color smear. Tears also feed back into the loop. |

## Tips

- **Classic optical zebra:** the defaults.
- **Slow, hypnotic bands:** Generations/s 6, Zoom 1.02, Camera turn 0.8.
- **Stop-motion zebra:** Generations/s 3.
- **Calm feedback tunnel, no stripes:** Invert per generation 0, Camera contrast 1.2, VHS sharpening 0, VHS noise 0.
- **Rainbow zebra:** Color 1, Camera contrast 2.5.
- **Streaming stripes:** Camera shift X 0.01, Camera shift Y 0.005, Camera turn 0, Zoom 1.
- **Camcorder overload:** Camcorder zebra 1, Camera contrast 2.5.
- **Total meltdown:** Feedback gain 2, Camera contrast 3.5, VHS sharpening 2, VHS noise 0.8.

The loop starts from the input and clears whenever the shader is reloaded or the output size changes.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
