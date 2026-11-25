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
- files in website/logo/: videoping-icon.svg / -512 / -128 / -32.png, videoping-logo.svg / .png (lockup, text as outlines), videoping-logo.vectorcraft (editable source, made with VectorCraft)
- v2 "glitch" (same mark, broken like a damaged video signal): files videoping-v2-icon.svg / -512 / -128 / -32.png, videoping-v2-logo.svg / .png
  - displacement: 4 horizontal bands of the mark cut out and shifted sideways (tearing / datamosh)
  - channel split: ghost copies of the play triangle (and of "ping" in the lockup) offset left in blue and right in orange, like the R G B switches drifting apart
  - scanlines over the icon (20 %)
  - no displacement band through the wordmark: at the top / bottom of the letters the shifted curves read as strike lines, the word stays readable
  - large sizes only (splash, manual, posters): at 32 px the glitch turns into mud, so window / taskbar icon stays v1
- all four artboards (icon, lockup, icon v2, lockup v2) are in website/logo/videoping-logo.vectorcraft
- v2 is the official logo; where it is used:
  - window icon of videoping and of the output window (built into the binary: website/logo/videoping-v2-icon-256.png)
  - launcher / dock icon: packaging/videoping.desktop (app id "videoping") + icon theme in ~/.local/share/icons/hicolor: v2 from 48 px up and scalable, v1 for 16 and 32 px (install.md); on Wayland the window icon comes from there
  - manual.html: v2 lockup as page header, v2 icon in the table of contents, favicon (v1 32 px for the browser tab, v2 128 px)
  - install.md: v2 lockup on top
  - page background of the manual = #141414, the same as the app and the logo background, so the lockup has no visible box

# tabs

the ui has top-level tabs:

- **live**: everything described below — the default tab
- **midi mapping**: built (see "midi mapping")
- planned, in this order (decided 2026-11-21): first the mcp server (no tab), then:
  - **ai**: built 2026-11-22 (see "ai tab")
  - **shader editor**: built 2026-11-22 (see "shader editor")
- **projection mapping**: built 2026-11-22 (see "mapping")
- sequencer: no tab until there is a real sequencer (the greyed-out placeholder was removed 2026-11-22)

everything below belongs to the tab "live".

# layout of the tab "live"

5 columns, all borders draggable:

| column | content |
|---|---|
| 1 finder | finder preview, finder |
| 2 prepare grid | 4x8 slot grid, below it: master (blackout, freeze, brightness), global bpm tapper, record, output options, general config |
| 3 prepare detail | prepare preview pinned on top, below it (scrolls): clip interface, effect stack; default width ~30% of the window |
| 4 mixer + output | main output preview pinned on top, below it (scrolls): mixer (3 layers), output effects; takes the remaining width |

- the prepare preview (column 3) and the main output preview (column 4) always stay on top, also when the rest of the column is scrolled
- both previews always have the same size (the width of the narrower of the two columns)

# save / load

- everything is saved:
  - prepare grid (all pages, instances, their clip settings, effects and all parameters incl. range settings)
  - mixer layers (loaded instances, mix-type, mix %, r/g/b, active, order, selected layer)
  - bpm, output effects, master (brightness), output options
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

## quit guard
- (2026-11-22) the main window's × / Alt+F4 asks "quit videoping?" (modal: what stops — output window, NDI, a running recording is finished and saved, unsaved shader editor changes are lost; the session autosaves); quit / cancel, Esc or a click beside = cancel; the output window's own × only closes the output window

## finder preview

