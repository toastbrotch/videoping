# AGENTS.md: notes for coding agents

videoping is live VJ software for Linux, written in Rust (egui / eframe, OpenGL 3.3 via glow,
GStreamer, ALSA MIDI). Read this first, then the files it points to.

## Read before changing things

- `dev.md`: setup, build, tests and self-tests, architecture (modules), ISF support, gotchas, release.
- `features.md`: the spec. Every design decision, with the reasons and measurements. Check it before
  changing behavior; a change that contradicts it needs the maintainer's OK.
- `ideas.md`: ideas that are not built yet.
- `website/manual.html`: the end-user manual (HTML, written by hand).

## Commands

```sh
cargo build --release
cargo test                                   # unit tests (no GPU needed)
VIDEOPING_SELFTEST=/tmp/st target/release/videoping   # self-test in a real window, see dev.md
scripts/render-shaders.py <outdir> <shader.fs>...     # render shaders, GPU time, compile errors
scripts/mcp-test.sh <port> <outdir>                    # MCP checks against a running self-test
python3 scripts/install-html.py                        # install.md -> website/install.html
scripts/screenshots.py                                 # all website / manual screenshots (website/img)
```

Release: set `version` in Cargo.toml, tag `v<version>`, push the tag; GitHub Actions builds the .deb
and tar.gz into a draft release (dev.md, "release").

## Rules

- **Docs with every change:** a design decision goes into `features.md`, a user-visible change into
  `website/manual.html` (and README / website when they mention it), a parked idea into `ideas.md`;
  when the ui looks different, renew the screenshots (`scripts/screenshots.py`).
- **Commit only when the maintainer asks.** Never commit `restart` or private shader folders; only
  `isf-pack/` holds shaders in the repository.
- **Never touch the user's data** in tests or self-tests: the autosave
  (`~/.local/share/videoping/`), the config and MIDI profiles (`~/.config/videoping/`), presets.
  The self-test keeps everything in its own folder.
- **Never print `~/.config/videoping/config.json`:** it holds API keys.
- Linux only (no Windows / macOS code paths). No audio (for now). In the ui, colors are reserved for
  the layer color code (blue, green, orange); the plum accent marks selected / active things.
- Keep the user's existing ISF files unchanged; the shader pack is ours.

## Shader pack (`isf-pack/generators`, `isf-pack/effects`)

- ISF 2, compiled as GLSL 330. Every `TARGET` buffer is persistent, output size, **8 bits per
  channel** (FLOAT / WIDTH / HEIGHT are ignored); several passes may share a target.
- English labels; every input has LABEL, MIN, MAX, DEFAULT. The defaults look natural, the ranges
  reach far into the extreme. New inputs go at the end (renaming or reordering breaks saved settings
  and MIDI mappings).
- Straight (not premultiplied) alpha. Generators with a background have a
  "Background (0 = transparent)" slider (`bgAlpha`); dark backgrounds are pure black. An effect's alpha
  becomes the layer's alpha: effects that only change colors keep the input's alpha.
- `"CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0"`.
- Every shader has a `.md` doc next to it: title, "`x.fs` is an ISF generator / effect from the
  videoping shader pack. …", a ⚠ line when it is heavy, a slider table (label, range, default, what
  it does) that matches the header, tips, the license line.
- Test them in a running videoping (`scripts/render-shaders.py`), not only by compiling.

## Traps we ran into

- egui leaves GL blending on after painting the ui: reset the GL state before rendering.
- Self-test: its mouse script clears the session in the first ~8 s (wait before testing); slot 9 is
  live, use slots 20 and up; finder folders are the pack and the self-test folder.
- Background processes started in one shell call end with that call: start, test and stop in one go,
  and stop a test instance by its process ID, never by name (the user may have videoping running).
- The ai features call paid APIs: run real requests only when the maintainer agrees.
