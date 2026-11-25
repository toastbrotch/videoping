# Optical feedback keyer

`optical-feedback-keyer.fs` is an ISF effect from the videoping shader pack. **Keyed optical feedback**, after the Sandin Image Processor, a modular analog video synthesizer Dan Sandin built in Chicago in 1971–73. Its keyers decided *where* in the picture an effect happens.

Here the effect is a camera-at-the-monitor feedback loop, as in `optical-feedback`, with zoom, rotation, shift, contrast and per-generation inversion. A **window keyer** decides where that feedback runs:
- **Inside the key:** the picture grows, spirals and echoes.
- **Outside the key:** the live input stays calm, or turns black.

This gives controlled compositions instead of full-frame chaos. Glowing **key edges** are fed back as well, a light **three-tube misconvergence** adds color fringes, and a **colorizer**, Sandin's trademark, turns brightness bands into colors on the way out.

> ⚠ About 10 ms per frame at 1080p on an Intel Iris Xe (fine on a dedicated GPU).

## Feedback loop

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Generations/s** | 1 – 60 | 15 | How many feedback generations per second (as in optical feedback). Lower = slower; below about 8 it steps visibly. |
| **Feedback mix (in the key)** | 0 – 1 | 1 | Inside the key: how much of the fed-back camera picture versus the live input. |
| **Feedback gain** | 0 – 3 | 1.3 | Brightness of the camera picture each generation. Around 1 = stable echoes; above 1 the keyed areas grow and saturate to flat color, below 1 they fade. |
| **Camera zoom** | 0.97 – 1.03 | 1.028 | Zoom per generation. Above 1 = echoes flow outward. |
| **Camera turn (degrees)** | -2 – 2 | 0.52 | Rotation per generation, for spirals. |
| **Camera shift X** | -0.05 – 0.05 | 0.002 | Sideways shift per generation; positive = right. |
| **Camera shift Y** | -0.05 – 0.05 | 0 | Vertical shift per generation; positive = up. |
| **Camera focus** | 0 – 1 | 0.85 | Camera focus. Low = soft glowing echoes, 1 = sharp. |
| **Black level** | 0 – 0.3 | 0 | Subtracted every generation; higher = echoes die out faster. |
| **Camera contrast** | 0.5 – 4 | 4 | Contrast of the camera. Higher = hard zebra edges. |
| **Invert per generation** | 0 – 1 | 0.14 | Inverts the camera picture every generation, for zebra bands inside the key. Flickers at 1. |

## Keyer

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Key source (input / loop / motion)** | 0 – 2 | 0 | Which signal cuts the key. **0** = brightness of the input: feedback where the input is bright (with the default levels). **1** = brightness of the fed-back picture itself: the loop keys itself, so keyed areas can grow or eat themselves away. **2** = difference between input and feedback: feedback where the picture changes (motion). In-between values blend. |
| **Key low** | 0 – 1 | 0.37 | Lower level of the key window: the key is on where the signal is above this. |
| **Key high** | 0 – 1.1 | 1.1 | Upper level: the key is off again above this. 1.1 = no upper limit. Lower it to key only a band of brightness. |
| **Key softness** | 0 – 0.3 | 0.04 | Softness of the key edges. 0 = hard cut, higher = soft blend between inside and outside. |
| **Invert key** | 0 – 1 | 0 | Swaps inside and outside: feedback in the dark areas instead of the bright ones. |
| **Key memory** | 0 – 0.99 | 0.059 | Keyed areas stay open after the signal has left them and fade at this rate per generation. 0.9+ = long-lasting holes that motion (Key source 2) paints into the picture. |
| **Black outside** | 0 – 1 | 0.02 | Outside the key: 0 = live input, 1 = black (only the feedback is visible). |
| **Key edges** | 0 – 2 | 0.4 | Glowing outline along the key's edge. It's fed back too, so outlines echo outward. |
| **Key edge hue** | 0 – 1 | 0.55 | Color of that outline (around the color wheel; 0.55 = cyan). |

## Color

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Colorizer** | 0 – 1 | 0 | Sandin-style colorizer on the output: brightness is cut into bands, and each band gets its own color. 0 = off (the plain picture). |
| **Colorizer steps** | 2 – 16 | 4.6 | How many brightness bands, so how many colors. |
| **Colorizer hue** | 0 – 1 | 0 | Shifts all band colors around the color wheel. |
| **Colorizer speed** | -1 – 1 | 0.05 | Lets the band colors cycle over time (turns per second). 0 = fixed colors. |
| **Input color** | 0 – 1 | 0 | Saturation of the input. 0 = black and white in, so color only comes from the tubes, the key edges and the colorizer. |
| **Tube errors** | 0 – 1 | 0.2 | Three-tube misconvergence: red zooms and turns slightly more than green, blue slightly less, so color fringes part over generations (as in optical-feedback-3tubes, in one slider). |
| **Sharpening** | 0 – 3 | 0.5 | Edge sharpening in the camera; its overshoot adds parallel echo lines. |
| **Noise** | 0 – 1 | 0.05 | Grain, fresh every generation. It gives the loop something to grow from in flat areas. |

## Tips

- **Keyed zebra spirals:** the defaults: very high contrast and gain above 1 turn the bright parts of the clip into hard, spiralling feedback bands that zoom outward, while the dark parts stay live, in black and white with light tube fringes and cyan key edges. Turn Colorizer up for color.
- **Feedback only in a brightness band:** Key low 0.3, Key high 0.6, Key edges 1.2.
- **Self-keying loop:** Key source 1, Key low 0.5. The feedback decides where it continues.
- **Motion painting:** Key source 2, Key low 0.1, Key memory 0.9. Movement opens holes that keep spiralling.
- **Zebra in the shadows:** Invert key 1, Black outside 1, Invert per generation 1, Camera contrast 1.8.
- **Black-and-white tube fringes:** Colorizer 0, Tube errors 0.6.
- **Long-lived self-keyed shapes:** Key memory 0.97, Key source 1, Key low 0.6, Colorizer steps 12.

The defaults run the loop hot (gain 1.3, contrast 4). That's what makes the hard bands; together with Key source 1 or a high Key memory it lets the keyed area take over the whole frame and saturate into flat areas. Lower the gain or raise Black level to bring structure back.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
