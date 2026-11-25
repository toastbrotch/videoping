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
  libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
  libasound2-dev
```

`libasound2-dev` is for MIDI (ALSA, crate `midir`).

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
export PKG_CONFIG_PATH=$L/pkgconfig:/usr/lib/x86_64-linux-gnu/pkgconfig   # keep the system path: alsa
```

# build, run, test

Only one videoping runs at a time (lock `$XDG_RUNTIME_DIR/videoping.lock`); the self-test is exempt
and does not touch the real MIDI controllers, so it can run beside a session (but a session window in
front of it slows it down on Wayland: hidden windows get almost no frames).

```sh
cargo run                 # debug build
cargo run --release       # use this for real playback performance
cargo test                # unit tests (state logic, isf parsing)
cargo clippy
cargo audit               # known vulnerabilities in the dependencies (cargo install cargo-audit)
```

Shaders: `scripts/render-shaders.py <outdir> <file.fs>... [--set P=value] [--wait s] [--width px]` loads
shaders into a self-test videoping (via MCP), saves two frames of each as PNG, prints the GPU time of the
frame and compile errors. The shader pack lives in `isf-pack/generators/` (committed, CC BY-NC-SA).

`scripts/mcp-test.sh` also checks that browsers are kept out of the MCP server (look-alike Origin
/ Host, `Origin: null`, `text/plain` -> 403 / 415) and that files outside the finder are refused.

Useful env vars:

