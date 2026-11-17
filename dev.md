videoping — development
=======================

`features.md` is the spec. Every design decision goes there first.

# setup

## rust

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # if not installed
rustup update stable                                              # needs Rust >= 1.92 (edition 2024)
```

## system packages (Ubuntu / Debian / Pop!_OS)

Runtime packages from `install.md`, plus:

```sh
sudo apt install build-essential pkg-config \
  libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev
```

That is all: the windowing libs (wayland, x11, xkbcommon) and OpenGL are loaded at runtime,
and NDI is loaded with `dlopen` (no NDI headers needed to build).

### without root (fallback)

If you can't install the `-dev` packages, extract them locally and point pkg-config there
(the runtime libs of the same version must be installed):

```sh
mkdir -p ~/gstdev && cd ~/gstdev
apt-get download libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev
for f in *.deb; do dpkg-deb -x "$f" root; done
L=$PWD/root/usr/lib/x86_64-linux-gnu
for l in $L/*.so; do ln -sf /usr/lib/x86_64-linux-gnu/$(readlink $l) $l; done
sed -i "s#^libdir=.*#libdir=$L#; /^Requires.private/d" $L/pkgconfig/*.pc
export PKG_CONFIG_PATH=$L/pkgconfig
```

# build, run, test

```sh
cargo run                 # debug build
cargo run --release       # use this for real playback performance
cargo test                # unit tests (state logic, isf parsing)
cargo clippy
```

Useful env vars:

| var | effect |
|---|---|
| `NDI_RUNTIME_DIR_V6` | folder with `libndi.so.6` |
| `GST_DEBUG=3` | GStreamer warnings (codec / decoding problems) |
| `VIDEOPING_SELFTEST=<dir>` | self-test: builds a session (id-display generator over a clip with gameboy-camera), writes `out_WxH.rgba` (output, rows bottom-up) and `ui_WxH.rgba` (screenshot) into `<dir>`, quits after ~7 s. Never touches the autosave or the stored ui layout. |
| `VIDEOPING_SELFTEST_CLIP=<mp4>` | clip for the self-test (default `<dir>/testclip.mp4`) |
| `VIDEOPING_SELFTEST_NDI=1` | self-test also sends NDI |

Self-test end to end:

```sh
d=/tmp/vp-selftest; mkdir -p $d
ffmpeg -f lavfi -i testsrc2=size=1280x720:rate=30 -t 8 -c:v libx264 -pix_fmt yuv420p $d/testclip.mp4
VIDEOPING_SELFTEST=$d cargo run
ffmpeg -f rawvideo -pix_fmt rgba -s 1920x1080 -i $d/out_1920x1080.rgba -vf vflip $d/out.png
```

Dev examples (no ui):

```sh
cargo run --example transport_test -- clip.mp4   # forward / backward / bounce / bpm against GStreamer
cargo run --example ndi_test -- /path/to/ndi/lib  # sends 2 s of test frames as "videoping-test"
```

# architecture

```
src/
  main.rs      eframe setup (glow / OpenGL renderer)
  app.rs       the "live" tab: 5 columns, drag & drop, autosave, output window, self-test
  widgets.rs   range slider, transport row, timeline, mix controls, texture painting
  state.rs     Session = everything that is saved (serde); animation logic (AnimParam, Speed)
  render.rs    render engine: instances, effect chains, mixer, output effects, master, NDI readback
  gfx.rs       OpenGL helpers: Target (texture+fbo), Program, Quad, built-in shaders (blit/mix/master)
  isf.rs       ISF parser + GLSL wrapper (files on disk stay unchanged)
  video.rs     GStreamer playbin -> appsink RGBA; transport via rate seeks
  ndi.rs       NDI sender (libndi.so loaded at runtime, own thread)
```

Frame flow (all on the ui thread, inside `App::ui`, same GL context as egui):

```
for each active instance (live in a layer, selected, or still without thumbnail):
    source  : mp4 frame upload -> flip blit  |  isf generator (all PASSES)
    effects : isf effect -> mix(input, effect, mix-type/amount/rgb) -> next
    thumb   : blit into 256x144
mixer  : black <- L3 <- L2 <- L1   (mix shader)
output : output effects -> freeze buffer -> master (brightness / blackout) -> out
NDI    : flip blit -> glReadPixels into PBO (async, 1 frame late) -> sender thread
egui   : paints textures via register_native_glow_texture (bottom-up -> flipped uv)
```

## ISF support

- the `.fs` is wrapped for `#version 330`: uniforms are declared, `gl_FragColor` →
  `isf_FragColor`, `varying` → `in`, `texture2D` → `texture`, `IMG_*` macros use `textureSize`.
- inputs: float, long (VALUES/LABELS → dropdown), bool, event, color, point2D, image
  (`inputImage` = effect input, other images: black for now → patch-bay later), audio (black).
- `PASSES` with `TARGET` (all targets persist between frames), optional `.vs` next to the `.fs`.
- extra uniforms: `uInstanceSeed` (per instance, saved), `TIME`, `TIMEDELTA`, `RENDERSIZE`,
  `PASSINDEX`, `FRAMEINDEX`, `DATE`.
- pass `WIDTH`/`HEIGHT` expressions and `FLOAT` targets are not supported yet (full size, 8 bit).
- shaders hot-reload when the file changes (checked every second); compile errors are printed
  to stderr and shown as ⚠ / red text in the ui.

## gotchas

- edition 2024: `gen` is a reserved keyword.
- `move` closures capture only the fields they use (since edition 2021). The NDI thread must
  hold the whole `Lib` (`let lib = lib;`), otherwise libndi is unloaded and crashes after ~1 s.
- GL textures are bottom-up; video frames and NDI are top-down → flip blits.
- egui API is 0.37 (`App::ui(&mut self, ui, frame)`, `egui::Panel`, `show(ui, ..)`).

# status of the prototype (milestone 1)

done: live tab with 5 resizable columns, finder (mp4 / isf generator / isf effect, search, folders),
prepare 4x8 grid with pages, detail panel (preview, clip transport, generator params, effect stack),
3 mixer layers, bpm tapper, master (blackout / freeze / brightness), output effects, output
window (fullscreen), NDI output, autosave + save / load, duplicate, drag & drop.

not yet: NDI input, patch-bay, midi (bhoreal, twister), videopong / online shaders, sets,
audio, api / mcp, other tabs.
