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
| `VIDEOPING_SELFTEST=<dir>` | self-test: builds a session (id-display generator over a clip with gameboy-camera), writes `out_WxH.rgba` (output, rows bottom-up) and `ui_WxH.rgba` (screenshot) into `<dir>`, also simulates mouse input (drags the "[" limiter, clicks "Lk" of mixer layer L2, clicks an empty prepare slot, opens "new" and clicks "clear everything", blackout + click on the brightness slider) and prints `selftest drag: …` / `selftest click: …` / `selftest empty slot click: …` / `selftest new: …` / `selftest fade in: …`, quits after ~7 s. Never touches the autosave or the stored ui layout. |
| `VIDEOPING_SELFTEST_CLIP=<mp4>` | clip for the self-test (default `<dir>/testclip.mp4`) |
| `VIDEOPING_SELFTEST_EFFECT=<fs>` | first (open) clip effect of the self-test (default `isf-effect/gameboy-camera.fs`) |
| `VIDEOPING_SELFTEST_RESET=1` | change the first parameter of the first clip effect, right click its row, print value before / after (expected: default) |
| `VIDEOPING_SELFTEST_NOMETERS=1` | self-test with the resource meters off (overhead comparison: `time` both runs); with meters on, the self-test prints the meter of each instance |
| `VIDEOPING_SELFTEST_REC=<seconds>` | record from 1.5 s to the given time (default 4) into the self-test folder (without it the recording folder is the default and nothing is recorded); the self-test stays open until then; check the file with `cargo run --release --example transport_test -- <file>` |
| `VIDEOPING_SELFTEST_REC_OFF=1` | same timing without recording (CPU comparison) |
| `VIDEOPING_SELFTEST_PREPARE=<generator.fs>` | put that generator into slot 3 and select it (not live): prepare priority test, prints its renders per second |
| `VIDEOPING_SELFTEST_NOPRIORITY=1` | old behaviour for comparison: prepare work every frame |
| `VIDEOPING_SELFTEST_SHADERS=<folder>` | compile every `.fs` in that folder on the GPU, print ok / FAIL per file, quit |
| `VIDEOPING_SELFTEST_FPS=25` | self-test with this output frame rate; prints the measured video and ui fps (from second 2 on) |
| `VIDEOPING_SELFTEST_FX=1` | instead of the default input script: drags the first clip effect by its "≡" handle onto the output effects and prints the effect counts before / after |
| `VIDEOPING_SELFTEST_NDI=1` | self-test also sends NDI and receives its own output as NDI input (slot 17, selected) |

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
cargo run --example ndi_loop_test -- [ndi lib dir] # NDI loopback: send, discover, receive, check pixels
```

# documentation

| file | for whom |
|---|---|
| `features.md` | the spec: every design decision, thresholds, measurements, open points |
| `manual.html` | end users: how to play with videoping; one self-contained HTML page (styles inline), screenshots in `docs/`; open it in a browser |
| `install.md` | installation only |
| `docs/logo/` | the logo (v1 calm, v2 glitch = official): SVG, PNG sizes, editable `videoping-logo.vectorcraft` (VectorCraft) |
| `packaging/videoping.desktop` | launcher entry; icons and install commands in install.md |
| `dev.md` | this file |

Screenshots for the manual come from the self-test (it uses only the repo shaders and the test clip
in the finder, and clears the status line before the screenshot):

```sh
S=/tmp/vp-docs; mkdir -p $S; cp <testclip.mp4> $S/testclip.mp4
VIDEOPING_SELFTEST=$S ./target/release/videoping   # no REC: the record box shows the default folder
U=$S/ui_3000x1818.rgba   # size depends on the screen
c(){ ffmpeg -y -f rawvideo -pix_fmt rgba -s 3000x1818 -i $U -vf "$1" docs/$2; }
c "scale=1500:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=160[p];[b][p]paletteuse=dither=none" overview.png
c "crop=1300:36:1700:0" topbar.png
c "crop=390:1300:0:36" finder.png
c "crop=620:1440:392:36" prepare.png
c "crop=890:1700:1022:36" detail.png
c "crop=1080:1100:1920:36" mixer.png
```

(the crop rectangles fit a 3000x1818 window; detail.png and mixer.png also go through the palette
filter to stay small)

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
  ndi.rs       NDI: libndi.so loaded once at runtime; sender, finder (discovery), receiver (one thread each)
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
window (fullscreen), NDI output, NDI input (finder discovery, receive), autosave + save / load, duplicate, drag & drop.

not yet: patch-bay, midi (bhoreal, twister), videopong / online shaders, sets,
audio, api / mcp, other tabs.
