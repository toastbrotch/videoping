# Fold colors

`fold-colors.fs` is an ISF effect from the videoping shader pack. Folds every channel's brightness back on itself many times (a solarizer on steroids), cuts it into levels and spins the hue. At the top it becomes psychedelic color chaos.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Fold** | 0 – 1 | 0.3 | How many times the brightness is folded back on itself. |
| **Steps** | 2 – 32 | 32 | Number of brightness levels after folding. 32 = smooth. |
| **Hue spin** | 0 – 1 | 0 | Turns the hue. |

## Tips

- **Solarized:** Fold 0.2.
- **Psychedelic:** Fold 1, Hue spin 0.5.
- **Poster folds:** Fold 0.5, Steps 4.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
