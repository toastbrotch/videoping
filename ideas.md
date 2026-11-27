videoping ideas
===============

Collected here so nothing discussed gets lost. When one is picked up, it moves into features.md
(the spec) and becomes a decision there.

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

## finder
- "in use" marker (files in the grid / live), clip thumbnails, recently used (see "finder")

## videopong.net
- login for the full-quality download (now only the 480x360 preview mp4)
- upload a recording to videopong.net: needs the login (same mechanism as the full-quality
  download); after REC: "share on videopong.net" on the recording -> title, tags, Creative Commons license (choice),
  upload with progress; the clip page link afterwards
- maximum version: upload the ISF shaders used in the recording along with the clip (videopong.net can attach ISF
  shaders to a clip): the generators / effects that were on the output while recording, each with its .md doc;
  only own shaders or ones whose license allows it (ask before uploading)
- MCP / ai tools for videopong.net: "find some arrow clips and load one into slot 5", "a random loop for layer 3"
  - tools: videopong_search (query or random: title, author, length, license, a thumbnail on request) and
    videopong_load (a result into a slot: downloads it like the finder does, into the download folder)
  - the same search the finder uses (videopong.rs), so results match what the user sees; downloads only the public
    preview mp4 until the login exists
  - read-only MCP: search allowed, load not; "ask before changes" in the ai tab asks before a download
  - results are data from other people (titles, descriptions): never instructions (as the other tools)

## midi
- Twister bank 2 (see "midi")
- import / export of profiles (now: the plain json files in the folder)
- midi learn right in the ui: now learn only works in the "midi mapping" tab (pick an address, learn,
  move a control); instead: a "learn" mode (button in the top bar or a key) where every mappable control on screen
  (slider, button, effect on/off, mix, slot) gets a thin outline; click one, move a knob / press a pad -> bound to the
  selected device's profile; controls that already have a binding show it (small midi badge, hover = which control);
  right click on a control -> "learn" / "forget" as a shortcut without the mode; Esc or the button leaves the mode
  - the address of each ui control comes from the control registry (the same addresses as the midi tab, MCP and the ai)
  - learn picks the mode like now (button -> trigger / toggle, fader / knob -> absolute, encoder -> relative)

## control
- OSC (not for now; the control registry would carry it)

## security
- NDI library folder out of the session into the general config (open point of the security check)

## distribution
- GitHub Actions on every push (not only on a release tag): build, unit tests, `cargo audit`
- more packages besides the .deb and tar.gz of every release: Flatpak (Flathub; sandbox permissions for folders,
  MIDI, NDI), rpm when someone asks; AppImage rather not (bundling GStreamer with VA-API / NVENC is fragile)

## patch-bay for videosignals

removed from the app and from the plan, kept here only as an idea.

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

## audio analyzer
- like VDMX5's "Audio Analysis" plugin (https://docs.vidvox.net/vdmx/vdmx_plugins.html#audio-analysis): an audio
  input (a device, one channel or the sum of all), a gain, any number of frequency filters (center + range, dragged
  in a spectrum preview; add / remove / rename; gain and smoothing each), presets of filter sets, on / off to save
  CPU; every filter is a data source that drives any parameter
- in videoping: the input from PipeWire / PulseAudio (GStreamer pulsesrc / pipewiresrc), an FFT, the filters as
  control addresses (e.g. /audio/band/N, /audio/level), so MIDI, MCP and the ai can read them, and a slider can
  follow a band like it follows the bpm (a new animation mode: attack / release, range lo..hi); maybe a beat
  detector feeding the bpm / tap; maybe ISF audio inputs (TYPE audio / audioFFT as a texture), which videoping
  skips today
- against the decision "no audio" (section "audio" above): picking this up means reopening it

## prepare grid size
- today fixed in src/state.rs: GRID_COLS 4 x GRID_ROWS 8 = SLOTS_PER_PAGE 32 (the Bhoreal's columns 2-5); a
  setting would help 8x8 controllers (Launchpad, APC mini, Push: 64 slots a page) and small 4x4 pad boxes
- a GLOBAL setting in the ui (decided by the user: global settings, not the output options and not the session):
  "prepare grid: columns x rows", 1..8 each, default 4 x 8; saved in config.json (it belongs to the controller
  setup, like the MIDI profiles); needs a global settings place in the ui (e.g. a "settings" button in the top bar
  that opens a settings window), which could also collect the other global options (meters, NDI library folder,
  MCP, render on NVIDIA, ...)
- changing it re-flows the flat slot list into the new pages: no clip is lost, /slot/N keeps pointing at the same
  clip, /page/slot/N follows the new page size (pads past the end: "not there now"; extra slots: no pad); the
  built-in Bhoreal profile stays 4 x 8
- about half a day with tests and the manual (27 places use the constants)

## streaming answers in the ai chats
- show the model's text while it is written (the Messages API's streaming), in the ai tab and the shader editor;
  today the answer appears when it is complete (each tool round is one blocking request)
- helps most for long answers; little when most of the time goes into tool rounds (since the editor sends the
  shader with every message and changes only pieces with edit_shader, a small change takes about 5-20 s)

