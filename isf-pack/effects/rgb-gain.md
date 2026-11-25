# RGB gain

`rgb-gain.fs` is an ISF effect from the videoping shader pack. One gain per color channel: 1 leaves it, 0 kills it, above 1 boosts it, below 0 inverts it.

## Sliders

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Red** | −2 – 3 | 1 | Gain of the red channel. |
| **Green** | −2 – 3 | 1 | Gain of the green channel. |
| **Blue** | −2 – 3 | 1 | Gain of the blue channel. |

## Tips

- **No red:** Red 0.
- **Blue blast:** Blue 3.
- **Half negative:** Green -1, Blue -1.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
