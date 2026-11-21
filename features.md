videoping
---------

videoping is a live vjing application, 5 columns, all column-borders are moveable to arrange, fullscreen, reduce ui-clutter, slick, darkmode, cleaned-up, most important buttons are directly available. foldout/tabs only when absolutely neede. maybe resolume 2.41 kind or vdmx , mcp and api, borrow from artcraft filmcraft, it is used by mouse but should be played with midicontrollers ultimately). keep in mind this is my personal vjing tool. for the midi mapping i have a 8x8 led-pushbutton controller (bhoreal https://github.com/toastbrotch/docs.bhoreal.com-archive/blob/master/docs/start.md ) and a 4x4 led enhanced rotary encoder controller (midifighter twister)

# ui style

- accent color (selected buttons, R G B, limiters, range sliders, effect checkbox): muted plum #8B4A87, white text on it
  - chosen from the hue gap furthest away from the layer colors (#1B6898 blue, #388E3C green, #C8640A orange), same calm saturation
- all buttons have the same size as the R G B buttons (min 30x18, text 12), corners only slightly rounded (2 px)
- on/off of effects and of mixer-layers is a big checkbox (18x18): effects in the accent color, layers in their layer color
- transport rows: a visible gap (10 px) between the groups: play directions | loop / once / bounce | bpm + speed + ½x 2x
- mix-type + "Ch R G B" stay on one line when there is room, otherwise "Ch R G B" wraps to its own line (right aligned)

# logo
- mark: a plum play triangle (▶, the video / live source) sending out three "ping" waves in the three layer colors (blue, green, orange from inside out) = a live signal going out, mixed from 3 layers
- connected to videopong.net (vj footage sharing) only by name: no ping-pong ball or paddles, the logo is about the live part
- wordmark: "videoping" in Inter Semi Bold, lowercase, "video" light grey #e6e6e6, "ping" plum #8B4A87
- icon: the mark on a dark rounded square (#141414, like the app); still readable at 32 px
- files in docs/logo/: videoping-icon.svg / -512 / -128 / -32.png, videoping-logo.svg / .png (lockup, text as outlines), videoping-logo.vectorcraft (editable source, made with VectorCraft)
- v2 "glitch" (same mark, broken like a damaged video signal): files videoping-v2-icon.svg / -512 / -128 / -32.png, videoping-v2-logo.svg / .png
  - displacement: 4 horizontal bands of the mark cut out and shifted sideways (tearing / datamosh)
  - channel split: ghost copies of the play triangle (and of "ping" in the lockup) offset left in blue and right in orange, like the R G B switches drifting apart
  - scanlines over the icon (20 %)
  - no displacement band through the wordmark: at the top / bottom of the letters the shifted curves read as strike lines, the word stays readable
  - large sizes only (splash, manual, posters): at 32 px the glitch turns into mud, so window / taskbar icon stays v1
- all four artboards (icon, lockup, icon v2, lockup v2) are in docs/logo/videoping-logo.vectorcraft
- v2 is the official logo; where it is used:
  - window icon of videoping and of the output window (built into the binary: docs/logo/videoping-v2-icon-256.png)
  - launcher / dock icon: packaging/videoping.desktop (app id "videoping") + icon theme in ~/.local/share/icons/hicolor: v2 from 48 px up and scalable, v1 for 16 and 32 px (install.md); on Wayland the window icon comes from there
  - manual.html: v2 lockup as page header, v2 icon in the table of contents, favicon (v1 32 px for the browser tab, v2 128 px)
  - install.md: v2 lockup on top
  - page background of the manual = #141414, the same as the app and the logo background, so the lockup has no visible box

# tabs

the ui has top-level tabs:

- **live**: everything described below — the default tab
- later (not planned yet):
  - mapping
  - shader editor
  - midi mapping
  - sequencer

everything below belongs to the tab "live".

# layout of the tab "live"

5 columns, all borders draggable:

| column | content |
|---|---|
| 1 finder | finder preview, finder |
| 2 prepare grid | 4x8 slot grid, below it: master (blackout, freeze, brightness), global bpm tapper, record, patch-bay, output options, general config |
| 3 prepare detail | prepare preview pinned on top, below it (scrolls): clip interface, effect stack; default width ~30% of the window |
| 4 mixer + output | main output preview pinned on top, below it (scrolls): mixer (3 layers), output effects; takes the remaining width |

- the prepare preview (column 3) and the main output preview (column 4) always stay on top, also when the rest of the column is scrolled
- both previews always have the same size (the width of the narrower of the two columns)

# save / load

- everything is saved:
  - prepare grid (all pages, instances, their clip settings, effects and all parameters incl. range settings)
  - mixer layers (loaded instances, mix-type, mix %, r/g/b, active, order, selected layer)
  - patch-bay routings
  - bpm, output effects, master (brightness), output options
  - midi mappings
  - ui layout (column widths, current tab)
- autosave: the current state is saved continuously, on restart (or after a crash) everything is back
- save / load buttons: save the whole state to a file / load it from a file
  - placement (proposal): small buttons in the top bar next to the tabs, so they are available from every tab
- new button (next to save / load): clears everything after a confirmation dialog ("clear everything" / "cancel", Esc = cancel)
  - cleared: prepare grid, mixer, effects, output effects, bpm, master
  - kept: finder folders and output options (resolution, frame rate, screen, NDI, gpu) — that is setup, not content
- fps counter in the top bar (next to the session file, visible from every tab): "measured / target fps" of the output
  - measured = video frames rendered in the last second; target = output options frame rate
  - grey when on target; bright white with ⚠ when below 95 % of the target or when a frame was dropped in the last 2 s (no color: colors are reserved for the layers)
  - hover: number of dropped frames since start (a frame counts as dropped when it is rendered a whole interval too late)
- media (mp4, isf shaders) are referenced by path, not copied into the file

# column 1

## finder preview

- shows preview if something is clicked in the finder

## finder

find the things you want to use. clips, generators are dragged to the prepare-lane (or doubleclick: just add it at the top), effects are dragged to a clip in the prepare area. it scrolls automatically up/down either in clips/generators and also inside such on the already assigned effects. 
finder finds these:
- local mp4
- local isf generator shader
- local isf effect shader
- NDI Sources
  - discovered automatically on the network (list refreshes by itself, also local sources)
  - click = preview, double-click / drag = new instance in the prepare grid (like a clip)
  - an NDI instance has no clip interface; the detail shows its status: resolution @ fps, or "no signal" (normal grey text, no color: colors are reserved for the layer color-code)
  - effects work on it like on any clip; it is received only while it is live / selected (stops 10 s later)
- the finder folders are watched: every 2 s the known folders are checked (modification time only, cheap); a new, deleted or renamed file (e.g. a finished recording, copied clips) appears without a manual refresh
- videopong clips (prio2)
- glslop.com shaders or https://editor.isf.video/shaders? (prio2)
- set (each from above can be added to a set) (prio2)

explicit not implemented:
- local ffgl generatoren compiled as so
- local ffgl effect compiled as so

# column 2 + 3 (prepare)

## prepare

here are the clips/generators in a 4x8 grid prepared and get enhanced with effects. double-click loads it into the selected mixer-layer

- a clip/generator loaded into a mixer-layer is the *same instance* as in the prepare lane (no copy): it is live
  - effects of a live clip are tweaked right here in the prepare lane and change the output immediately
  - live clips are marked (e.g. colored border + which mixer-layer it is playing in)
  - the timeline marker moves while it plays
- loading a clip that is already live into a second mixer-layer: for now nothing happens (open decision)
- duplicate: any clip/generator in the prepare lane can be duplicated (incl. its effects and settings) into a new, independent prepare entry

### layout

the prepare area uses two columns (see "layout of the tab live"):

- column 2: 4x8 slot grid (4 wide, 8 high) = mirrors bhoreal columns 2-5 1:1
  - each slot: thumbnail + name, empty slots are drop targets for the finder
  - type badge top left on the thumbnail (neutral grey on dark, colors belong to the layers):
    ▶ clip (mp4), ∿ generator (isf), NDI (NDI source); the live marker "L1".."L3" sits top right
  - live slot: border in the color of the mixer-layer it plays in (same as bhoreal LED)
  - borders: live = layer color, selected (not live) = white, otherwise the same thin grey border as an empty slot
  - thumbnails are static, only live or selected slots play (performance)
  - click = select (shows detail in column 3), double-click / bhoreal pad = load into selected mixer-layer
  - click on an empty slot = deselect: the prepare detail (column 3) becomes empty
    - the column layout does not change: column 3 keeps its width and shows an empty (black) preview of the same size
  - more than 32 instances: pages of 32 slots, the bhoreal follows the page
  - move an instance to another slot: drag & drop
  - delete an instance: small x on the slot
    - not possible while it is live in the mixer: the x is not shown (unload it from the layer first)
  - drag a new clip / generator / NDI source from the finder onto a slot:
    - empty slot: new instance
    - occupied slot, not live: the old instance is replaced by the new one
    - live slot: not possible (no drop highlight, nothing happens)
  - dropping an effect onto an occupied slot adds it to that instance (also when live)
- column 3: detail of the selected instance
  - prepare preview (on top)
  - generic clip interface (below)
  - effect stack (below)

### detail of the selected instance

- generic clip interface is visible, also if this clip is in the mixer
  - only for clips with a timeline (mp4), not visible on isf generators and NDI sources
  - timeline with limiters
    - neutral grey (no green: colors are reserved for the layer color-code), white position marker
    - limiters are wide bracket handles "[" and "]" (easy to catch, cursor changes on hover)
  - playmode if possible: forward (default), backward, jumpcut, pause 
    - jumpcut: jump to a random position (between the limiters) every N beats, synced to the global bpm
      - N is set with the same beat selector as the bpm speed mode (1, 2, 4, 8, 16), ½x/2x work on it too
  - loop, once, bounce
    - once stops at the end; switching to loop then restarts from the start, switching to bounce turns around
  - speed, two modes (toggle):
    - free: speed slider
    - bpm: set how many beats the part between the limiters lasts (e.g. 1, 2, 4, 8, 16), speed adapts to the global bpm
      (example: 120 bpm, 8 beats -> the limited part plays in 4 s, regardless of the clip length)
    - ½x and 2x buttons: halve / double the speed instantly (in bpm mode: double / halve the number of beats)

- effects (layering top-down, max 8 per clip)
  - reorder: drag & drop
  - drag handle "≡": button-sized grab area (26x18) at the start of the effect row, hand cursor
  - the effect stack (clip effects and output effects) reaches down to the bottom of its column: the whole empty space below is a drop area
  - no horizontal lines in the prepare detail (below the name / duplicate row, between the clip interface and the clip effects)
    nor between the mixer layers and the output effects
  - move to the output effects and back: drag the "≡" handle onto the other effect stack (onto a row = insert there,
    onto the empty area = append); the effect keeps all its settings (parameters, ranges, animation, mix, r/g/b);
    a clip that already has 8 effects refuses it
  - remove: small x on the effect row
  - active/inactive
  - mix-type, R G B and mix slider are always visible, also when the effect is folded in
  - all parameters (only on click folded out)
    - parameter names: left aligned, normal text size, long names wrap onto a second line (full name on hover)
      - the name column spans the slider row and the play-button row, so a wrapped name does not push the play buttons down
    - each parameter has a range slider
      - same look and handles as the clip timeline: "[" "]" bracket handles for the range, white value marker, neutral grey range
    - mix-type (alpha, lumakey, inverslumakey, add)
    - mix percentage
    - r, g, b channel
    - rangeslider can be pause (default), move forward, move backwards, jumpcut
      - jumpcut: same rule as the clip: jump to a random value inside the range every N beats (N from the beat selector, ½x/2x work on it)
    - loop, once, bounce
    - speed, same two modes as the clip: free (speed slider) or bpm (how many beats one cycle through the range lasts)
    - ½x and 2x buttons, same as the clip

## prepare preview

- top of column 3, always visible (does not scroll away): shows the selected instance incl. its effects

# column 4 (mixer + output, merged)

## mixer
- 3 layers (layering top-down)
  - each has mix-type (alpha, lumakey, inverslumakey, add)
    - only these 4 mix-types everywhere (no lighten, darken, difference etc.)
    - definitions (top = the layer / effect result, l = its luma, BT.601):
      - alpha: crossfade, weight = top alpha × mix
      - lumakey: dark parts transparent, weight = smoothstep(0.05, 0.35, l) × alpha × mix (black gone, from l = 0.35 fully visible)
      - inverse lumakey: the same key on the inverted luma, bright parts transparent, weight = smoothstep(0.05, 0.35, 1 − l) × alpha × mix (white gone, up to l = 0.65 fully visible)
        - not the inverted matte of the lumakey (1 − key): that also removed all mid tones (fixed 2026-11-21)
      - add: bottom + top × mix
    - the key thresholds are fixed for now (no threshold / softness control)
  - mix percentage
    - shown as 0-100 % everywhere (mixer layers, effects, output effects; brightness too)
  - r, g, b channel
    - r/g/b toggles have no red/green/blue color: the layer colors are reserved for the layer color-code
  - mix-type and r/g/b are same-size buttons (like clip-mixer.gif), accent color when on, on one line:
    - left: A (alpha), Lk (lumakey), iLk (inv lumakey), Ad (add) — no dropdown
    - right: "Ch" R G B, with a margin to the right edge so it stays away from the × of effects / layers
    - below: mix slider (0-100 %)
  - active/inactive
  - see clip-mixer.gif 
  - move up/down a layer
  - basics of the generic clip interface (only 3 layers -> enough vertical room), like clip-mixer.gif (not shown for isf generators and NDI sources):
    - timeline with moving position marker and limiters
    - playmode, loop/once/bounce
    - speed (free/bpm, ½x/2x)
    - same instance as in the prepare lane, so changes here and there are the same
    - effects are not shown here (only in the prepare detail)
  - click on a layer = select it + focus its instance in the prepare lane (shows its detail, switches the prepare page if needed)
    - also when clicking one of the layer's controls (the controls still do their own action)
- mark the selected layer

## record
- own box in column 2, below bpm (above the patch-bay), directly visible
- REC button: records the output (after master, the same picture as NDI) to a file
  - while recording: "● REC" (red) plus running time, file size and resource meter; hover: path, frames, dropped frames, encoder
  - the target folder is not shown in the box (only in the REC hover); it is set in the general config (recording folder)
  - files are named `videoping-YYYY-MM-DD_HH-MM-SS.mp4`
  - keeps recording through new / load; ends when the output resolution changes; finished properly on quit
  - disabled (with the reason on hover) when no hardware encoder works

## patch-bay for videosignals

- placed in column 2, below the prepare grid (under master and bpm)

goal: route video signals between any points, mainly backwards for feedback loops.

- sources (tap points):
  - each prepare entry (post effects)
  - each mixer-layer (post)
  - each output effect (post)
  - global output
- destinations: anything that takes a video input
  - extra image inputs of isf effects (e.g. displace, mask, feedback)
  - a "patch" source as clip/generator in the prepare lane (like an NDI source)
- a signal routed backwards (to an earlier point in the chain) is the previous frame (1 frame delay) -> this makes feedback possible without endless loops

visuals:
- a small dropdown "source" at every video input (effect image input, patch clip) -> no extra clutter, routing is visible where it is used
- no matrix overview (not needed)
- no node-graph with cables (too much clutter)
- patch indicator (small icon + source name, e.g. "⇄ L2") on the folded effect row / prepare slot that gets its input from the patch-bay, so patches are visible without folding out
- no extra panic button: panic = global blackout + global freeze (see mixer / bhoreal)
- not mapped to midi

## global bpm tabber
- placed in column 2, below the prepare grid (under master)
- all on one line: TAP, 4 beat dots, bpm slider (fills the rest; "bpm" on hover), ½x, 2x
- bpm is always an integer (tap, slider and ½x/2x round to whole bpm)
- tap button sets the bpm
- bpm slider: change the bpm globally by hand (e.g. slow everything down), tap and slider set the same value
- ½x and 2x buttons: halve / double the bpm instantly
- drives jumpcut (clips and parameter ranges jump every N beats)
- drives the speed of clips in bpm mode (beats per limited part)
- drives the speed of animated parameter range sliders in bpm mode (beats per cycle)

## master (blackout, freeze, brightness)
- one group in column 2, below the prepare grid, directly visible, no foldout
- all on one line: BLACKOUT, FREEZE, brightness slider (fills the rest of the line; no text label, "brightness" on hover)
- global blackout button
- global freeze button
- global brightness slider (0-100%) next to them
  - during blackout the slider shows 0 %; moving it up from 0 ends the blackout and sets the brightness -> fade in from black
  - ending the blackout with the button restores the brightness from before
- all three act at the very end (after the output effects), for NDI and screen output

## output effects (layering top-down)
- multiple effects, they work exactly like the effects in the prepare lane:
  - active/inactive, reorder by drag & drop, remove with x
  - can be dragged to the selected clip's effect stack and back (keeps all settings)
  - all parameters with range sliders (pause/forward/backward/jumpcut, loop/once/bounce, free/bpm speed, ½x/2x)
  - mix-type, mix percentage
  - r, g, b channel


## output options (fold open)
  - placed in column 2, below the patch-bay
  - resolution: fullHD (default)
  - frame rate: 25 / 30 / 50 / 60 (default 60) for screen and NDI output
    - video (instances, mixer, output, NDI) is rendered at this rate, the ui repaints at the display rate
    - NDI announces this frame rate
  - screen: buttons 1, 2, 3 ...; the fullscreen output window goes to this screen (also on Wayland)
    - "connected: eDP-1, HDMI-A-1" shows the connected outputs as a hint (their order can differ from the numbers)
  - output window on/off, fullscreen on the chosen screen (F / double-click in the output window)
  - ndi name
  - NDI library folder (folder with libndi.so.6, saved; empty = default places)

## general config (fold open)
- placed in column 2, below the output options
- settings of the program itself, not of a session: own file `~/.config/videoping/config.json`; save / load / new do not change it
- resource meters on/off (default on): to compare the overhead or switch them off for a show
- recording folder (empty = `~/Videos/videoping`); "add to finder" adds it to the finder folders, so recordings can be dropped onto a slot

## output preview
- main output preview at the top of column 4, always visible (does not scroll away)

# midi mapping

## bhoreal (8x8 led-pushbuttons)

8 vertical columns, 8 pads each (LED shows state):

| column | 8 pads (top-down) |
|---|---|
| 1 | select mixer-layer 1-3, activate/deactivate mixer-layer 1-3, global blackout, global freeze |
| 2-5 | select instance in the prepare lane = load it into the selected mixer-layer (4x8 = 32 instances) |
| 6-8 | activate/deactivate effects of the clip in mixer-layer 1 / 2 / 3 (max 8 effects per clip) |

- LED colors per mixer-layer (same colors as the live-marking in the ui), layer 1 = #1B6898 (blue), layer 2 = #388E3C (green), layer 3 = #C8640A (orange)
  - column 1: select/activate pads in the color of their layer
  - columns 2-5: instance pad lights up in the color of the layer it is live in, dim/off when not live
  - columns 6-8: effect pads in the color of their layer, bright = active, dim = inactive, off = no effect
- global blackout and global freeze: same as the master buttons in column 2
- more than 32 instances: paging, the bhoreal shows the current page of the prepare grid

## midifighter twister (4x4 rotary encoders)

- tbd

# technical

## rendering / isf
- existing isf files are used unchanged (no editing of shaders)
- generator or effect is decided automatically from the isf header (isf standard): an image input named `inputImage` = effect, otherwise generator; the folder a file sits in does not matter
- rendering with OpenGL (rust crate glow), not wgpu: isf shaders in GLSL 1.20 / ES 1.00 style (gl_FragColor, IMG_NORM_PIXEL, TIME, RENDERSIZE) run as they are
- the host provides the standard isf uniforms (TIME, RENDERSIZE, inputImage, ...) plus the non-standard `uInstanceSeed`:
  - one fixed random value per instance (like the ISFBrowser host does), so two instances of the same shader look different
  - a duplicated instance gets a new uInstanceSeed
  - uInstanceSeed is saved, so an instance looks the same after save/load
- parameter LABEL from the isf header is shown in the ui (falls back to NAME)
- bool inputs are shown as the big checkbox (accent color), not as a 0..1 slider; same layout as the sliders: name in the left label column, checkbox where the slider starts
  - bool inputs are declared as `uniform bool`; if a shader then does not compile (it compares them like floats, e.g. `if (uInvert > 0.5)`), it is compiled again with them declared as `float` (0.0 / 1.0), so such shaders still run unchanged
- color inputs and dropdowns (long inputs with VALUES/LABELS): same layout, name in the left label column, control where the slider starts
- right click on a parameter (its name or its control) resets it to the default of the isf header, as freshly loaded: value, range limits back to MIN..MAX, animation off (pause); works for effect and generator parameters


## resource meters
- on/off in the general config (default on); off hides the meters, the GPU timing keeps running for the render priority
- shown only while an item is rendered (live, selected, or as preview), nothing for idle slots
- indicator: 3 small bars like a signal meter (1 = light, 2 = medium, 3 = heavy), grey only (colors are reserved for the layers)
  - prepare slots: bottom left of the thumbnail = the whole instance (source + its active effects)
  - effects (clip effects and output effects): next to the effect name = that effect alone
  - hover shows the numbers (GPU ms and % of the frame budget, VRAM, decoder for clips)
- REC (while recording): meter next to time and size: GPU time of the NV12 conversion + readback, CPU time of the copy to the encoder, and the encoder queue (nearly full = heavy: the encoder does not keep up, frames get dropped)
- top bar, next to the fps counter: "GPU x ms · VRAM x MB · CPU x %" (hover: RAM, cores, on NVIDIA the GPU memory of all programs)
- level = the highest of:
  - GPU time relative to the frame budget (1000 / fps ms): < 15 % light, < 40 % medium, else heavy
  - own VRAM of the item: < 64 MB light, < 256 MB medium, else heavy
  - clips: decoding. hardware decoder = light; software decoder by pixel rate (< 95 Mpx/s light, < 380 Mpx/s medium, else heavy, 1080p30 ≈ 62 Mpx/s); heavy when the decoder does not keep up (delivers < 85 % of the frames it should)
- how it is measured (must not add noticeable load itself):
  - GPU: OpenGL `TIME_ELAPSED` queries around each source and each effect, read back a few frames later without waiting; only every 8th video frame (each query costs CPU in the driver: measuring every frame cost ~1.2 ms per frame on Intel/Mesa, every 8th frame is not measurable any more)
  - VRAM: bookkeeping of the frame buffers videoping allocates (no GPU call); total GPU memory only on NVIDIA (GL_NVX_gpu_memory_info)
  - CPU / RAM: `/proc/self` once per second, for the whole program (CPU per clip cannot be separated reliably, so clips show their decoding instead)


## render priority (main mix first)
- every video frame: 1. live instances (in the layers), 2. main mix (mixer, output effects, master, NDI, REC), 3. only then prepare work
- prepare work = the selected instance when it is not live, the finder preview, thumbnails of slots that never rendered
- prepare work runs at a reduced rate, chosen from the main mix's GPU time per frame (resource meter timing):
  - < 60 % of the frame budget: half the output rate (60 -> 30 fps)
  - < 80 %: a quarter, < 95 %: an eighth, above: paused until the main mix recovers
  - a dropped frame in the last 2 s counts as "tight" (at least a quarter) only when videoping's own GPU load is above 50 % of the budget (on Wayland the compositor alone can drop frames, pausing the preview would not help)
  - slower at once, faster again only after 2 s of calm
- thumbnails of newly loaded slots: one per prepare frame, only without pressure (no burst after loading a session)
- every instance has its own clock: animations, generator TIME and jumpcut beats run in real time also at the lower rate, nothing jumps when an instance goes live
- unchanged on purpose: resolution (a preview looks exactly like it will live, feedback buffers continue when it goes live) and clip decoding (prepare clips keep playing)
- the prepare detail header shows the rate when the instance is not live: "prepare 30 fps" or "paused: main mix has priority" (grey; hover: GPU time of the main mix)
- the GPU timing always runs (also with the resource meters hidden), its cost is not measurable
- measured (Intel Iris Xe, live clip with rutt-etra, fractal-terrain selected in prepare, 8 ms GPU per render): main output 37 fps with priority = the same as without the prepare generator; before (prepare every frame): 30 fps


## recording
- goal: a recording plays again in videoping as a clip, with all transport modes (backward, bounce, jumpcut, scrubbing)
- format: H.264 in .mp4, output resolution and frame rate, no audio
  - keyframe every fps / 4 frames (8..15), no B-frames: backward play, jumpcut and scrubbing stay smooth
  - bitrate 40 Mbit/s at 1080p60, scaled by pixel rate (min 8 Mbit/s); about 300 MB per minute at 1080p60
  - BT.709, frames stamped by frame number (constant frame rate, stays in real time when frames are dropped)
- hardware encoder only (low CPU): NVIDIA NVENC (nvcudah264enc), else Intel / AMD VA-API (vah264lpenc, vaapih264enc); tested once at program start in the background; no software fallback (x264 would cost more than a CPU core)
- RGBA -> NV12 conversion in a shader on the GPU, async readback through two PBOs (like NDI), the render thread never waits
- crash-safe: index space for 1 hour reserved at the start of the file, updated every 2 s; after a crash the file plays up to the last update (tested: killed after 7 s, plays 6 s); longer recordings are finished normally on stop
- measured (1080p, Intel Iris Xe + NVENC): about 8 % of one CPU core (~1.6 ms per frame)

## shader documentation (.md next to the .fs) — proposal, to be decided later
- many isf shaders have a markdown file with the same name (`chaos-feedback.fs` + `chaos-feedback.md`) that explains the shader
- common structure of these files: `# title`, a short description paragraph, a slider table `| **Label** | range | default | description |`, then sections like tips / starting points; some have a ⚠ warning (e.g. flashing)
- the bold label in the table is the LABEL of the isf header, so a table row can be matched to its parameter
- ideas (none decided yet):
  1. parameter tooltip: hover over a parameter name shows its description from the slider table
  2. "?" button on the effect / generator header (only when an .md exists): opens the whole document rendered (e.g. crate egui_commonmark, tables included) in its own window that can stay open
  3. finder: hover over a shader file shows the first description paragraph
  4. ⚠ marker: if the .md contains a ⚠ warning, a small grey ⚠ next to the shader name in the finder and the detail, the warning text on hover
- read-only: the .md and .fs files are never changed; reloaded when the file changes (like the shaders)

## audio
- no audio: mp4 clips play without sound, no audio input / audio-reactive effects

## gpu (hybrid laptops)
- the gpu in use is shown in the output options ("gpu: ...") and printed at start
- option "render on NVIDIA GPU (applies on restart)" in the output options, or start with `--nvidia`:
  videoping restarts itself once with PRIME render offload
- measured on the Pop!_OS laptop (Wayland, 120 Hz 3K panel): native Wayland ~53 fps (Intel) / 30 fps (NVIDIA);
  X11 (XWayland) 60 fps on both but the ui is upscaled (blurry) on the HiDPI panel — open decision

## documentation
- features.md = spec (decisions), manual.html = end-user manual as one HTML page (dark like videoping, table of contents; concept, tour with screenshots in docs/, playing, performance, files, troubleshooting), install.md = installation only, dev.md = development
- user-visible changes go into manual.html in the same step as features.md

## platform
- Linux only for now (no Windows / macOS builds). The code still has Linux-specific bits (libndi.so names / paths, DejaVu font paths); a port would need those plus native CI builds.

## stack
- ui: egui / eframe (glow renderer), the same OpenGL context renders the video and the ui
- mp4 playback: GStreamer (playbin, hardware decoding via VA-API when available); playmodes via rate seeks
- NDI: libndi.so (NDI 6 runtime) is loaded at runtime, not bundled
- autosave: `~/.local/share/videoping/autosave.json`; blackout/freeze always start off
- install / dev setup: see install.md and dev.md
