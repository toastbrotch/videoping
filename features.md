videoping
---------

videoping is a live vjing application, 5 columns, all column-borders are moveable to arrange, fullscreen, reduce ui-clutter, slick, darkmode, cleaned-up, most important buttons are directly available. foldout/tabs only when absolutely neede. maybe resolume 2.41 kind or vdmx , mcp and api, borrow from artcraft filmcraft, it is used by mouse but should be played with midicontrollers ultimately). keep in mind this is my personal vjing tool. for the midi mapping i have a 8x8 led-pushbutton controller (bhoreal https://github.com/toastbrotch/docs.bhoreal.com-archive/blob/master/docs/start.md ) and a 4x4 led enhanced rotary encoder controller (midifighter twister)

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
| 2 prepare grid | 4x8 slot grid, below it: master (blackout, freeze, brightness), global bpm tapper, patch-bay |
| 3 prepare detail | prepare preview pinned on top, below it (scrolls): clip interface, effect stack; default width ~30% of the window |
| 4 mixer + output | main output preview pinned on top, below it (scrolls): mixer (3 layers), output effects, output options; takes the remaining width |

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
  - live slot: border in the color of the mixer-layer it plays in (same as bhoreal LED)
  - thumbnails are static, only live or selected slots play (performance)
  - click = select (shows detail in column 3), double-click / bhoreal pad = load into selected mixer-layer
  - more than 32 instances: pages of 32 slots, the bhoreal follows the page
  - move an instance to another slot: drag & drop
  - delete an instance: small x on the slot
    - also when it is live: the instance is removed and its mixer-layer becomes empty
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
  - speed, two modes (toggle):
    - free: speed slider
    - bpm: set how many beats the part between the limiters lasts (e.g. 1, 2, 4, 8, 16), speed adapts to the global bpm
      (example: 120 bpm, 8 beats -> the limited part plays in 4 s, regardless of the clip length)
    - ½x and 2x buttons: halve / double the speed instantly (in bpm mode: double / halve the number of beats)

- effects (layering top-down, max 8 per clip)
  - reorder: drag & drop
  - remove: small x on the effect row
  - active/inactive
  - all parameters (only on click folded out)
    - each parameter has a range slider
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
- mark the selected layer

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
- bpm is always an integer (tap, slider and ½x/2x round to whole bpm)
- tap button sets the bpm
- bpm slider: change the bpm globally by hand (e.g. slow everything down), tap and slider set the same value
- ½x and 2x buttons: halve / double the bpm instantly
- drives jumpcut (clips and parameter ranges jump every N beats)
- drives the speed of clips in bpm mode (beats per limited part)
- drives the speed of animated parameter range sliders in bpm mode (beats per cycle)

## master (blackout, freeze, brightness)
- one group in column 2, below the prepare grid, directly visible, no foldout
- global blackout button
- global freeze button
- global brightness slider (0-100%) next to them
- all three act at the very end (after the output effects), for NDI and screen output

## output effects (layering top-down)
- multiple effects, they work exactly like the effects in the prepare lane:
  - active/inactive, reorder by drag & drop, remove with x
  - all parameters with range sliders (pause/forward/backward/jumpcut, loop/once/bounce, free/bpm speed, ½x/2x)
  - mix-type, mix percentage
  - r, g, b channel


## output options (fold open)
  - resolution: fullHD (default)
  - screen
  - ndi name
  - NDI library folder (folder with libndi.so.6, saved; empty = default places)

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
- rendering with OpenGL (rust crate glow), not wgpu: isf shaders in GLSL 1.20 / ES 1.00 style (gl_FragColor, IMG_NORM_PIXEL, TIME, RENDERSIZE) run as they are
- the host provides the standard isf uniforms (TIME, RENDERSIZE, inputImage, ...) plus the non-standard `uInstanceSeed`:
  - one fixed random value per instance (like the ISFBrowser host does), so two instances of the same shader look different
  - a duplicated instance gets a new uInstanceSeed
  - uInstanceSeed is saved, so an instance looks the same after save/load
- parameter LABEL from the isf header is shown in the ui (falls back to NAME)


## stack
- ui: egui / eframe (glow renderer), the same OpenGL context renders the video and the ui
- mp4 playback: GStreamer (playbin, hardware decoding via VA-API when available); playmodes via rate seeks
- NDI: libndi.so (NDI 6 runtime) is loaded at runtime, not bundled
- autosave: `~/.local/share/videoping/autosave.json`; blackout/freeze always start off
- install / dev setup: see install.md and dev.md
