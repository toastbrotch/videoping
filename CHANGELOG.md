# Changelog

What changed in each release. The release workflow puts the section of a version into its GitHub
release (`.github/workflows/release.yml`); write it before tagging.

## 0.0.2

- **Undo / redo** (Ctrl+Z, Ctrl+Shift+Z, ↶ ↷ in the top bar): one history per tab; changes from a
  MIDI controller are undone like mouse changes; clips keep playing.
- **Effects make transparency:** an effect's alpha becomes the layer's alpha, so "transparent" in
  transform / mirror / kaleido, transparent vignette edges and pixelate gaps and the strobe's
  transparent flash now work; transparent generators stay transparent under effects.
- **New effect: chroma key**, plus 20 more effects in the pack (36 in all) and two new generators
  (synthwave, CRT TV).
- **Shader editor ai** understands the shader better and answers faster: it gets the shader, its
  documentation and your slider values, and changes only the parts that need changing.
- The ui symbol font is built in; the output screen list shows exactly where fullscreen goes; the
  choices of generators and effects are buttons; a big learn button in the midi mapping.
- Fixes: deleting a MIDI profile no longer crashes (#1); folding an effect no longer makes the
  column creep.

## 0.0.1

- First public alpha: 3 layers with a prepare lane, ISF generators and effects (15 generators in the
  shader pack), clips, NDI and videopong.net, MIDI controllers, projection mapping, recording, NDI
  output, an ai in the booth (ai tab, MCP), the shader editor.
