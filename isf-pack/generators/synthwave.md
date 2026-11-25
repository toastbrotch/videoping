# Synthwave

`synthwave.fs` is an ISF generator from the videoping shader pack. Horizontal neon lines are emitted at the bottom and travel upward, either flat across the screen or as a perspective floor that recedes to a synthwave horizon.

Each line records a wave signal at the moment it is emitted and keeps that shape as it travels. The floor therefore shows the history of the signal, like a seismograph or a pile of oscilloscope traces. On top of that come a sun and sky, a vertical floor grid, eight palettes, a Weird control and a Glitch control.

> ⚠ About 18 ms per frame at 1080p on an Intel Iris Xe (fine on a dedicated GPU).

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Rate (lines/s)** | 0.2 – 40 | 4 | How many lines are emitted per second. The distance between lines is Speed divided by Rate, so a higher Rate packs them closer together. |
| **Speed (<0 = downwards)** | -4 – 4 | 1 | Travel speed. Positive moves bottom → top (in perspective: away from you, toward the horizon). Negative reverses it, so lines come from the horizon toward you. Around 0 the lines freeze and bunch up tightly. |
| **Perspective** | 0 – 1 | 1 | 0 = flat lines moving straight up the screen (they also cross the sky); 1 = a full perspective floor where lines bunch up toward the horizon. Values in between give a half-tilted floor. |
| **Horizon height** | 0.15 – 0.95 | 0.55 | Height of the horizon line, and with it the sun. |
| **Wave type (0-5)** | 0 – 5 | 1 | Shape of the signal on the lines; see the table below. Values in between blend the two neighboring types. |
| **Wave height** | 0 – 1 | 0.3 | Wave amplitude. 0 gives perfectly straight lines (classic grid floor); high values give huge mountain ridges. |
| **Wave frequency** | 0.5 – 40 | 6 | How many wiggles fit across a line. Low gives broad hills, high gives dense noise. |
| **Wave motion** | 0 – 6 | 1 | How fast the signal changes over time. 0 gives every line the same frozen shape (calm mountain ranges); high values make each line look completely different from the one before. |
| **Hidden lines** | 0 – 1 | 0 | Hides lines behind higher ridges in front of them, in the style of the Joy Division "Unknown Pleasures" cover. 0 = all lines glow through each other; 1 = solid ridges. |
| **Line width (px)** | 0.5 – 24 | 2 | Line thickness at the bottom of the screen. With perspective, lines get thinner toward the horizon. |
| **Glow** | 0 – 3 | 1 | Neon glow around lines and grid. |
| **Floor grid** | 0 – 60 | 12 | Number of vertical floor grid lines running toward the horizon. 0 switches them off. |
| **Palette (0-7)** | 0 – 7 | 0 | Color scheme; see the table below. Rounded to the nearest whole number. |
| **Sun** | 0 – 1 | 1 | Brightness of the striped synthwave sun and its glow. 0 leaves only the sky. |
| **Weird** | 0 – 1 | 0 | Bends the world; see below. |
| **Glitch** | 0 – 1 | 0 | Digital breakdown; see below. |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the plain sky and floor shows. 0 = transparent: the lines, grid, sun and solid ridges stay, and in the mixer (mix type A) the layer below shows through the rest. |

## Wave types

| Value | Name | Look |
|---|---|---|
| 0 | Sine | Smooth, regular sine waves |
| 1 | Seismic | A quiet jittery line with quake bursts that come and go and wander sideways |
| 2 | Audio | An oscilloscope signal that pumps on a beat, strongest in the center |
| 3 | Digital | Hard square steps at random levels, like a Tron city |
| 4 | Zigzag | Sharp triangle zigzags |
| 5 | Heartbeat | Heart-monitor (ECG) spikes |

## Palettes

| Value | Name | Colors |
|---|---|---|
| 0 | Synthwave | Magenta lines turning cyan in the distance, purple sky, yellow-pink sun |
| 1 | Outrun | Orange to magenta, sunset sky |
| 2 | Vaporwave | Pastel mint and pink |
| 3 | Matrix | Greens on black |
| 4 | Ice | White and blue |
| 5 | Fire | Yellow to red |
| 6 | Mono | White on black |
| 7 | Rainbow | Every line its own color, drifting over time |

## Weird

Weird starts subtle and gets stronger as it rises:

- The horizon rolls from side to side and bends into a wave.
- Lines melt and bend more the further they travel.
- Valley walls rise at the sides of the floor.
- Line colors shift.
- From about 0.5 up, a second, mirrored floor fades in as a ceiling in the sky, turning the scene into a tunnel.

## Glitch

Glitch changes many things at once, and they all re-roll faster as the value rises: from 3 to 23 times per second.

- Horizontal bands tear sideways.
- Some bands stutter in time.
- The red, green and blue parts of the lines split apart vertically.
- Single lines drop out, flash white and thick, or spike to triple height.
- The horizon jumps.
- Striped noise blocks appear.
- Thin bands flash inverted.
- Scanlines flicker.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
