# Composite video glitch

`composite-video-glitch.fs` is an ISF effect from the videoping shader pack. Composite video with the **faults of tired analog gear**. Underneath is a real NTSC/PAL encode and decode, with dot crawl, rainbows and smeared color. On top, analog processing errors fire as **random events**, at the place in the signal path where they happen in real equipment:
- **Timing:** lines jitter and wobble like a stretched tape. Sync tears rip blocks of lines sideways at a slant. When vertical sync is lost the picture rolls, with the black blanking bar running through it.
- **Color:** the color carrier loses lock, so hues drift and rainbow bands slide across the lines. The color killer flickers the color off, and the color delay jumps.
- **Levels:** bands of lines get a wrong black level, and the gain pumps up and down.
- **Tape:** dropouts turn parts of lines into noise or repeat the line above, as a dropout compensator does. The bottom lines bend away from head switching. Bursts of snow and interference hit the picture.

> ⚠ About 9 ms per frame at 1080p on an Intel Iris Xe (fine on a dedicated GPU).

## Randomness

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Chaos (all faults)** | 0 – 2 | 1 | Master for all faults: scales how often every random event happens. 0 = no faults (plain composite video), 2 = twice as often. |
| **Glitch rate** | 0 – 1 | 0.4 | How often the events re-roll: from about 1.5 times per second (0) to 25 times per second (1). Low = longer, calmer faults; high = nervous flicker. |
| **Random seed** | 0 – 100 | 0 | A different pattern of events. Whole numbers. |

## Faults

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Line jitter** | 0 – 1 | 0.3 | Every line is slightly off in time, fresh every frame: fine shimmering edges. |
| **Tape wobble** | 0 – 1 | 0.25 | Slow, wavy bending of the lines, like a stretched tape or worn transport. |
| **Sync tears** | 0 – 1 | 0.4 | Blocks of lines lose horizontal sync and rip sideways, often slanted, leaving black where the picture was torn off. Higher = more frequent and further. |
| **Vertical roll** | 0 – 1 | 0.2 | Vertical hold lost: from time to time the picture rolls up or down for a while, with the black blanking bar running through. |
| **Color unlock** | 0 – 1 | 0.4 | The color carrier loses lock: hues drift slowly; in events, rainbow bands slide across the lines, the color flickers off, and the color shifts sideways. |
| **Level errors** | 0 – 1 | 0.3 | Black-level errors in bands of lines (lighter or darker stripes) and gain that pumps up and down. |
| **Dropouts** | 0 – 1 | 0.3 | Parts of lines turn into noise or repeat the line above, fresh every frame, like damaged tape. |
| **Head switching** | 0 – 1 | 0.4 | The bottom lines bend sideways and jitter, the VHS head-switching point. |
| **Interference bursts** | 0 – 1 | 0.3 | Short bursts of heavy snow and of interference patterns. |

## Signal

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Standard (0 NTSC, 1 PAL)** | 0 – 1 | 0 | TV standard; changes the look of the dots and rainbows and how the comb filter works. |
| **Signal resolution** | 160 – 1200 | 480 | Signal samples per line. Lower = softer picture with bigger artefacts. |
| **Color carrier filter** | 0 – 1 | 0.6 | How much of the color carrier is filtered out of the brightness. 0 = strong dot patterns. |
| **Comb filter / delay line** | 0 – 1 | 0 | Separation using the neighboring line: NTSC removes the dots; PAL cancels hue errors (without it, color-lock faults show as Hanover bars). |
| **Color blur** | 0 – 1 | 0.5 | How soft the color is. Low = sharp color but more rainbows; high = smeared color. |
| **Color delay** | -1 – 1 | 0.25 | Color shifted sideways against the brightness (Color unlock makes it jump around this value). |
| **Saturation** | 0 – 2.5 | 1.1 | Color strength. |
| **Sharpening (peaking)** | 0 – 2 | 0.5 | Edge enhancement: bright and dark outlines around edges. |
| **Antenna noise** | 0 – 1 | 0.05 | Constant snow. From about 0.35 up the color fades (the color killer); Interference bursts add snow bursts on top. |
| **Mix** | 0 – 1 | 1 | Blend with the original picture. |

## Tips

- **Worn VHS tape:** the defaults.
- **Calm but broken:** Glitch rate 0.1, Chaos 0.7. Faults hang around longer.
- **Total meltdown:** Chaos 2, Glitch rate 0.8.
- **Rolling TV:** Vertical roll 1, Sync tears 0.6, everything else low.
- **Color going crazy:** Color unlock 1, Standard 1 (PAL) with Comb filter 0, the others low.
- **Only timing errors:** Color unlock 0, Level errors 0, Interference bursts 0; Jitter, Wobble and Sync tears up.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
