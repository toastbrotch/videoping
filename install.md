<img src="website/logo/videoping-v2-logo.png" alt="videoping" width="480">

install videoping
=================

videoping is a personal live vjing tool for Linux. There are no binary packages yet:
you build it from source once (see the build step below; details in `dev.md`).

Tested on Pop!_OS 24.04 (Ubuntu 24.04 base), Intel Iris Xe, Wayland.

This guide is also a page of the website: `website/install.html` (generated from this file).

# 1. runtime requirements

- a GPU driver with **OpenGL 3.3** or newer (Mesa / NVIDIA / AMD are all fine)
- **GStreamer 1.20+** with the usual plugins, for mp4 playback
- **DejaVu Sans** font (ui symbols like ▶ ◀ ↑ ↓; without it some icons show as boxes)
- optional: the **NDI 6 runtime** for NDI output
- MIDI controllers: ALSA (`libasound2`, installed on every desktop); class-compliant USB MIDI
  devices need no driver

Ubuntu / Debian / Pop!_OS:

```sh
sudo apt install \
  gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
  gstreamer1.0-libav gstreamer1.0-vaapi \
  fonts-dejavu-core
```

`gstreamer1.0-vaapi` (or the `va` plugins in plugins-bad) gives hardware H.264 decoding on Intel/AMD.
Recording (REC) needs a hardware H.264 encoder: NVENC on NVIDIA (`nvcodec` in plugins-bad, with the
NVIDIA driver) or VA-API on Intel / AMD (`va` in plugins-bad or `gstreamer1.0-vaapi`). Without one, REC
is disabled.

# 2. NDI runtime (optional)

videoping does not ship NDI. It loads `libndi.so.6` at runtime, searched in this order:

1. the **NDI library folder** set in the ui (output options, saved with the session)
2. `$NDI_RUNTIME_DIR_V6`
3. `/usr/local/lib`, `/usr/lib`, `/usr/lib/x86_64-linux-gnu`, `/opt/ndi/lib`, `~/.local/lib`
4. the system library path

## install the regular NDI SDK (free)

1. Download the "NDI SDK for Linux" (free, the license is shown during install):
   - website: https://ndi.video/for-developers/ndi-sdk/ (asks for name / email), or
   - direct link:

     ```sh
     cd ~/Downloads
     wget https://downloads.ndi.tv/SDK/NDI_SDK_Linux/Install_NDI_SDK_v6_Linux.tar.gz
     ```

2. Unpack it and run the installer script. It shows the NDI license; read it and accept with
   `y`. The script creates the folder `NDI SDK for Linux` next to itself:

   ```sh
   tar -xzf Install_NDI_SDK_v6_Linux.tar.gz
   sh Install_NDI_SDK_v6_Linux.sh
   ls "NDI SDK for Linux/lib/x86_64-linux-gnu/"     # libndi.so  libndi.so.6  libndi.so.6.x.x
   ```

   (ARM boards: use `lib/aarch64-rpi4-linux-gnueabi` or the matching folder instead.)

3. Make the library findable. Pick **one**:

   - just for your user (only videoping looks there, nothing else is affected):

     ```sh
     mkdir -p ~/.local/lib
     cp -P ~/Downloads/"NDI SDK for Linux/lib/x86_64-linux-gnu/"libndi.so* ~/.local/lib/
     ```

   - system-wide:

     ```sh
     sudo cp -P ~/Downloads/"NDI SDK for Linux/lib/x86_64-linux-gnu/"libndi.so* /usr/local/lib/
     sudo ldconfig
     ```

   - or leave it where it is and enter the folder
     `~/Downloads/NDI SDK for Linux/lib/x86_64-linux-gnu` as **NDI library folder** in the
     output options.

   If an older copy (e.g. the Varda development build) is already in that place, the new files
   replace it.

4. Make sure discovery works: `avahi-daemon` must run (`systemctl status avahi-daemon`) and
   the firewall must allow mDNS (5353/udp) and the NDI ports (5960+ tcp/udp).

5. Check that you have the regular build and not the 30-minute development build. This should
   print `0`:

   ```sh
   strings ~/.local/lib/libndi.so.6 | grep -c "30 minutes"
   ```

6. Start videoping, open **output options**, turn on **NDI output**. It shows "NDI sending" in
   green; hover it to see which libndi was loaded. Check it on another machine with
   NDI Studio Monitor (Windows / macOS, part of the free NDI Tools) or any NDI receiver.

7. NDI input: sources on the network show up in the finder under **NDI sources** (also
   videoping's own output). Double-click one to put it into the prepare grid.

If NDI can't be loaded, the output options show the error in red. When it works they show
"NDI sending" (hover shows which libndi was loaded).

Note: some apps bundle the *NDI Advanced SDK development build* (e.g. Varda), which stops
sending after 30 minutes. It works for testing (copy its `libndi.so*` the same way), but use the
regular NDI runtime for shows. libndi is never committed to this repo (proprietary license).

# 3. build and install

Needs the build tools from `dev.md` (Rust + GStreamer dev packages).

```sh
cd videoping
cargo build --release
install -Dm755 target/release/videoping ~/.local/bin/videoping
```

Launcher entry and icon (so videoping shows up in the app menu / dock with its logo; on Wayland the
window icon also comes from here):

```sh
install -Dm644 packaging/videoping.desktop ~/.local/share/applications/videoping.desktop
for n in 512 256 128 64 48; do
  install -Dm644 website/logo/videoping-v2-icon-$n.png ~/.local/share/icons/hicolor/${n}x${n}/apps/videoping.png
done
# 32 and 16 px: the calm v1 logo (the glitch version turns into mud that small)
install -Dm644 website/logo/videoping-icon-32.png ~/.local/share/icons/hicolor/32x32/apps/videoping.png
install -Dm644 website/logo/videoping-icon-16.png ~/.local/share/icons/hicolor/16x16/apps/videoping.png
install -Dm644 website/logo/videoping-v2-icon.svg ~/.local/share/icons/hicolor/scalable/apps/videoping.svg
gtk-update-icon-cache ~/.local/share/icons/hicolor 2>/dev/null || true
```

The desktop entry starts `videoping` from `~/.local/bin`. If that folder is not in your `PATH`, put
the full path into `Exec=` in the installed file.

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

The easiest way to render on the NVIDIA card:

- output options → **render on NVIDIA GPU (applies on restart)**, then restart videoping, or
- one-off: `videoping --nvidia`

videoping then restarts itself once with PRIME render offload; the output options show
`gpu: NVIDIA GeForce ...`.

**Measured on this laptop (Pop!_OS, Wayland, 120 Hz 3K panel), output set to 60 fps:**

| | native Wayland (default) | X11 / XWayland |
|---|---|---|
| Intel | ~53 fps, sharp | 60 fps, ui upscaled (blurry) |
| NVIDIA (offload) | **30 fps**, sharp | 60 fps, ui upscaled (blurry) |

So on the laptop screen, NVIDIA under Wayland is *slower* than Intel: every frame is copied to
the Intel GPU that drives the panel. To try X11: `env -u WAYLAND_DISPLAY videoping` (or with
`--nvidia`). For gigs with a projector on the NVIDIA-wired port, `system76-power graphics nvidia`
(see below) avoids the copy completely.

Manually, the same as `--nvidia` (both variable sets are needed: EGL on Wayland, GLX on X11):

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

# 5. using videoping

See the [manual](website/manual.html) (open it in a browser): concept, a tour of the screen, playing a set, effects, output and
recording, performance, files and settings, troubleshooting.
