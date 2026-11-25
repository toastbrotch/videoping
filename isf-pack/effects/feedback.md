# Feedback

`feedback.fs` is an ISF effect from the videoping shader pack. Classic video feedback: the previous output is zoomed, turned and shifted and mixed back over the picture. Low = echoes; around 1 the loop takes over (tunnels, spirals, streams); above 1 it builds up until it burns out to white (a little of the input always stays in).

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Feedback** | 0 – 1.2 | 0.7 | Amount of feedback. Around 1 the loop takes over; above 1 it builds up. |
| **Zoom** | −1 – 1 | 0.3 | Zoom per frame; negative = the other way. |
| **Turn** | −1 – 1 | 0 | Turn per frame; negative = the other way. |
| **Shift X** | −1 – 1 | 0 | Sideways shift per frame: the feedback streams left or right. |
| **Shift Y** | −1 – 1 | 0 | Vertical shift per frame: the feedback streams up or down. |

## Tips

- **Echo tunnel:** the defaults.
- **Spiral:** Feedback 1, Zoom 0.4, Turn 0.3.
- **Stream:** Feedback 1, Zoom 0, Shift Y 0.3.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