| var | effect |
|---|---|
| `NDI_RUNTIME_DIR_V6` | folder with `libndi.so.6` |
| `GST_DEBUG=3` | GStreamer warnings (codec / decoding problems) |
| `VIDEOPING_SELFTEST=<dir>` | self-test: builds a session (id-display generator over a clip with gameboy-camera), writes `out_WxH.rgba` (output, rows bottom-up) and `ui_WxH.rgba` (screenshot) into `<dir>`, also simulates mouse input (drags the "[" limiter, clicks "Lk" of mixer layer L2, clicks an empty prepare slot, opens "new" and clicks "clear everything", blackout + click on the brightness slider) and prints `selftest drag: …` / `selftest click: …` / `selftest empty slot click: …` / `selftest new: …` / `selftest fade in: …`, quits after ~7 s. Never touches the autosave or the stored ui layout. |
| `VIDEOPING_SELFTEST_CLIP=<mp4>` | clip for the self-test (default `<dir>/testclip.mp4`) |
| `VIDEOPING_PACK=<dir>` | where the shader pack is (default: `<exe>/../share/videoping/isf-pack`, the repository when run from `target/`, `./isf-pack`); a new session puts it into the finder |
| `VIDEOPING_SELFTEST_EFFECT=<fs>` | first (open) clip effect of the self-test (default `isf-effect/gameboy-camera.fs`) |
| `VIDEOPING_SELFTEST_RESET=1` | change the first parameter of the first clip effect, right click its row, print value before / after (expected: default) |
| `VIDEOPING_SELFTEST_NOMETERS=1` | self-test with the resource meters off (overhead comparison: `time` both runs); with meters on, the self-test prints the meter of each instance |
| `VIDEOPING_SELFTEST_REC=<seconds>` | record from 1.5 s to the given time (default 4) into the self-test folder (without it the recording folder is the default and nothing is recorded); the self-test stays open until then; check the file with `cargo run --release --example transport_test -- <file>` |
| `VIDEOPING_SELFTEST_REC_OFF=1` | same timing without recording (CPU comparison) |
| `VIDEOPING_SELFTEST_PREPARE=<generator.fs>` | put that generator into slot 3 and select it (not live): prepare priority test, prints its renders per second |
| `VIDEOPING_SELFTEST_NOPRIORITY=1` | old behaviour for comparison: prepare work every frame |
| `VIDEOPING_SELFTEST_L1MIX=a\|lk\|ilk\|ad` | mix-type of layer 1 (the generator over the clip; default lk), check the result in `out_*.rgba` |
| `VIDEOPING_SELFTEST_L1=clip` | clip in layer 1 over the generator (several brightness levels, for key checks) |
| `VIDEOPING_SELFTEST_L1AMOUNT=<0..100>` | mix of layer 1 in % (luma fade check with `_L1MIX=lk\|ilk`) |
| `VIDEOPING_SELFTEST_MIDI=1` | a virtual MIDI device "vp-selftest pads" (ALSA, Bhoreal profile; named so that no other running videoping picks it up) presses pads (select L3, load the copy of the generator, blackout, effect off), the LED feedback is checked, then cc 20 is learned onto /master/freeze and used, the orientation is calibrated, and a virtual "vp-selftest twister" turns and pushes encoder 1 (/sel/param/1 of the focused effect); the mouse script is off |
| `VIDEOPING_SELFTEST_TAB=midi` | start in the midi mapping tab (screenshot of the tab; with `_MIDI` the virtual device is selected) |
| `VIDEOPING_SELFTEST_MCP=<port>` | MCP server on that port (otherwise off in the self-test); `_MCP_RO=1`: read only |
| `VIDEOPING_SELFTEST_STAY=<seconds>` | keep the self-test open that long (for calls from outside, e.g. `scripts/mcp-test.sh <port> <outdir>`) |
| `VIDEOPING_SELFTEST_CONTROL=1` | at 2 s: drive the session through the control registry (6 sets incl. a layer alias and an invalid address), print the result and the reported changes |
| `VIDEOPING_SELFTEST_CONFIG=1` | output options and general config folded open, column 2 scrolled to the bottom (screenshot of the settings) |
| `VIDEOPING_SELFTEST_AI=<url>` | the ai tab talks to a mock Anthropic server at that url (key "selftest"); at 2 s it sends a message, at 5.5 s it prints the brightness and the chat. The mock must answer with `set /master/brightness 0.5` + `frame`, then a text (see features.md "ai tab") |
| `VIDEOPING_SELFTEST_FINDER=<search>` | two favorites (gameboy-camera, chaos-feedback) and that search typed into the finder (screenshot of the accordion / search; empty = favorites only) |
| `VIDEOPING_SELFTEST_QUIT=1` | the quit question is active in the self-test: at 2 s the window's close is requested, at 3 s it must still run with the question open; then it quits (mouse script off) |
| `VIDEOPING_SELFTEST_LONGNAME=1` | the clip in slot 9 gets a very long name (screenshot: cut with "…" before the ×) |
| `VIDEOPING_SELFTEST_VP=<query>` | needs the internet: the finder's videopong category searches the query; at 3.5 s results and thumbnails are printed and the first clip is downloaded into slot 3 (into the self-test folder); the mouse script is off. Use `VIDEOPING_SELFTEST_STAY=22` (videopong.net can be slow) |
| `VIDEOPING_SELFTEST_MAP=1\|grid` | a mapping preset "selftest stage" (two slices, one corner-pinned, a soft mask) in the self-test folder, mapping on (grid: test grid); at 3 s pixels of the mapped output are checked (outside black, slices lit, mask black). With `VIDEOPING_SELFTEST_TAB=mapping`: the screenshot, and from 2 s a simulated press 12 px beside a corner of slice 2 in the output view, → → →, Shift+↓ (expected: grabbed, moved 3, 10 px) |
| `VIDEOPING_SELFTEST_EDIT=<.fs>` | open it in the shader editor (with `VIDEOPING_SELFTEST_TAB=shader` for the screenshot); at 1.5 s an error is typed in, at 2.5 s the mapped error line is printed (and that the file on disk is unchanged), then fixed. With `VIDEOPING_SELFTEST_AI=<url>` the editor's ai runs against a mock instead (scratchpad mock_shader.py: get_shader, a broken set_shader, a fixed one + look) |
| `VIDEOPING_SELFTEST_DOC=<effect .fs>` | that effect first on the clip (slot 9), its documentation window (.md) open (screenshot of the shader docs) |
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

# release

A release is a git tag; GitHub builds the packages (`.github/workflows/release.yml`):

1. set `version` in `Cargo.toml` (shown in the top bar and by `videoping --version`), commit
2. `git tag v0.0.1 && git push origin main v0.0.1` (the tag must match the version, or the build stops)
3. GitHub Actions (Ubuntu 22.04, so the binary runs on 22.04 and newer): `cargo test`, `cargo deb`,
   `scripts/release-tar.sh`, then a **draft** release with the .deb, the tar.gz and `SHA256SUMS`
