# Optical feedback, three tubes

`optical-feedback-3tubes.fs` is an ISF effect from the videoping shader pack. It is `optical-feedback.fs` with an old **three-tube color camera** filming the monitor. Red, green and blue each have their own pickup tube, and the tubes are **misconverged**: red and blue have their own small zoom, rotation and shift errors relative to green.

Every generation the three color channels part a little further, so:
- black-and-white zebra bands become **rainbow fringes and color spirals**, even from a black-and-white input
- the misconvergence can **drift** slowly, like tubes warming up
- **tube lag** smears bright areas behind, longest on red

Everything from optical feedback stays, including the generation rate, plus a Zebra level slider.

Like optical feedback, it simulates **optical video feedback**: a camera pointed at the monitor that shows the camera's own picture. Every generation, the picture is filmed again, slightly zoomed, turned and shifted. The camera is contrasty and can invert, so every brightness edge multiplies into **zebra stripes**: black-and-white bands that crawl outward from edges and fill the picture.

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
| **Camcorder zebra** | 0 – 1 | 0 | Strength of the camcorder zebra pattern (moving diagonal stripes) on areas brighter than Zebra level. |
| **Zebra level** | 0.4 – 1 | 0.85 | Brightness above which Camcorder zebra appears. Lower = more of the picture gets striped. |
| **VHS sharpening** | 0 – 3 | 0.6 | VHS edge enhancement. Its overshoot halos get fed back and add parallel bands next to every edge. 0 = clean edges. |
| **Input color** | 0 – 1 | 0 | Saturation of the input. 0 = the input enters in black and white, so all color comes from the tubes; 1 = the input's full color. |
| **Hue drift** | -0.1 – 0.1 | 0 | Turns the hue of the fed-back picture a little every generation, for colors that cycle through the bands. 0 = off. |
| **VHS noise** | 0 – 1 | 0.3 | VHS damage: line jitter, a tracking band rolling up the picture, the head-switching tear at the bottom, grain, white dropouts, and stronger color smear. Tears also feed back into the loop. |

## Three-tube camera

Green is the reference tube: it uses the camera's own zoom, rotation and shift. Red and blue add their own errors on top. Small values are what you want: even 0.002 zoom difference grows into wide fringes over a few seconds of generations.

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Red tube zoom** | -0.01 – 0.01 | 0.002 | Extra zoom of the red tube. Positive = red rings grow faster than green, so red runs ahead on the outside. |
| **Red tube turn** | -1 – 1 | 0.15 | Extra rotation of the red tube (degrees per generation): red spirals off the other colors. |
| **Red tube shift X** | -0.01 – 0.01 | 0.001 | Extra sideways shift of the red tube: red trails stream off to one side. |
| **Red tube shift Y** | -0.01 – 0.01 | 0 | The same vertically. |
| **Blue tube zoom** | -0.01 – 0.01 | -0.002 | Extra zoom of the blue tube. The default is opposite to red, so red and blue part in both directions. |
| **Blue tube turn** | -1 – 1 | -0.15 | Extra rotation of the blue tube. |
| **Blue tube shift X** | -0.01 – 0.01 | -0.001 | Extra sideways shift of the blue tube. |
| **Blue tube shift Y** | -0.01 – 0.01 | 0 | The same vertically. |
| **Tube drift** | 0 – 1 | 0.3 | Lets the red and blue errors wander slowly on their own (about ±0.003 zoom, ±0.4°, ±0.003 shift at 1), so the color fringes keep changing. 0 = fixed errors. |
| **Tube lag** | 0 – 0.95 | 0.3 | Tube lag: each tube keeps part of its last image, so bright areas smear behind. Red lags most, blue least, which gives colored trails. High values give long, ghostly color trails. |

## Tips

- **Three-tube zebra:** the defaults: black-and-white zebra bands with yellow and blue zones and color fringes.
- **Plain optical feedback (tubes aligned):** all eight tube sliders, Tube drift and Tube lag at 0.
- **Rainbow spirals:** Invert per generation 0, Feedback gain 1.05, Camera contrast 1.4.
- **Strong misconvergence:** Red tube zoom 0.006, Blue tube zoom -0.006, Red tube turn 0.5, Blue tube turn -0.5.
- **Sideways color split:** Red tube shift X 0.006, Blue tube shift X -0.006, the other tube errors 0.
- **Ghost trails:** Tube lag 0.9, Invert per generation 0, Feedback gain 1.
- **Color cycling:** Input color 1, Hue drift 0.03.

The loop starts from the input and clears whenever the shader is reloaded or the output size changes.

## Known behavior

VHS noise disturbs the picture line by line. The loop amplifies these small horizontal disturbances into short horizontal dashes like everything else, and zoom and rotation carry them outward, so after a while a band of dashes can gather along the frame's edge (most visibly on the right with the default rotation). Optical feedback does the same. Lower VHS noise, or set it to 0, if you don't want it.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
