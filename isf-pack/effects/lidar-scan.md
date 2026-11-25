# LiDAR scan

`lidar-scan.fs` is an ISF effect from the videoping shader pack. The picture becomes a 3D point cloud: brightness is height, the picture is raised into a relief, and the points sit on a grid over it.

- **Real spheres.** Every measured point is a 3D sphere sitting on the relief, with diffuse light, a specular highlight and a rim. Up close, the spheres are as big as they really are and hide what is behind them.
- **Four cameras:** an orbit around the cloud, a flight across it, a dive into it and through it, and a fixed tripod inside it. The cloud can also repeat endlessly, so a flight never runs out of points.
- **Overlay:** a scan plane sweeps through, with a ground grid, axes and telemetry (also the camera altitude and mode).

> ⚠ About 15 ms per frame at 1080p on an Intel Iris Xe with the defaults; a low flight over an endless, dense cloud of tiny spheres without fog can take 40 ms. Lower Point density or raise Depth fog if the frame rate drops.

## Sliders

### Point cloud

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Point density** | 20 – 320 | 90 | Spheres from top to bottom of the picture (across scales with the aspect ratio). Low = big spheres you can fly between; high = a fine cloud. |
| **Sphere size** | 0.05 – 0.5 | 0.38 | Sphere size relative to the spacing. 0.5 = neighboring spheres touch. |
| **Height** | 0 – 1 | 0.35 | How high bright areas rise. 0 = a flat sheet of spheres. |
| **Smoothing** | 0 – 1 | 0.4 | Smooths the relief. Low = jagged; high = soft hills. |
| **Measurement noise** | 0 – 1 | 0.15 | Random offset of every sphere, like the measuring noise of a real scanner. 0 = a perfect grid. It has less room with large spheres, because a sphere always stays inside its own grid cell. |
| **Sphere shading** | 0 – 1 | 1 | 0 = flat discs in one color (still with correct size and overlap); 1 = fully lit spheres. |
| **Highlight** | 0 – 1 | 0.5 | Strength of the specular highlight on each sphere. |
| **Coloring (0 height, 1 picture)** | 0 – 1 | 0 | Sphere colors: 0 = by height (blue to red), 1 = the picture's own colors. |

### Camera

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Camera (0 orbit, 1 fly-over, 2 dive, 3 tripod)** | 0 – 3 | 0 | **0 Orbit:** circles around the cloud. **1 Fly-over:** flies straight across the cloud in the Heading, at Flight height, and loops (with Endless on, the default, it flies on forever; with Endless off, it crosses the single picture and loops). **2 Dive:** comes in from far away and high up, dives into the cloud and through the layer of spheres, then starts over. **3 Tripod:** stands still at Tripod X/Z and Flight height and pans around. |
| **Speed** | -1 – 1 | 0.15 | Orbit: turning speed. Fly-over and dive: flight speed. Tripod: panning speed. Negative = the other way (a dive runs backwards, out of the cloud). 0 = still. |
| **Tilt** | 0 – 1 | 0.45 | Orbit: camera elevation (low = from the side, 1 = from above). Fly-over and tripod: how far the camera looks down; below 0.2 it looks slightly up. Dive: how steep the dive is (low = flat approach, 1 = straight down). |
| **Heading** | 0 – 1 | 0 | Direction around the cloud, a full circle. Sets the flight direction in Fly-over and dive, the starting direction for the tripod, and shifts the orbit. |
| **Zoom** | 0.4 – 2.5 | 0.75 | Orbit and dive: camera distance (higher = closer). Fly-over and tripod: lens zoom. |
| **Wide angle** | 0 – 1 | 0.4 | Field of view. High = wide angle, strong perspective; very good for flying close to the spheres. |
| **Flight height** | 0 – 1 | 0.3 | Fly-over and tripod: camera height. Low values put the camera between the spheres. Dive: how high in the relief the dive aims. |
| **Weaving** | 0 – 1 | 0.3 | Fly-over and dive: the camera weaves sideways and banks into the turns. 0 = straight flight. |
| **Tripod X** | -1 – 1 | 0 | Tripod: camera position from left to right over the picture. |
| **Tripod Z** | -1 – 1 | 0 | Tripod: camera position from front to back. |
| **Endless (tiled)** | on/off | on | Repeats the cloud in all directions, mirrored so there are no seams. For endless flights. |
| **Depth fog** | 0 – 1 | 0.35 | Spheres and grid fade into the dark with distance. Higher also makes it faster, because distant spheres are not calculated. |

### Overlay

| Slider | Range | Default | What it does |
|---|---|---|---|
| **Scan** | 0 – 1 | 0.5 | Speed of the scan plane sweeping through; spheres it passes flash white. 0 = off. |
| **Ground grid** | 0 – 1 | 0.5 | Visibility of the ground grid and axes. |
| **Labels** | 0 – 1 | 1 | Telemetry on (above 0.5) or off. |
| **Text size** | 1 – 4 | 2 | Size of the pixel font. It also scales with the output resolution. |
| **HUD color** | 0 – 1 | 0.5 | Color of grid, rim light, scan glow and telemetry. |

The dive loop (and, with Endless off, the fly-over loop) fades briefly to black where it starts over, so the jump back is not a hard cut.

## Tips

- **Sphere cloud:** the defaults.
- **Single plate:** Endless off, for one picture-sized cloud floating over the grid, for a classic scan.
- **Ball pit:** Point density 40, Sphere size 0.5, Coloring 1, Highlight 1.
- **Low flight:** Camera 1, Flight height 0.15, Tilt 0.25, Speed 0.3, Wide angle 0.7, Height 0.6.
- **Endless landscape:** Camera 1, Flight height 0.4, Tilt 0.35, Weaving 0.6, Depth fog 0.5.
- **Dive:** Camera 2, Speed 0.3, Tilt 0.5, Weaving 0, Point density 60.
- **Inside the cloud:** Camera 3, Flight height 0.1, Tilt 0.25, Wide angle 0.8, Speed 0.1, Coloring 1.
- **Old school:** Sphere shading 0, Point density 120, Sphere size 0.3 (flat dots).

Brightness is height. For people in front of a bright background, put invert (brightness) before it so the subject rises out of the plane.

License: CC BY-NC-SA 4.0 (see LICENSE-shaders.md in the videoping repository).
