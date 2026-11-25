# Frame difference

`frame-diff.fs` is an ISF effect from the videoping shader pack. It shows **what changed**: moving parts of the picture light up, still areas go black, like the FrameDiff FreeFrame plugin.

It has two ways of deciding what counts as "changed":
- **Exact (Mode 0):** compares each frame with the frame 1, 2 or 3 frames ago, as the plugin does. Moving edges light up as sharp outlines, doubled by the delay.
- **Memory (Mode 1):** compares with a slowly updating average of the recent past instead, which remembers anywhere from a tenth of a second to ten seconds. Motion glows. Things that stop moving fade out. Things that leave the picture leave a ghost of where they were, until the memory catches up.

It delays at most 3 frames; for longer delays, use Memory mode.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Mode (0 exact, 1 memory)** | 0 – 1 | 0 | Below 0.5: compare with a frame a fixed number of frames ago (Delay). From 0.5: compare with the running average (Memory). |
| **Delay (frames)** | 1 – 3 | 1 | Exact mode: how many frames back to compare with. More = thicker, doubled outlines on fast motion. |
| **Memory** | 0 – 1 | 0.3 | Memory mode: how long the average remembers, from about a tenth of a second (0) to ten seconds (1). Short = only fast motion lights up; long = slow motion lights up too, and departures leave long-lasting ghosts. Counted in seconds, so the same at any frame rate. |
| **Hardness** | 0 – 1 | 0 | Contrast of the result, as in the plugin. 0 = soft gray, proportional to how much changed; 1 = any change at all goes straight to white. |
| **Color** | 0 – 1 | 0 | 0 = the difference in gray (as in the plugin); 1 = in color, so a red object moving over blue lights up in the colors that changed. |
| **Original underneath** | 0 – 1 | 0 | Mixes the original picture in underneath, so only the motion glows on top of a visible image. 0 = black background, as in the plugin. |

## Tips

- **The plugin's look:** the defaults (exact, 1 frame).
- **Hard motion outlines:** Delay 2, Hardness 0.9.
- **Motion detector / ghost trails:** Mode 1, Memory 0.8, Color 1.
- **Glowing motion over the picture:** Mode 1, Memory 0.3, Original underneath 0.4.

- **Heat camera of the motion:** frame-diff → gradient-map (thermal).
- **Motion with spiralling trails:** frame-diff → feedback.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