- shows preview if something is clicked in the finder
- stoppable (2026-11-22): × at the top right of the preview (like every other ×) (while something is chosen), or a click on the previewed file again; then the preview instance is dropped (nothing renders)

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
- overview with many files (decided + built 2026-11-22):
  - accordion: the category headers (clips · generators · effects · NDI sources) always stay visible, each with its number of files; one category is open and fills the rest of the column with its own scroll; a click on another header switches, every category keeps its scroll position; a click on the open header closes it (all closed = only the headers; a search opens a category with hits again)
  - search ("search all") covers all categories: the headers show the hits (accent when > 0); when the open category has no hit, a category with hits opens; Esc in the field clears it
  - folders as groups: inside a category the files are grouped by the folder they are in (shown from the finder folder's name on, e.g. "clips/loops/abstract", with the number of files); groups are collapsible, open by default when a category has 3 or fewer, always open while searching; a folder with one file and a category with one folder show the files without a header
  - favorites: ☆ appears left of a file on hover, a click pins it (★, accent) under "★ favorites" on top of its category (it stays in its folder too); saved in the session, kept by "new" (like the finder folders)
  - later: "in use" marker (files in the grid / live), clip thumbnails, recently used
- nested finder folders (a folder inside another finder folder): every file is listed once (2026-11-22)
- the finder folders are watched: every 2 s the known folders are checked (modification time only, cheap); a new, deleted or renamed file (e.g. a finished recording, copied clips) appears without a manual refresh
- videopong clips (decided 2026-11-22, see "videopong")
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
    - the name is one line cut with "…" to the slot width minus the room of the × (bottom right), so a long name never hides the × (2026-11-22)
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

- effects (layering top-down, max 16 per clip; decided 2026-11-24, was 4 to fit the bhoreal, which is just one setup: a midi profile maps as many as it has pads for, the rest is reachable via learn, the encoders that follow the opened effect, mcp and the ai tab; the 16 is only a safety cap against an ai / MCP client adding effects in a loop during a show; every effect costs one render pass, visible in the GPU meters)
  - reorder: drag & drop
  - drag handle "≡": button-sized grab area (26x18) at the start of the effect row, hand cursor
  - the effect stack (clip effects and output effects) reaches down to the bottom of its column: the whole empty space below is a drop area
  - no horizontal lines in the prepare detail (below the name / duplicate row, between the clip interface and the clip effects)
    nor between the mixer layers and the output effects
  - move to the output effects and back: drag the "≡" handle onto the other effect stack (onto a row = insert there,
    onto the empty area = append); the effect keeps all its settings (parameters, ranges, animation, mix, r/g/b);
    a clip (or the output) that already has 16 effects refuses it
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
    - definitions (top = the layer / effect result, l = its luma, BT.601, m = mix slider 0..1):
      - alpha: crossfade, weight = top alpha × m
      - lumakey: dark parts transparent; weight = smoothstep(lo, lo + 0.3, l) × alpha, lo = 0.05 + (1 − m) × 0.95
      - inverse lumakey: the same key on the inverted luma, bright parts transparent; weight = smoothstep(lo, lo + 0.3, 1 − l) × alpha
        - not the inverted matte of the lumakey (1 − key): that also removed all mid tones (fixed 2026-11-21)
      - add: bottom + top × m
    - in lumakey / inverse lumakey the mix slider moves the cut instead of fading ("luma fade", decided 2026-11-21):
      - 100 % = only black (iLk: white) is keyed out, soft edge 0.05..0.35 (the key as before)
      - going down the cut moves towards white (iLk: black): the dark (iLk: bright) parts disappear first, the brightest (iLk: darkest) last
      - 0 % = nothing left, so 0 % = off and 100 % = fully on stays true for every mix-type
      - one slider instead of an extra threshold slider: more expressive live, no extra control; same for effects (same mix code)
      - hover on the mix slider explains it per mode
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
    - an empty layer: the prepare detail becomes empty (like a click on an empty slot), so the twister parameter encoders control nothing (fixed 2026-11-22); the same for the bhoreal select pads
    - also when clicking one of the layer's controls (the controls still do their own action)
- mark the selected layer

## record
- own box in column 2, below bpm (above the output options), directly visible
- REC button: records the output (after master, the same picture as NDI) to a file
  - while recording: "● REC" (red) plus running time, file size and resource meter; hover: path, frames, dropped frames, encoder
  - the target folder is not shown in the box (only in the REC hover); it is set in the general config (recording folder)
  - files are named `videoping-YYYY-MM-DD_HH-MM-SS.mp4`
  - keeps recording through new / load; ends when the output resolution changes; finished properly on quit
  - disabled (with the reason on hover) when no hardware encoder works

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
- max 16 output effects (decided 2026-11-24, like the effects per clip; was 4)
- multiple effects, they work exactly like the effects in the prepare lane:
  - active/inactive, reorder by drag & drop, remove with x
  - can be dragged to the selected clip's effect stack and back (keeps all settings)
  - all parameters with range sliders (pause/forward/backward/jumpcut, loop/once/bounce, free/bpm speed, ½x/2x)
  - mix-type, mix percentage
  - r, g, b channel


## output options (fold open)
  - placed in column 2, below record
  - resolution: fullHD (default)
  - frame rate: 25 / 30 / 50 / 60 (default 60) for screen and NDI output
    - video (instances, mixer, output, NDI) is rendered at this rate, the ui repaints at the display rate
    - NDI announces this frame rate
  - screen: buttons 1, 2, 3 ...; the fullscreen output window goes to this screen (also on Wayland)
    - "connected: eDP-1, HDMI-A-1" shows the connected outputs as a hint (their order can differ from the numbers)
  - output window on/off, fullscreen on the chosen screen (F / double-click in the output window)
  - ndi name
  - NDI library folder (folder with libndi.so.6, saved; empty = default places)

## settings layout (output options, general config)
- decided 2026-11-21: both are made of titled groups (thin frame, small grey title), labels in one column, controls aligned
  - output options: picture (resolution, frame rate) · screen (screen, connected, output window, fullscreen) · NDI (send + status, name, library folder) · GPU (renders on, NVIDIA)
  - general config: display (resource meters) · recording (folder, add to finder) · MCP (server + status, port, read only, connect command)
- "NDI sending" is grey, not green (colors belong to the layers); errors stay red

## general config (fold open)
- placed in column 2, below the output options
- settings of the program itself, not of a session: own file `~/.config/videoping/config.json`; save / load / new do not change it
- resource meters on/off (default on): to compare the overhead or switch them off for a show
- recording folder (empty = `~/Videos/videoping`); "add to finder" adds it to the finder folders, so recordings can be dropped onto a slot

## output preview
- main output preview at the top of column 4, always visible (does not scroll away)

# control: one registry for midi, mcp (and osc)

decided 2026-11-21: midi and mcp are two front ends to the same thing, so there is one shared base:

- control registry (src/control.rs): every controllable thing has a stable address, a kind and can be read and set
  - kinds: trigger (may still read a state, e.g. "is selected"), toggle, range (min..max, maybe whole numbers), choice; some addresses are read only
  - numbers in addresses count from 1, like the ui
  - addresses: /master/blackout|freeze|brightness, /bpm (+ /bpm/tap|half|double), /record, /page, /layer/N/select|active|mix|mixtype|r|g|b|unload, /slot/N/load|select|state, /slot/N/fx/K/active|mix|mixtype|param/P, /slot/N/param/P (generator), /out/fx/K/...
  - instances are addressed by their slot (decided 2026-11-21, was an internal instance id): /slot/9/... = slot 9 as in the grid (top-down per column, page 2 starts at 33); a binding to a slot controls whatever is in that slot
  - parameters are addressed by number (decided 2026-11-21, was the isf input name): /param/3 = the 3rd parameter, counted top-down like the parameter list in the ui (isf header order, image inputs left out); the names change constantly while shaders are developed, the number fits every shader and the twister ("the n-th parameter"); colors and events keep their number but are not controllable yet
  - aliases: /layer/N/fx/K/... and /layer/N/param/P reach whatever plays in layer N (the bhoreal effect pads); /page/slot/N/... = slot N of the page shown (the bhoreal follows the page)
  - every address has a description in words ("slot 9 (testclip) · gameboy-camera · Kontrast"), shown in the midi mapping tab
  - parameters: float / long -> range from the isf header (long rounded), bool -> toggle, point2D -> /x and /y
  - set clamps to the range; brightness up during blackout ends it (like the slider); tap and record go through the app like the buttons
  - change notifications: a snapshot of all values, diffed against the last one -> what changed, also by mouse (needed for led feedback and mcp)
- on top of it, in this order:
  1. registry (done)
  2. midi: bhoreal as a fixed preset with led feedback (below)
  3. mcp server inside videoping — built 2026-11-21:
     - transport: mcp "streamable http" on 127.0.0.1 only, port 7466 (general config); plain json answers, no event stream; requests from a foreign web page (Origin header) are refused
     - connect claude code once: `claude mcp add --transport http videoping http://127.0.0.1:7466/mcp`
     - tools: state (overview: mixer, master, bpm, slots with effects, output effects, fps, gpu), list_addresses (address, kind, meaning; filter), get, set (number / true-false / choice; triggers without value), list_files (finder: clip / generator / effect / ndi), load_file (into a slot, default the first free one), add_effect (on a slot's instance or the output), frame (png of the output, the preview or a slot; width default 640), shader_errors, shader_info (a shader's parameters in videoping's numbering)
     - tool calls are answered by the app at the next frame (frames are read on the gpu: scaled down, one small read)
     - every change shows in the status line ("mcp: /layer/1/mix = 0.5"); "read only (for shows)" in the general config refuses all changes, reading still works
     - general config: mcp server on/off (default on), port, read only, the claude code command to copy
     - paths given to load_file / add_effect are stored absolute, like the finder does
  4. midifighter twister (midi learn and the "midi mapping" tab are done)
- after the mcp server (decided 2026-11-21):
  - ai tab: a thin chat client inside videoping on the same tools as the mcp server — built 2026-11-22, see "ai tab"
    - provider + model setting (anthropic, openai-compatible incl. local ollama / lm studio); settings and key in the ai tab itself (stored in config.json)
    - every tool call visible (what the ai changed), undo; "ask before applying" for shows; output / preview frames as images when the model supports them
    - caveats: local models are often weak at tool use; the key lives in a config file; every message costs api usage; for playing live the controller stays faster
  - shader editor tab, with ai: write / change isf shaders inside videoping with a live preview (hot reload), the ai sees the output frame and the compile errors — built 2026-11-22, see "shader editor"
- midi must feel instant: input on its own thread, applied at the start of the next video frame, not through the ui
- mcp use cases (most useful first): writing shaders with claude live (hot reload + look at the output frame), preparing sets, asking / debugging (state, resource meters), playing via an agent (least)
- osc: not for now (decided 2026-11-22); the registry would carry it if it comes back

# midi mapping

decided 2026-11-21: generic for any midi controller, edited in the tab "midi mapping"; bhoreal and twister are built-in profiles of the same engine

- engine: knows no devices, only bindings: midi message (channel, note / cc) <-> control registry address
  - modes: trigger (press = fire once), toggle (press = on/off), momentary (on while held), absolute (cc 0-127 -> the address range), relative (endless encoder: +/- steps)
  - feedback to the controller from the registry change notifications (leds show the true state, also after mouse changes): toggle / trigger -> velocity per state, choice -> velocity per value (e.g. slot state empty / loaded / L1 / L2 / L3 -> bhoreal colors), range -> value back as cc (twister rings)
  - relative targets in the registry for controllers: /page/slot/N (slot N of the current page, the bhoreal follows the page), /layer/N/fx/K/... (what is loaded in layer N), /sel/... (the selected instance; /sel/param/#n = its n-th parameter)
- profiles: one per controller, picked automatically by midi port name; bhoreal and twister ship built in, editable
  - stored in the general config (~/.config/videoping/midi/<profile>.json), not in the session: a controller mapping is setup, like the output options
  - decided 2026-11-21: saved automatically (0.5 s after the last change and on quit), no save button, nothing is lost
  - all files in the folder are loaded at start; a file with the name of a built-in replaces it
  - profile bar in the tab: name (rename: type + Enter; built-ins keep their name), new (empty, for the selected device), duplicate (a copy not matched to any device yet), delete (asks "really delete?"; built-ins can not be deleted), built-in (back to the original, removes your file)
  - choosing a profile for a device in the dropdown moves the device name to that profile (one profile per device name), so it is picked automatically next time; switching between variants (Bhoreal / Bhoreal club) = the dropdown
  - import / export: later; the files are plain json in the folder shown in the tab
- tab "midi mapping":
  - devices: midi inputs / outputs (alsa), on/off, profile, activity light, monitor of the last incoming messages
  - bindings of the selected device: control, mode, target address, feedback
  - addresses: the registry as a searchable tree; choose an address, "learn", move a control -> binding created
- order: 1. engine + bhoreal profile with leds (playable) — done 2. the tab with bindings table and learn — done 3. twister profile
- the tab as built (2026-11-21):
  - top bar: "live" and "midi mapping" switch the tab; the output keeps rendering in every tab
  - left: every midi input (except videoping's own ports and "Midi Through"), activity light, profile per device (dropdown: none / any profile; choosing one makes that profile also match the device name next time), monitor of the last messages
  - middle: the bindings of the selected device's profile: control (channel, note / cc), mode (dropdown), target address (editable; "←" takes the address chosen on the right), feedback (off / on values editable), × to remove; the row of the control just moved lights up in the accent color; above it the profile bar (see profiles)
  - right: the writable addresses (searchable) with their range and their meaning in words; a foldable "what the addresses mean" legend; learn: choose an address, press learn, move a control
  - a target that does not exist right now (no such effect in that layer, empty slot) shows "not there now"; hover on a target shows what it is
  - learn picks the mode from the address: trigger / toggle for buttons, cc on a range -> absolute (fader / knob) or relative (an encoder step: value 1, 63, 65 or 127); a button on a range is refused; a learned control replaces an older binding of the same control
  - a device without profile gets a new one on its first learned control (named after the device), or with "new profile"
  - cc buttons: value 0 counts as release (trigger / toggle / momentary)
- a binding can read its feedback from another address than it triggers (a slot pad triggers /page/slot/N/load, its color comes from /page/slot/N/state)

## bhoreal (8x8 led-pushbuttons)

8 vertical columns, 8 pads each (LED shows state):

| column | 8 pads (top-down) |
|---|---|
| 1 | select mixer-layer 1-3, activate/deactivate mixer-layer 1-3, global blackout, record |
| 2-5 | select instance in the prepare lane = load it into the selected mixer-layer (4x8 = 32 instances) |
| 6-8, rows 1-4 | activate/deactivate effects 1-4 of the clip in mixer-layer 1 / 2 / 3 (effects 5-16 have no pad) |
| 6, rows 5-8 | activate/deactivate output effects 1-4 |
| 7-8, rows 5-8 | free (8 pads) |

- protocol (firmware bhoreal_slim_midi, docs table): pads send note on channel 1, note 0..63, press velocity 64, release velocity 0 (note on); leds: note on, velocity = hue on the firmware color wheel (velocity × 8 -> red -> green -> blue -> red), velocity 0 = off; the usb port name is "Arduino Leonardo" according to the docs
  - orientation: played with the usb cable at the top right
  - the device can present itself turned by 180° with the cable in the same place (seen 2026-11-21 vs 2026-11-22; a hobby device): so the profile does not rely on a fixed numbering (decided 2026-11-22)
    - bindings use the pads as you see them (row × 8 + column from the top left); the profile's grid (size 8, orientation: one of 8 placements, 4 rotations each also mirrored) translates to the device's notes, in and out
    - calibrate in the midi mapping tab: press "calibrate", then the pad next to the usb cable (top right); one corner press is enough, the rotation wins over the mirrored placement; saved with the profile
    - default orientation: turned 180° (as on 2026-11-21: top right = note 56)
    - the monitor shows both: the device note and the pad as you see it
  - pad order (checked on the hardware 2026-11-21, leds and pads alike): with the cable top right the top left pad is note 63, the pad right of it 62, the bottom right pad note 0: note = 63 − (row × 8 + column), column / row from the top left; the usb port shows as "Arduino Leonardo MIDI 1"
    - (a first test gave the opposite corner: the device was held turned by 180° then)
- the leds can only show full brightness, no dimming (changed 2026-11-21, was "bright / dim"): plum (the accent) = "there, but not on"
  - layer colors as hue: layer 1 = #1B6898 (blue, velocity 72), layer 2 = #388E3C (green, 44), layer 3 = #C8640A (orange, 10), plum = 108
  - column 1: select pad in the layer color when that layer is selected, else off; layer on/off pad in the layer color when on, else off; blackout red when on; record red while recording
  - columns 2-5: slot of the current page: live = color of its layer, loaded but not live = plum, empty = off
  - columns 6-8, rows 1-4: effect pads: active = color of the layer, there but off = plum, no effect = off
  - column 6, rows 5-8: output effects: on = yellow (the output belongs to no layer), there but off = plum, no effect = off
- every attached controller is cleared completely on connect (startup, plugging in), when its profile changes, and on quit (decided 2026-11-21), also controllers without a profile: notes 0..127 and ccs 0..127 on channel 1 set to 0, plus every bound control on its own channel
  - a profile can limit it (clear_notes / clear_ccs): bhoreal = notes 0..63 only (its firmware does not range-check higher notes), no ccs; twister = ccs 0..63 (the rings of all 4 banks, also encoders without a binding), no notes
- built-in profile "Bhoreal" of the generic midi engine (src/midi.rs), picked for ports named "bhoreal" or "leonardo"; tested with a virtual midi port (self-test), not yet with the real device
- global blackout: same as the master button in column 2; bottom left = record (REC), decided 2026-11-21 (was freeze; freeze stays in the ui and can go on a free pad via learn)
- more than 32 instances: paging, the bhoreal shows the current page of the prepare grid

## midifighter twister (4x4 rotary encoders)

- protocol (checked on the hardware 2026-11-21): encoders numbered row by row from the top left (top left = cc 0, top right = cc 3, bottom right = cc 15); turning: cc on channel 1, absolute 0..127; pushing: the same cc on channel 2, 127 = pressed, 0 = released; a cc sent to it on channel 1 sets the led ring
- built-in profile "Midi Fighter Twister" (default layout, decided 2026-11-21):
  - column 4, top-down: mix of layer 1, layer 2, layer 3, master brightness (absolute; the led ring shows the value, also after mouse changes, so turning never jumps)
  - columns 1-3 (decided + built 2026-11-22): parameters 1-12 of the focus of the selected instance, row by row (top left = 1, row 2 starts with 4), absolute with ring feedback; push (channel 2) = reset that parameter to its isf default
    - focus = the effect opened last (▸ -> ▾) in the selected instance, marked ◎ next to its name; no focus: the generator's own parameters, else effect 1 (runtime only, not saved)
    - registry: /sel/param/N (and /sel/param/N/reset, /sel/fx/K/...) resolve to the selected instance's slot; any .../param/N/reset = back to default
    - switching the focus moves the rings to the new values (feedback), so turning never jumps
- the earlier idea below is what was built (bank 2 not yet):

earlier idea, decided 2026-11-21:
- bank 1: the 16 encoders = the first 16 parameters of the selected effect (or of the selected generator), relative mode
  - push an encoder = reset that parameter to its isf default (same as right click in the ui)
  - led ring = the value
- bank 2: layer mixes and master
- like every profile: editable in the midi mapping tab

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

## ai tab — built 2026-11-22
- a chat inside videoping (top-level tab "ai") on the same tools as the mcp server (state, list_addresses, get, set, list_files, load_file, add_effect, frame, shader_errors, shader_info); system prompt = the mcp instructions + "be short, act with the tools, say in one line what you changed"
- providers: Anthropic Messages API (raw HTTP with ureq, API version 2026-10-21) and OpenAI-compatible chat completions (OpenAI, Ollama, LM Studio: own url)
  - anthropic: model default claude-opus-5-5 (▾ menu: opus 5.5, sonnet 5.5, haiku 5.5, fable 5.1, or type any), effort low / medium / high (default low: live use wants fast answers), adaptive thinking (model default), the assistant content goes back unchanged (thinking blocks), top-level prompt caching, server-side refusal fallback ("default") on the models that have it, max_tokens 16000, no streaming (answers are short; "thinking … n s" while waiting)
  - openai-compatible: tools as functions; tool results as tool messages, images as a following user message
- settings in the tab (stored in config.json, which is written with mode 0600 because of the keys): provider, model, key (password field, "show"), url (empty = official), ask before changes, images
  - key: paste (clipboard: eframe feature "clipboard" was missing, so paste worked in no text field; fixed 2026-11-22), "get a key" link to the Anthropic console, empty field = env ANTHROPIC_API_KEY / OPENAI_API_KEY (the hint shows "from ...")
  - not possible: the Claude Code / claude.ai subscription login — that is for Anthropic's own apps; videoping needs an API key (billed per use in the console)
- every tool call is a line in the chat: name, arguments, result (4 lines, hover for all), the image of `frame` as a thumbnail; `set` has undo (the value before; the model is told with the next message)
- "ask before changes" (default off): set / load_file / add_effect wait for apply / skip / apply all; reading tools never wait
- the mcp "read only" setting does not apply to the ai tab (it is for clients from outside); status line shows "ai: ..." for its changes
- requests in a worker thread; tool calls at a frame (they need the gpu for `frame`); the loop runs on in other tabs; at most 25 tool rounds per message; stop answers open calls with "stopped by the user"
- not saved: a new start or "new chat" = a new conversation; switching provider starts a new conversation (other message format)
- token count of the chat at the bottom right (input incl. cached)
- self-test: VIDEOPING_SELFTEST_AI=<url of a mock server> (scratchpad mock: answers set + frame, then text) checks the whole loop without api cost
- open: streaming, a cheaper model for quick questions, images from the prepare preview / slots in the chat

## videopong (videopong.net clips in the finder) — decided + built 2026-11-22
- like the Resolume plugin ../ffgl/source/plugins/VideoPong: the public JSON API https://www.videopong.net/api (no key): /clip/search/<query>/<offset>/<count> (count max 50), /clip/random/<n>; answer {status: {error, errorstring}, data: [[clip, user], ...], count}; clip: id, title, duration, framerate, resolution, license, url_thumb (jpg), url_preview_mp4, url_videopong; user: name
- only the public preview mp4 (480x360 H.264): the full-quality download (url_download) needs a videopong login -> later, if wanted; checked 2026-11-24: the preview is 480x360 for every original (640x480, 1280x720, 1920x1080); said on the website, in the README and the manual (decided 2026-11-24)
- finder: a 5th category "videopong.net" (named so 2026-11-22) in the accordion: search field (Enter) + "random"; results 50 at a time, "more" loads the next 50; each row: thumbnail, title, duration, original resolution, author, license; ✓ = downloaded
- click = download (if needed) + finder preview; double click = download + first free slot; drag onto a slot = download + into that slot (status line shows the download); a downloaded clip is a normal mp4 (same as a finder clip)
- clips cached in ~/Videos/videopong/<title> [<id>].mp4 (visible, reusable, sessions find them offline); with the first download (any download, also an already cached one used again) the folder is added to the finder folders (2026-11-22), so the downloads also appear under "clips"; thumbnails cached in ~/.cache/videoping/videopong-thumbs/
- network in background threads (search, 4 thumbnail workers, downloads with progress); the ui never waits
- built: src/videopong.rs (ureq + image/jpeg for the thumbnails, 160x120 max), finder category FileKind::Videopong, Drag::Videopong (downloaded on drop), instance name = the title (without " [id]"); a global "search all" never switches to videopong (its count is the result list, not hits)
- self-test: VIDEOPING_SELFTEST_VP=<query> (real network: search, thumbnails, first clip into slot 3)
- later (open): login for the full-quality download; MCP / ai tools to search and load videopong clips

## mapping (projection mapping) — decided + built 2026-11-22
- scope: 1 slices (an input rectangle of the composition placed on the output with a corner pin), 2 masks (polygons that black out parts of the output, soft edge, invertible), 3 test grid; not now: mesh warp, soft-edge blending, per-slice on/off / dimming, several mapped outputs
- applied at the very end, after master, to the screen output (fullscreen window); output options "mapping also on NDI" and "mapping also on the recording" (default off: NDI and the recording stay the clean composition)
- presets: a mapping belongs to a venue, not to a set: named presets in ~/.config/videoping/mapping/<name>.json (like the midi profiles), saved by themselves; the session only stores which preset is active and whether the mapping is on
- tab "projection mapping" (renamed from "mapping" 2026-11-22): preset bar (choose / new / duplicate / rename / delete, mapping on, test grid, also NDI, also recording); list of slices and masks (add, select, rename, delete) with the numbers of the selected one; input view (the composition with the slice rectangles) and output view (the projector image with the slice corners and mask points), handles dragged with the mouse, arrow keys nudge the selected handle (1 px, Shift 10 px); masks: double click on an edge adds a point, right click on a point removes it
- handles (fixed 2026-11-22 after the first test: corners were hard to catch): catch radius 16 px, also outside the view (corners on its border), nearest handle wins (the selected item slightly preferred); cursor: crosshair over a handle, move over an inside, grabbing while dragging; dragging is relative (a handle grabbed beside its point does not jump to the mouse); the last grabbed handle has a white ring + its position in px; arrow keys (read from the raw key events, so nothing else takes them) move it 1 px, Shift 10 px
- the output window shows the mapping live while editing (with the test grid when on); the operator previews (live tab) stay unmapped
- gpu: per slice one pass with a perspective-correct corner pin (homography), per mask one pass that darkens (blending); cheap for a few slices
- coordinates normalized 0..1, origin top left, independent of the output resolution
- built: src/mapping.rs (types, homography, presets: saved 0.5 s after a change and on quit; the self-test uses its own folder), gfx SLICE_FS / MASK_FS (polygon up to 64 points, crossing test + edge distance for the soft edge) / GRID_FS, Renderer::render_output(.., MapFrame) after master into `mapped`; the screen window shows Renderer::screen_tex(); the mapping is also computed while the tab is open and the mapping is off (for its output view), but only shown on the screen when on
- the session stores output.mapping (preset name), mapping_on, map_ndi, map_rec; "new" keeps them (output options)
- self-test: VIDEOPING_SELFTEST_MAP=1|grid (two slices, one corner-pinned, a soft mask; pixel checks outside / inside / in the mask)

## shader editor — built 2026-11-22
- top-level tab "shader editor": code (left), preview + parameters or ai (right), compile result (bottom), file bar (top)
- open: "open ▾" (finder generators + effects, with search), ✎ next to an effect in the effect stack and next to "generator" in the prepare detail, right click on a shader in the finder; "new generator" / "new effect" start from a small working template, the file field gets a free name next to the other shaders of that kind
- save (Ctrl+S) writes the file in the "file" field: another name = a copy; an existing other file is never overwritten; revert = back to the saved text; "•" = unsaved; opening / new with unsaved changes asks (save / discard / cancel)
- live preview without saving: the text goes to a work copy in $XDG_RUNTIME_DIR/videoping-edit/<name>.fs (a custom .vs next to the file is copied along), written 250 ms after the last key and compiled at once (Renderer::reload_shader); the real file and its live uses change only on save (then reloaded at once)
- preview instance (id EDITOR_ID): a generator alone; an effect on a copy of the selected instance (else what plays in a layer), as its last effect; params follow input changes (reconcile_params); rendered only while the tab is open
- parameters of the preview as sliders (with the .md descriptions of the saved file)
- errors: GLSL log lines (NVIDIA "0(N)", Mesa "0:N(") and JSON header errors ("at line N") mapped to file lines (the body starts after "*/"; text before the header counts); red line numbers; click an error = cursor to that line
- code view: monospace 13, line numbers, highlighting in greys + accent (comments, keywords, types, ISF builtins, numbers, JSON strings), no wrapping (scrolls sideways)
- ai (panel "ai" under the preview): its own chat with the provider / model / key of the ai tab; tools get_shader, set_shader (whole file, compiled at once, answers "compiles ✓ — kind, parameters" or the errors by file line), look (the editor preview; waits 3 frames after set_shader so the new shader is rendered), list_shaders, read_shader (finder shaders only, with their .md); system prompt: ISF in videoping (header, builtins, passes), workflow get → set → fix → look, keep the user's inputs, LABEL/MIN/MAX/DEFAULT for new inputs, fast + no hard flashing, answer short
  - it only changes the editor text, never the file; every set_shader has undo (also one that does not compile)
- MCP frame source "editor" (Claude Code can look at the editor preview too)
- self-test: VIDEOPING_SELFTEST_EDIT=<.fs> (error typed in → mapped line, fixed, file untouched); + VIDEOPING_SELFTEST_AI=<mock> = the editor's ai against a mock (get, broken set, fixed set, look)

## shader documentation (.md next to the .fs) — decided + built 2026-11-22
- many isf shaders have a markdown file with the same name (`chaos-feedback.fs` + `chaos-feedback.md`) that explains the shader
- structure used: `# title`, the first paragraph = short description, a slider table `| **Label** | range | default | description |` (bold first cell = the isf LABEL, else NAME; last cell = description, the cells between with their column names), `> ⚠ ...` lines = warnings; everything else is free
- all four ideas built:
  1. parameter tooltip: hover over a parameter name = its label + description (range, default) from the slider table
  2. "?" button on the effect header and on a "generator" line above the generator parameters (only when an .md exists): the whole document rendered (egui_commonmark 0.26) in its own window; several can be open, they stay open while playing (also in the midi tab), ? again or × closes
     - tables with a cell longer than 50 characters are shown as lists (`- **Label** · range · default x — text`): egui_commonmark table cells do not wrap; short tables stay tables
  3. finder: hover over a shader file = the first paragraph (+ the path); clips: path only
  4. ⚠ marker: a grey ⚠ after the name in the finder and next to the "?" when the .md has a warning line, the warning on hover
- read-only: the .md and .fs files are never changed; re-read when the .md changes (stat at most once a second per shader)
- MCP `shader_info` returns the .md text too (context for the ai tab later)
- self-test: VIDEOPING_SELFTEST_DOC=<effect .fs> puts that effect first on the clip and opens its doc window (screenshot)

## audio
- no audio: mp4 clips play without sound, no audio input / audio-reactive effects

## gpu (hybrid laptops)
- the gpu in use is shown in the output options ("gpu: ...") and printed at start
- option "render on NVIDIA GPU (applies on restart)" in the output options, or start with `--nvidia`:
  videoping restarts itself once with PRIME render offload
- measured on the Pop!_OS laptop (Wayland, 120 Hz 3K panel): native Wayland ~53 fps (Intel) / 30 fps (NVIDIA);
  X11 (XWayland) 60 fps on both but the ui is upscaled (blurry) on the HiDPI panel — open decision

## documentation
- README.md = the front page of the repo (GitHub, 2026-11-23): logo, early alpha + vibecoded with Claude, screenshot, the features in short, build from source, links to videoping.net and the docs, community, license
- website/install.html = the install guide as a page of the website, generated from install.md (the one source) by scripts/install-html.py; website and manual link to it (2026-11-23)
- website: section "what it plays" (2026-11-23): sources clips / ISF generators / NDI / videopong.net, "every effect is an ISF shader", a short ISF explainer (open standard from VIDVOX: GLSL + JSON header; example header -> the controls videoping builds; generator vs effect by inputImage; input types -> controls; multi-pass / persistent / custom vertex / hot reload / .md docs; editor.isf.video library, most run as they are; write your own in the editor or with the ai; files stay unchanged unless saved)
- website/index.html = the sales page for videoping (2026-11-23): one static HTML file in the repo, published at https://videoping.net (the website/ folder); links to the source on GitHub (https://github.com/toastbrotch/videoping) in the hero, the get-it section, the footer and the install page's top bar, images in website/img/ and website/logo/ (moved there 2026-11-23, with the manual: website/manual.html); the pitch is linux, shaders, ai (headline "Linux. Shaders. AI.", decided 2026-11-23; replaces "play video like an instrument"); unique selling points (in this order: shaders live, ai in the booth, made for your hands, prepare, mapping, videopong.net): made for your hands (any midi controller via learn, leds follow the state, the built-in Bhoreal / Twister profiles only as examples; decided 2026-11-23), an ai in the booth, shaders live (editor + ai), prepare then go live (render priority), map any surface, videopong.net built in; the community story (from the maker of videopong.net and linux4vjs.net; the maker's own why, in the community section and the README: loves Resolume and VDMX, hundreds of gigs with them, loves Linux, waited a decade for a Linux release, now builds the tool with AI; 2026-11-24); show-safe details, "one program instead of five", specs, build from source; only real facts and numbers (videopong.net: 27,000+ clips, Creative Commons, non-profit, ad-free; linux4vjs.net archived since 2020); it says clearly (hero badges, section "made with Claude", footer) that videoping is an early alpha and completely vibecoded with Claude (Claude Code): ~14,000 lines of Rust, 0 typed by hand, first line 17.11.2026, one week; what early alpha means (rough edges, few tested machines, build from source, formats may change)
- features.md = spec (decisions), website/manual.html = end-user manual as one HTML page (dark like videoping, table of contents; concept, tour with screenshots in website/img/, playing, performance, files, troubleshooting), install.md = installation only, dev.md = development
- user-visible changes go into website/manual.html in the same step as features.md

## license — decided 2026-11-23
- the program: GPL-3.0-or-later (`LICENSE`, `license` in Cargo.toml); forks that are distributed stay open; all dependencies are permissive (MIT / Apache / BSD / Zlib / ISC, one MPL-2.0), GStreamer (LGPL) is linked dynamically, libndi is loaded at runtime and never shipped
- the shaders that come with videoping (isf-effect/, isf-generator/, their .md docs): CC BY-NC-SA 4.0 (`LICENSE-shaders.md`); they are data loaded at runtime, not part of the program; the basic shaders are still to be written; third-party shaders keep their own license
- the name "videoping" and the logo are covered by neither license
- shown in install.md (and so install.html) and in the website footer

## single instance
- only one videoping at a time (decided 2026-11-22): two would fight over the midi controllers (leds), the output window, NDI and the autosave
- an exclusive lock on $XDG_RUNTIME_DIR/videoping.lock, released by the system when the process ends (also after a crash); a second start ends at once with "videoping is already running" (terminal + desktop notification via notify-send)
- the self-test is exempt and leaves the real midi controllers alone (only the midi self-test uses its own virtual device), so it can run beside a session

## platform
- Linux only for now (no Windows / macOS builds). The code still has Linux-specific bits (libndi.so names / paths, DejaVu font paths); a port would need those plus native CI builds.

## security — checked + fixed 2026-11-24
- MCP server: browsers are kept out (DNS rebinding, web pages posting to localhost): the Host header must be exactly 127.0.0.1 / localhost / [::1] (with any port), an Origin header (if there is one) too, with http/https and nothing after the host; "null" is refused; a POST must be Content-Type application/json (a browser then needs a preflight, which gets 405); before, "http://localhost.evil.com" passed a starts_with check and "null" was allowed
- MCP and ai tab tools touch files only from the finder folders: load_file, add_effect and shader_info refuse other paths (shader_info read the .md next to any path, and that text went to the ai provider); read_shader already did this
- prompt injection: the instructions (MCP and both ai chats) say that text from files and the network (shader code, comments, .md docs, file and clip names) is data, never instructions; such requests are ignored and mentioned; the manual advises "ask before changes" in shows with shaders from strangers
- config.json (holds the ai keys) is created with mode 0600 (temp file + rename), never readable by others, not even for a moment
- videopong.net: https only (ureq https_only), also for preview and thumbnail urls from the api; clips without a usable id are skipped
- install.md: keep GStreamer up to date (it parses video files from anywhere, also shared clips)
- dependencies: `cargo audit` (dev.md); 2026-11-24: cookie 0.18.3 and url 2.5.9 (advisories RUSTSEC-2026-0594 / -0592), yanked yoke-derive updated; clean
- open (not fixed, decided 2026-11-24): the NDI library folder is stored in the session; a session file from someone else could make videoping load a foreign libndi.so

## stack
- ui: egui / eframe (glow renderer), the same OpenGL context renders the video and the ui
- mp4 playback: GStreamer (playbin, hardware decoding via VA-API when available); playmodes via rate seeks
- NDI: libndi.so (NDI 6 runtime) is loaded at runtime, not bundled
- autosave: `~/.local/share/videoping/autosave.json`; blackout/freeze always start off
- install / dev setup: see install.md and dev.md

## shader pack — generators built 2026-11-24
- folder `isf-pack/generators/` (committed; the user's own shaders in isf-effect/, isf-generator/, ivo-isf-* stay private): 13 ISF
  generators written from scratch by Claude (18 written; removed 2026-11-24: truchet weave "boring", noise bars "too cheesy", ink in water, flow field; 2026-11-25: reaction-diffusion), CC BY-NC-SA 4.0 (LICENSE-shaders.md), each with a .md doc (slider table, tips, a ⚠
  line for the heavy ones)
- contour map, moire, scanline terrain, lissajous, voronoi cells,
  kaleido flower, melting blobs, endless corridor, halftone sphere, starfield warp, hex pulse, dot matrix, shape morph
- added by the user 2026-11-25: synthwave, crt tv (15 in all); fractal terrain was added too and removed again the same
  day (based on Shadertoy "Md23RK", so not ours to license); brought in line with the pack: English labels (the NAMEs
  stay), pack-style .md docs (⚠ line: 16-20 ms at 1080p on the Iris Xe, the heaviest in the pack), the transparent
  background (crt tv: the cabinet; synthwave: plain sky and floor, the lines, grid, sun and solid ridges stay)
- pack CREDIT line (2026-11-25): "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net -
  CC BY-NC-SA 4.0" on every generator and effect
- one license for the whole pack (2026-11-25): every generator is CC BY-NC-SA 4.0 with the pack CREDIT line (crt tv and
  synthwave said CC0 before)
- 3D looks (asked by the user): the raymarched ones (melting blobs, endless corridor, halftone sphere) have a Style choice: shaded,
  wireframe (contour lines of the object-space coordinates, so the mesh sticks to the surface), outline (silhouette from the
  closest approach of missed rays, the inner rim where the surface turns away, crease edges where the normal jumps, depth
  steps), dots (a halftone screen of the lighting), x-ray (density glow collected through the volume); "fill under lines"
  mixes the shaded surface under the lines; scanline terrain: ridge lines / wire grid / solid / dots; starfield: dots /
  streaks / spinning wire forms (crosses, rings, squares, triangles, hexagons, sparkles, mixed; a form is at most half a
  grid cell, so 4 cells per layer are enough)
- English labels with MIN / MAX / DEFAULT, so all work as sliders and on MIDI; colors as color inputs; no audio
- dark backgrounds are always pure black by default (decided 2026-11-24); a light paper (halftone sphere) stays light
- wide ranges (2026-11-24, after playing): scanline terrain lines up to 24 px; voronoi down to 0.5 cells per screen height;
  starfield forms up to 16x with a star grid that grows with them (fewer, bigger forms; still 4 cells per layer) and density
  down to 0.01; kaleido flower: sharp edges (hard cut + color bands), crisp (finer octaves + edge / fold lines), dirt (grain,
  specks, scratches, faded print; all 0 by default); dot matrix: 14 patterns (+ lines, diagonals, scanner, spiral,
  checker, rain, sparkle, radar, squares); endless corridor: the user added crazy / kaleidoscope / rainbow sliders;
  halftone sphere: dots up to 120 px, lines up to 24 px, a "Paper" slider (0 = transparent): the shader composes paper,
  fill, shadow screen and ink with straight alpha, so only the print covers the layer below (mix type A)
- transparent background for all generators (2026-11-25): a "Background (0 = transparent)" slider (bgAlpha, default 1,
  appended as the last input so the other parameter numbers stay); straight alpha out of the shader; the 3D solids
  (melting blobs, endless corridor) know their shape (hit, lines, edges, fill; fog counts as background); the other
  ones key their light against the background color (alpha = how much brighter than the background, the color
  un-mixed), so glow and lines fade out softly; checked in the output over another generator (mix type A)
- videoping fix found on the way (2026-11-25): the live layers were rendered right after egui painted the ui, with its
  blending still on; a generator with transparent parts blended onto its own last frame (trails), and multi-pass
  shaders that keep data in alpha broke as live layers; now the GL state is reset before the
  live instances render (the prepare renders already came after a reset)
- videoping fix found on the way: several passes with the same TARGET declared the sampler twice (did not compile); now one
  sampler per buffer name (unit test); the ai's shader prompt no longer promises FLOAT / WIDTH / HEIGHT for passes (ignored:
  every buffer has the output size and 8 bits)
- tested: every shader compiled and rendered in a running videoping (self-test + MCP load_file / frame), every 3D style
  looked at; GPU time per frame at 1080p on Intel Iris Xe (whole frame, varies up to 2x with the GPU clock): flat 2D ones 1-4 ms,
  3D / lissajous / starfield up to 10-15 ms (optimized: fewer segments
  outside the figure, adaptive segment count; starfield: 4 cells instead of 9; terrain: stops at the sky; blobs: centers
  once per pixel)

## help button (2026-11-25)
- top bar, far right: "help" opens the online manual (https://videoping.net/manual.html) in the browser, at the
  chapter of the current tab (live: the screen, projection mapping, shader editor, midi mapping, ai tab); the hover
  shows the URL
- opened through egui's open_url (eframe feature "links", webbrowser crate: xdg-open on Linux)

## ◎ focus marker (2026-11-25)
- the hover shows what really follows the effect right now: per connected controller (its profile name) the
  parameter numbers its controls set through /sel/param/N (e.g. "Midi Fighter Twister: parameters 1–12"); no
  connected controller with such bindings: says so and how to map them (the "selected · …" parameters in the midi
  mapping tab); the manual explains ◎ with the effect rows too, not only in the Twister section

# future ideas (not planned)

Collected here (decided 2026-11-24) so nothing gets lost; when one is picked, it moves up into its own section.

## text input for ISF shaders
- ISF has no text input; videoping's addition, compatible with other ISF hosts: an image input with extra keys,
  e.g. `{ "NAME": "textImage", "TYPE": "image", "TEXT": "thank you\nberlin", "FONT_SIZE": 120, "ALIGN": "center" }`
  - other hosts see a normal image input (the shader still loads, you feed it an image)
  - videoping sees "TEXT": a text area instead of an image picker in the parameters; the text is rendered into a
    texture (white on transparent, rebuilt only when the text changes) and given to the shader, which does the look
    (scroll, wave, glitch, outline, split into letters)
- font: DejaVu Sans (already required); later an optional "FONT" key (path to a .ttf)
- saved with the instance like any parameter; an address in the control registry so MCP / the ai tab can set it
  ("put 'last song' on layer 1")
- no typing on midi: prepare several text instances in slots and launch them with pads
- basic text shaders come with the shader pack

## shader pack — effects built 2026-11-25
- folder `isf-pack/effects/`: 15 ISF effects written by Claude, CC BY-NC-SA 4.0, each with a .md doc (texts in
  a docs script: slider table, tips; strobe has a ⚠ photosensitivity line): color adjust, invert, posterize,
  threshold, blur, sharpen, edges, mirror, kaleido, transform, pixelate, trails, strobe, rgb split, vignette
- the user's rule: at the defaults every effect looks natural / as expected (color adjust, sharpen at defaults
  change little or nothing), but every slider goes into the extreme (blur 400 px, pixelate 600 px, contrast /
  saturation 8, edges strength 40, rgb split 500 px, kaleido 64 segments, transform zoom 0.02-30, sharpen 20)
- one job per effect, light on the GPU (blur: 4 separable passes, 25 taps each, two rounds so big radii have no
  gaps; premultiplied in the buffers so transparent parts don't bleed dark)
- the effects keep the input's alpha (and move it with the picture in mirror / kaleido / transform / blur)
- luma key: not an effect, the mix types already do it; chroma key: waits for the alpha decision (see below)
- strobe: rate in flashes per second, or 0 = follows the Phase slider: animate Phase with the bpm for a strobe
  on the beat (no beat uniform in ISF); freeze mode holds the picture in a persistent buffer
- trails: decay scaled by TIMEDELTA (same look at any frame rate); at least one 8-bit step down per frame so
  faint echoes don't stay for ever; Clear is an event (map it to a pad)
- 17 more from the user's own collection (2026-11-25), brought in line with the pack: composite video glitch,
  frame difference (was framediff), halftone chaos, lidar scan (was v2), color isolate, duotone, feedback, fold
  colors, gradient map, hue rotate, rgb gain (the mini-* ones without "mini-"), motion history, optical feedback
  (was v2), optical feedback 3 tubes, optical feedback keyer, rutt-etra (was rutt-etra-2d), tracking hud; English
  labels (NAMEs unchanged), descriptions without host notes / "v2" / German slider names, docs in pack form (host
  requirements, Resolume input limits and v1/v2 comparisons taken out; ⚠ GPU line for halftone chaos, lidar scan,
  composite video glitch, optical feedback keyer, tracking hud), CC BY-NC-SA 4.0 with the pack credit (were CC0);
  the shader code is unchanged; the originals in ivo-isf-effects/ stay as they are; then yolo detect and blob
  detection the same way (⚠ about 8 ms); 35 effects in all
- color reduce (from mini-color-reduce, 2026-11-25): rewritten to work like the ColorReduce ffgl plugin (the N most
  common colors of the frame on a 6x6x6 grid, nearest color per pixel, palette rotate); mini-color-reduce only cut
  each channel into fixed levels, and the user's colorreduce.fs port kept counts and ranks in FLOAT buffers, which
  videoping makes 8 bit (ranks clipped at 1, counts rounded); now 8-bit safe: counts as 16 bits in two channels,
  ranks as whole bytes; the count split into 216 x 72 row counts plus a sum pass (one pass over all 9216 samples
  in 216 threads cost ~9 ms, now ~1-2 ms); kept: dithering and pixel size from the mini version
- open question (asked 2026-11-25): the effect mix writes alpha 1 and uses the effect's alpha as coverage over the
  unprocessed input, so an effect can't make anything transparent (transparent options in transform, pixelate,
  vignette, strobe show the unprocessed input there) and any effect makes a transparent generator opaque


## finder
- "in use" marker (files in the grid / live), clip thumbnails, recently used (see "finder")

## videopong.net
- login for the full-quality download (now only the 480x360 preview mp4)
- upload a recording to videopong.net (idea 2026-11-25): needs the login (same mechanism as the full-quality
  download); after REC: "share on videopong.net" on the recording -> title, tags, Creative Commons license (choice),
  upload with progress; the clip page link afterwards
- maximum version: upload the ISF shaders used in the recording along with the clip (videopong.net can attach ISF
  shaders to a clip): the generators / effects that were on the output while recording, each with its .md doc;
  only own shaders or ones whose license allows it (ask before uploading)
- MCP / ai tools to search and load videopong clips

## midi
- Twister bank 2 (see "midi")
- import / export of profiles (now: the plain json files in the folder)
- midi learn right in the ui (idea 2026-11-24): now learn only works in the "midi mapping" tab (pick an address, learn,
  move a control); instead: a "learn" mode (button in the top bar or a key) where every mappable control on screen
  (slider, button, effect on/off, mix, slot) gets a thin outline; click one, move a knob / press a pad -> bound to the
  selected device's profile; controls that already have a binding show it (small midi badge, hover = which control);
  right click on a control -> "learn" / "forget" as a shortcut without the mode; Esc or the button leaves the mode
  - the address of each ui control comes from the control registry (the same addresses as the midi tab, MCP and the ai)
  - learn picks the mode like now (button -> trigger / toggle, fader / knob -> absolute, encoder -> relative)

## undo (idea 2026-11-24)
- undo only undoes what you are looking at (decided 2026-11-24): the history belongs to the tab that is open, never a
  change in another tab that would then happen "in the background"
- live tab: one history for everything in it (decided 2026-11-24): prepare grid (slots loaded / cleared / moved),
  prepare detail (parameters, ranges, animation settings, effects added / removed / reordered / moved), mixer, output
  effects, finder favorites, "new"; also instances that are live in a layer (the output changes with the undo, that is
  ok: it is all on screen)
- projection mapping tab, midi mapping tab: each its own history (slices / masks of the preset; bindings of the profile)
- shader editor: its own text undo (exists); ai tab: its own undo per change (exists)
- Ctrl+Z / Ctrl+Shift+Z (and buttons) act on the open tab; a whole slider drag = one step; a limited history per tab
  (e.g. 100 steps), lost on restart
- a status line "undo: <what>" before / when undoing, so it is clear what changes
- midi is undoable too (decided 2026-11-24): a knob / fader turn or a button press makes an undo step in the tab
  its target belongs to (mostly the live tab), exactly like the same change with the mouse; a knob turned
  continuously = one step (the step ends after a short pause, like a slider drag ends on release)
- not undone (mouse or midi): one-shot actions that cannot be taken back on the output: REC start / stop, bpm tap

## control
- OSC (not for now, decided 2026-11-22; the control registry would carry it)

## security
- NDI library folder out of the session into the general config (open point of the security check 2026-11-24)

## distribution
- GitHub Actions: build, unit tests, `cargo audit` on every push (also proves a clean Ubuntu build)
- ready-built program as a GitHub release: AppImage or .deb (no Rust / GStreamer dev setup needed)

## patch-bay for videosignals

removed from the app and from the plan (2026-11-21), kept here only as an idea.

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
