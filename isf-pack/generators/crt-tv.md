# CRT TV

`crt-tv.fs` is an ISF generator from the videoping shader pack. One curved CRT tube in a TV cabinet that zaps between channels, or stays on one channel you choose.

Each channel is one of three kinds:
- an oscilloscope with a glowing Lissajous figure (green or amber phosphor)
- a drifting test card (SMPTE color bars or a resolution chart)
- a dead channel full of static

On a **channel change** the picture collapses into a bright horizontal line, static bursts, and the next channel opens out, while the power LED blinks red.

A share of the channels (Overlay share) comes with one fixed overlay, which curves with the glass; the rest show their picture clean. Tuning to a channel always gives the same picture with the same overlay:
- sync roll with power flicker
- a scan sweep
- grain
- an ASCII/terminal conversion of the picture
- Matrix-style falling characters
- a spacecraft docking-camera HUD

When the tube fills the frame, its anatomy shows: scanlines, an RGB shadow mask, phosphor glow, vignette and a reflection on the glass.

> ⚠ About 16 ms per frame at 1080p on an Intel Iris Xe (fine on a dedicated GPU).

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Speed** | 0 – 3 | 1.84 | Speed of everything: channel changes, scope figures, overlays. 0 freezes the picture. |
| **Channel change rate (0 = never)** | 0 – 1 | 0.06 | How often the channel changes: 0 = never (the TV stays on the channel set by Channel); just above 0 = every 30 seconds, 1 = every 3 seconds. Speed speeds this up too. |
| **Channel** | 0 – 30 | 0 | Which channel the TV shows (whole numbers; what's on each channel depends on Random seed). With channel changes on, it's the channel the zapping starts from. |
| **Channel change effect** | 0 – 1 | 0 | Strength of the channel-change effect (collapse to a line, static burst, red LED). 0 = hard cuts between channels. |
| **VHS glitch** | 0 – 1 | 1 | Tracking stripes and block dropouts on the picture. |
| **Color noise** | 0 – 1 | 0.95 | Color noise and bright specks. |
| **Scope glow** | 0 – 1.5 | 1.5 | Brightness of the oscilloscope trace and its afterglow. |
| **Overlay share** | 0 – 1 | 0.5 | Share of channels that come with an overlay. 0 = no channel has one, 1 = every channel has one. Which overlay a channel gets depends on Random seed. |
| **Overlay strength** | 0 – 1.2 | 1.2 | Strength of the channels' overlays. 0 = no overlays, just the pictures. |
| **Hue drift** | 0 – 1 | 0.99 | Slow drift of the colors, so the picture never settles on one hue. 0 = true colors. |
| **Tube size** | 0.4 – 1.2 | 0.976 | Size of the tube in the frame. Small = more cabinet around it; above 1 the tube fills the frame and its edges are cut off. |
| **Tube curvature** | 0 – 1 | 0.3 | Curvature of the glass. The corners stay in place; the picture bulges towards the middle. 0 = flat screen. |
| **Cabinet** | 0 – 1 | 0 | Visibility of the cabinet around the tube: 0 = black, 1 = lit plastic with a bright bezel edge and the power LED. |
| **Scanlines** | 0 – 1 | 0 | Visible scanlines. |
| **Shadow mask (RGB)** | 0 – 1 | 0.12 | The red-green-blue phosphor stripes of the shadow mask. Most visible close up (large tube size). |
| **Phosphor glow** | 0 – 1 | 1 | Bright parts glow beyond themselves. |
| **Glass reflection** | 0 – 1 | 1 | A soft reflection on the upper left of the glass. |
| **Random seed** | 0 – 100 | 7 | Different order of channels and overlays. |
| **Background (0 = transparent)** | 0 – 1 | 1 | How much of the cabinet around the tube shows. 0 = transparent: only the tube stays, and in the mixer (mix type A) the layer below shows around it. |

## Tips

- **Clean channel zapping:** Overlay share 0, VHS glitch 0.2.
- **Overlay on every channel:** Overlay share 1.
- **Retro close-up:** Tube size 1.2, Scanlines 1, Shadow mask 1.
- **Small TV in the dark:** Tube size 0.5, Cabinet 0.3.
- **Nervous zapping:** Channel change rate 1, Speed 1.5.
- **Stay on one channel:** Channel change rate 0, then step through Channel to pick a scope figure or test card you like.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
