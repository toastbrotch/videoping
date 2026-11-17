install videoping
=================

videoping is a personal live vjing tool for Linux. There are no binary packages yet:
you build it from source once (see the build step below; details in `dev.md`).

Tested on Pop!_OS 24.04 (Ubuntu 24.04 base), Intel Iris Xe, Wayland.

# 1. runtime requirements

- a GPU driver with **OpenGL 3.3** or newer (Mesa / NVIDIA / AMD are all fine)
- **GStreamer 1.20+** with the usual plugins, for mp4 playback
- **DejaVu Sans** font (ui symbols like ▶ ◀ ↑ ↓; without it some icons show as boxes)
- optional: the **NDI 6 runtime** for NDI output

Ubuntu / Debian / Pop!_OS:

```sh
sudo apt install \
  gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
  gstreamer1.0-libav gstreamer1.0-vaapi \
  fonts-dejavu-core
```

`gstreamer1.0-vaapi` (or the `va` plugins in plugins-bad) gives hardware H.264 decoding on Intel/AMD.

# 2. NDI runtime (optional)

videoping does not ship NDI. It loads `libndi.so.6` at runtime, searched in this order:

1. `$NDI_RUNTIME_DIR_V6/libndi.so.6`
2. the system library path (`libndi.so.6`, e.g. `/usr/local/lib`)

Get the "NDI SDK for Linux" from https://ndi.video/for-developers/ndi-sdk/ and either copy
`lib/x86_64-linux-gnu/libndi.so.6*` to `/usr/local/lib` (then `sudo ldconfig`) or point the
variable at its folder:

```sh
export NDI_RUNTIME_DIR_V6=/path/to/NDI\ SDK\ for\ Linux/lib/x86_64-linux-gnu
```

Note: some apps bundle the *NDI Advanced SDK development build*, which stops sending after
30 minutes. Use the regular NDI runtime for shows.

NDI discovery uses mDNS: `avahi-daemon` must run and the firewall must allow mDNS (5353/udp)
and the NDI ports (5960+ tcp/udp).

# 3. build and install

Needs the build tools from `dev.md` (Rust + GStreamer dev packages).

```sh
cd videoping
cargo build --release
install -Dm755 target/release/videoping ~/.local/bin/videoping
```

Run it from the folder that holds your clips/shaders, or add folders in the finder
(column 1 → "folders").

```sh
videoping
```

# 4. run on the discrete NVIDIA card (hybrid laptops)

On a laptop with Intel/AMD + NVIDIA in **hybrid** mode, apps render on the integrated GPU by
default. videoping logs the GPU it uses at startup (stderr and the top bar status):

```
gpu: Mesa Intel(R) Iris(R) Xe Graphics (RPL-P) (Intel)
```

To render on the NVIDIA card, start it with PRIME render offload. Both variable sets are needed:
eframe uses EGL on Wayland and GLX on X11.

```sh
__NV_PRIME_RENDER_OFFLOAD=1 \
__GLX_VENDOR_LIBRARY_NAME=nvidia \
__EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/10_nvidia.json \
videoping
```

You should then see `gpu: NVIDIA GeForce RTX 4050 Laptop GPU/PCIe/SSE2 (NVIDIA Corporation)`.
(Tested on Pop!_OS 24.04, Wayland, driver 595.)

Shortcut: a small wrapper `~/.local/bin/videoping-nvidia`:

```sh
#!/bin/sh
export __NV_PRIME_RENDER_OFFLOAD=1
export __GLX_VENDOR_LIBRARY_NAME=nvidia
export __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/10_nvidia.json
exec videoping "$@"
```

(`chmod +x ~/.local/bin/videoping-nvidia`)

Alternatives:

- `switcherooctl launch videoping` (sets the same variables for you)
- a desktop entry with `PrefersNonDefaultGPU=true`: launchers (Pop!_OS / GNOME / COSMIC) then
  start it on the discrete GPU, or offer "Launch using Discrete Graphics" on right-click:

  ```ini
  # ~/.local/share/applications/videoping.desktop
  [Desktop Entry]
  Type=Application
  Name=videoping
  Exec=videoping
  Terminal=false
  PrefersNonDefaultGPU=true
  ```

- Pop!_OS: `system76-power graphics nvidia` (needs a reboot) runs the *whole* desktop on NVIDIA.
  This is useful for gigs where the projector hangs on an HDMI port wired to the NVIDIA card.
  `system76-power graphics` shows the current mode.

Notes:

- video decoding stays on GStreamer's default decoder (VA-API on the Intel GPU); frames go
  through system memory anyway, so this is fine.
- in hybrid mode every frame shown on a screen driven by the Intel GPU gets copied from the
  NVIDIA card. For fullscreen output on an external screen it is best when that screen is
  connected to the GPU that renders.

# 5. files videoping writes

| what | where |
|---|---|
| autosave of the whole session (every 2 s when something changed, and on quit) | `~/.local/share/videoping/autosave.json` |
| ui layout (column widths) | eframe storage in `~/.local/share/videoping/` |
| explicit save / load | the path in the top bar (default `~/videoping-session.json`) |

Media (mp4, isf) is referenced by path, never copied.

# 6. quick usage

- **finder** (column 1): click = preview, double-click = add to the first free prepare slot
  (effects: add to the selected instance), drag onto a slot / effect stack.
- **prepare** (column 2): 4x8 grid. Click = select (detail on the right), double-click =
  load into the selected mixer-layer, drag = move, × = delete (also removes it from the mixer).
- **mixer** (column 3): click a layer = select it + focus its instance. Drop a slot on a layer to load it.
  BPM: TAP, slider, ½x / 2x.
- **output** (column 4): BLACKOUT, FREEZE, brightness, output effects, output options
  (resolution, output window, NDI).
- **output window**: double-click or `F` = fullscreen, `Esc` = leave fullscreen. To choose the
  screen, move the window there first.