4. on GitHub: Releases → the draft → write what's new → Publish

A test build without a release: Actions → release → Run workflow (the files land in the run's
artifacts). Locally: `cargo install cargo-deb`, then `cargo deb && scripts/release-tar.sh`
(files in `target/release-files/`). The package layout (`[package.metadata.deb]` in Cargo.toml):
`/usr/bin/videoping`, the pack in `/usr/share/videoping/isf-pack`, `packaging/videoping.desktop`,
the icons, the docs and licenses; the tar.gz is the same tree under `bin/` and `share/`.

# documentation

| file | for whom |
|---|---|
| `features.md` | the spec: every design decision, thresholds, measurements, open points |
| `website/manual.html` | end users: how to play with videoping; one self-contained HTML page (styles inline), screenshots in `website/img/`; open it in a browser |
| `install.md` | installation only; the source of `website/install.html` (`python3 scripts/install-html.py`) |
| `website/logo/` | the logo (v1 calm, v2 glitch = official): SVG, PNG sizes, editable `videoping-logo.vectorcraft` (VectorCraft) |
| `packaging/videoping.desktop` | launcher entry; icons and install commands in install.md |
| `dev.md` | this file |

Screenshots for the manual come from the self-test (it uses only the repo shaders and the test clip
in the finder, and clears the status line before the screenshot):

```sh
S=/tmp/vp-docs; mkdir -p $S; cp <testclip.mp4> $S/testclip.mp4
VIDEOPING_SELFTEST=$S ./target/release/videoping   # no REC: the record box shows the default folder
U=$S/ui_3000x1818.rgba   # size depends on the screen
c(){ ffmpeg -y -f rawvideo -pix_fmt rgba -s 3000x1818 -i $U -vf "$1" website/img/$2; }
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
website/
  index.html   the sales page (static, opens from the repo; everything it shows is inside website/: img/, logo/, manual.html, install.html)
  install.html generated from install.md: after changing install.md run `python3 scripts/install-html.py` (never edit install.html by hand)
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
  control.rs   control registry: addresses (/layer/2/mix, /slot/5/load, /slot/9/fx/1/param/2 ...)
               list / get / set / snapshot+changes; the base for MIDI, MCP (and OSC); unit tested
  mcp.rs       MCP server: streamable HTTP on 127.0.0.1 (tiny_http thread), JSON-RPC (initialize,
               tools/list, tools/call), tool definitions; tool calls go to the app through a channel
  mcp_tools.rs the MCP tools, answered by the app at the next frame (child module of app); also used by the ai tab
  midi_tab.rs  the tab "midi mapping" (devices, bindings table, addresses + learn); child module of app
  midi.rs      MIDI engine: profiles of bindings (MIDI message <-> control address), modes, LED
               feedback, port scan every 2 s; built-in Bhoreal profile; unit tested
  ai.rs        ai tab client: Anthropic Messages API / OpenAI-compatible chat completions (ureq, blocking,
               worker thread); tool list from mcp.rs, request / answer formats
  ai_tab.rs    the chats (ai tab + shader editor ai): tool loop (calls run at a frame), undo, ask before changes
  videopong.rs videopong.net in the finder: API (search / random), thumbnails, downloads into ~/Videos/videopong
  mapping.rs   projection mapping: slices, masks, presets (config folder), corner-pin homography
  mapping_tab.rs the tab "mapping": preset bar, slice / mask list, input + output views with handles
  shader_editor.rs the tab "shader editor": work copy + live preview, error line mapping, highlighting, the ai tools
  shader_doc.rs the .md next to a shader: summary, parameter descriptions, warnings, view text
  meter.rs     resource meters (GPU timer queries, VRAM bookkeeping, process CPU / RAM)
  record.rs    REC: NV12 frames -> hardware H.264 -> crash-safe mp4
```

The app side of the registry is `App::control_list / control_get / control_set / control_changes`
(tap and record need the app, so `control::set` returns them as an `Effect`).

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
  (`inputImage` = effect input, other images: black for now), audio (black).
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

not yet: midi (bhoreal, twister), videopong / online shaders, sets,
audio, api / mcp, other tabs.
