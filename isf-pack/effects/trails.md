# Trails

`trails.fs` is an ISF effect from the videoping shader pack. Video feedback: every frame is laid over the last ones, so movement leaves trails. With zoom, turn and drift the trails become tunnels, spirals and smoke; hue drift colors each echo.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Trail length** | 0 – 1 | 0.8 | How long the trails last: 0 = none, 1 = several seconds. |
| **Mode** | choice | lighten (bright parts trail) | **lighten**: bright parts leave trails on dark; **blend**: everything smears (motion blur); **add**: trails add up and burn out to white. |
| **Zoom per frame** | -1 – 1 | 0 | Each echo is zoomed a little: positive = a tunnel towards you, negative = away. |
| **Turn per frame** | -1 – 1 | 0 | Each echo is turned a little: spirals. |
| **Drift per frame** | position | 0, 0 | Each echo moves a little: smoke, falling or rising streams. |
| **Center** | position | 0.5, 0.5 | The point the zoom and turn go around. |
| **Hue drift** | -1 – 1 | 0 | Each echo changes color a little: rainbow trails. |
| **Smear (blurs the echoes)** | 0 – 1 | 0 | Blurs the echoes, so they dissolve like smoke. |
| **Clear** | trigger | – | Wipes the trails. Map it to a pad. |

## Tips

- Trail length 0.95 + zoom 0.5 + turn 0.4 = an endless spiral tunnel.
- The trails run at the same speed at any frame rate.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
