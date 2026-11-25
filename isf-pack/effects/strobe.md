# Strobe

`strobe.fs` is an ISF effect from the videoping shader pack. Flashes the picture: to black, to white, inverted, to a color, or freezes it. Free running in flashes per second, or exactly on the beat by animating Phase with the bpm.

> ⚠ Fast flashing can trigger seizures in people with photosensitive epilepsy. Keep fast, high-contrast strobes short.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Flashes per second (0 = by phase)** | 0 – 30 | 2 | Flashes per second. 0 = the flash follows Phase only. |
| **Phase** | 0 – 1 | 0 | Where in the flash cycle it is. Animate it in videoping with the bpm (loop 0 → 1, one beat) and set the rate to 0: the strobe flashes on the beat. |
| **Flash length** | 0 – 1 | 0.25 | How long each flash is, as a part of the cycle. |
| **Flash to** | choice | black | **black**, **white**, **inverted**, **color**, **freeze** (the picture stops during the flash), **transparent** (the layer disappears during the flash). |
| **Fade (soft flash)** | 0 – 1 | 0 | 0 = hard flashes, higher = soft pulses. |
| **Random** | 0 – 1 | 0 | Skips flashes at random: an irregular, nervous strobe. |
| **Flash color** | color | #ffffff | Color for the flash mode color. |

## Tips

- Freeze + rate 4 + flash length 0.5 = a stutter effect.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
