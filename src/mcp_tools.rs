//! The MCP tools, answered by the app (child module of app: it reaches the session, the renderer,
//! the finder). Transport and tool definitions: mcp.rs.

use super::*;
use crate::control::Value as CValue;
use crate::mcp::{Content, ToolResult};
use serde_json::Value as J;

impl App {
    /// Once per frame: start / stop the server as the general config says, answer tool calls.
    pub(super) fn mcp_poll(&mut self, gl: &glow::Context) {
        let want = self.cfg.mcp.then_some(self.cfg.mcp_port);
        let running = self.mcp.as_ref().map(|s| s.port);
        if want != running {
            self.mcp = None;
            if let Some(port) = want {
                // a failed port is not retried every frame
                if self.mcp_failed != Some(port) {
                    match crate::mcp::Server::start(port) {
                        Ok(s) => {
                            self.mcp = Some(s);
                            self.mcp_failed = None;
                        }
                        Err(e) => {
                            self.set_status(e);
                            self.mcp_failed = Some(port);
                        }
                    }
                }
            }
        }
        let calls = self.mcp.as_ref().map(|s| s.calls()).unwrap_or_default();
        for c in calls {
            let r = self.mcp_call(gl, &c.name, &c.args, false);
            c.answer(r);
        }
    }

    /// "read only" is for MCP clients from outside; the ai tab has "ask before changes" instead
    fn mcp_changes_refused(&self, from_ai: bool) -> Option<ToolResult> {
        (self.cfg.mcp_read_only && !from_ai).then(|| ToolResult::error("videoping is set to read only for MCP (general config): changes are refused"))
    }

    /// One tool call (from an MCP client, or from the ai tab: from_ai).
    pub(super) fn mcp_call(&mut self, gl: &glow::Context, name: &str, a: &J, from_ai: bool) -> ToolResult {
        let s = |k: &str| a.get(k).and_then(|v| v.as_str()).map(|v| v.to_string());
        let who = if from_ai { "ai" } else { "mcp" };
        match name {
            "state" => ToolResult::text(self.mcp_state()),
            "list_addresses" => {
                let f = s("filter").unwrap_or_default().to_lowercase();
                let lines: Vec<String> = self
                    .control_list()
                    .into_iter()
                    .filter(|d| f.is_empty() || d.addr.to_lowercase().contains(&f) || d.label.to_lowercase().contains(&f))
                    .map(|d| {
                        let kind = match &d.kind {
                            crate::control::Kind::Trigger => "trigger".to_string(),
                            crate::control::Kind::Toggle => "on/off".to_string(),
                            crate::control::Kind::Range { min, max, integer } => format!("range {min}..{max}{}", if *integer { " (whole)" } else { "" }),
                            crate::control::Kind::Choice(c) => format!("choice {}", c.join("|")),
                        };
                        format!("{} | {kind}{} | {}", d.addr, if d.writable { "" } else { " (read only)" }, d.label)
                    })
                    .collect();
                ToolResult::text(if lines.is_empty() { "nothing matches".into() } else { lines.join("\n") })
            }
            "get" => {
                let addrs: Vec<String> = a.get("addresses").and_then(|v| v.as_array()).map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
                let lines: Vec<String> = addrs
                    .iter()
                    .map(|ad| match self.control_get(ad) {
                        Some(CValue::Bool(b)) => format!("{ad} = {b}"),
                        Some(CValue::Num(n)) => format!("{ad} = {n}"),
                        Some(CValue::Text(t)) => format!("{ad} = {t}"),
                        None => format!("{ad}: no value (unknown, a trigger, or not there now)"),
                    })
                    .collect();
                ToolResult::text(lines.join("\n"))
            }
            "set" => {
                if let Some(r) = self.mcp_changes_refused(from_ai) {
                    return r;
                }
                let Some(addr) = s("address") else { return ToolResult::error("address missing") };
                let v = match a.get("value") {
                    None | Some(J::Null) => CValue::Bool(true),
                    Some(J::Bool(b)) => CValue::Bool(*b),
                    Some(J::Number(n)) => CValue::Num(n.as_f64().unwrap_or(0.0) as f32),
                    Some(J::String(t)) => CValue::Text(t.clone()),
                    Some(other) => return ToolResult::error(format!("unsupported value {other}")),
                };
                match self.control_set(&addr, &v) {
                    Ok(()) => {
                        let now = self.control_get(&addr);
                        self.set_status(format!("{who}: {addr} = {}", value_text(&v)));
                        ToolResult::text(match now {
                            Some(n) => format!("{addr} = {}", value_text(&n)),
                            None => format!("{addr}: done"),
                        })
                    }
                    Err(e) => ToolResult::error(e),
                }
            }
            "list_files" => {
                let kind = s("kind").unwrap_or_default().to_lowercase();
                let f = s("filter").unwrap_or_default().to_lowercase();
                let mut lines: Vec<String> = self
                    .finder
                    .iter()
                    .filter(|e| kind.is_empty() || kind_name(&e.kind) == kind)
                    .filter(|e| f.is_empty() || e.name.to_lowercase().contains(&f))
                    .map(|e| format!("{} | {}", kind_name(&e.kind), e.path.display()))
                    .collect();
                if kind.is_empty() || kind == "ndi" {
                    for src in self.ndi_finder.as_ref().map(|n| n.sources()).unwrap_or_default() {
                        if f.is_empty() || src.to_lowercase().contains(&f) {
                            lines.push(format!("ndi | {src}"));
                        }
                    }
                }
                ToolResult::text(if lines.is_empty() { "no files (add folders in the finder)".into() } else { lines.join("\n") })
            }
            "load_file" => {
                if let Some(r) = self.mcp_changes_refused(from_ai) {
                    return r;
                }
                let Some(p) = s("path") else { return ToolResult::error("path missing") };
                // stored like the finder does it: absolute (relative paths would break from another folder)
                let path = std::fs::canonicalize(&p).unwrap_or_else(|_| PathBuf::from(&p));
                // files only from the finder folders (not anything on the disk); other names are NDI sources
                if path.exists() && !self.in_finder(&path) {
                    return ToolResult::error(format!("{p}: not in the finder folders (see list_files)"));
                }
                let kind = if path.exists() { isf::classify(&path) } else { Some(FileKind::Ndi) };
                let Some(kind) = kind else { return ToolResult::error(format!("{p}: not a clip, shader or NDI source")) };
                if kind == FileKind::Effect {
                    return ToolResult::error(format!("{p} is an effect: use add_effect"));
                }
                let slot = a.get("slot").and_then(|v| v.as_u64()).and_then(|n| (n as usize).checked_sub(1));
                if let Some(si) = slot {
                    // a slot on a page that does not exist yet: add pages
                    while self.s.slots.len() <= si {
                        self.s.slots.extend(std::iter::repeat_n(None, SLOTS_PER_PAGE));
                    }
                    if self.s.slot_is_live(si) {
                        return ToolResult::error(format!("slot {} plays in a layer: not replaced", si + 1));
                    }
                    self.s.slots[si] = None;
                }
                self.add_file_to_slot(&path, &kind, slot);
                match self.s.selected_instance.and_then(|id| self.s.slot_of(id)) {
                    Some(si) => {
                        self.set_status(format!("{who}: loaded {} into slot {}", isf::display_name(&path), si + 1));
                        ToolResult::text(format!("slot {}", si + 1))
                    }
                    None => ToolResult::error("could not load it"),
                }
            }
            "add_effect" => {
                if let Some(r) = self.mcp_changes_refused(from_ai) {
                    return r;
                }
                let Some(p) = s("path") else { return ToolResult::error("path missing") };
                // stored like the finder does it: absolute (relative paths would break from another folder)
                let path = std::fs::canonicalize(&p).unwrap_or_else(|_| PathBuf::from(&p));
                if !self.in_finder(&path) {
                    return ToolResult::error(format!("{p}: not in the finder folders (see list_files)"));
                }
                if isf::classify(&path) != Some(FileKind::Effect) {
                    return ToolResult::error(format!("{p}: not an ISF effect"));
                }
                let owner = if s("target").as_deref() == Some("output") {
                    Owner::Output
                } else if let Some(n) = a.get("slot").and_then(|v| v.as_u64()) {
                    match self.s.slots.get((n as usize).wrapping_sub(1)).and_then(|x| x.as_ref()) {
                        Some(i) => Owner::Instance(i.id),
                        None => return ToolResult::error(format!("slot {n} is empty")),
                    }
                } else {
                    match self.s.selected_instance {
                        Some(id) => Owner::Instance(id),
                        None => return ToolResult::error("no slot given and nothing selected"),
                    }
                };
                let before = self.effects_mut(owner).map(|v| v.len()).unwrap_or(0);
                self.add_effect(owner, &path);
                let after = self.effects_mut(owner).map(|v| v.len()).unwrap_or(0);
                if after > before {
                    let where_ = match owner {
                        Owner::Output => format!("/out/fx/{after}"),
                        Owner::Instance(id) => format!("/slot/{}/fx/{after}", self.s.slot_of(id).map(|x| x + 1).unwrap_or(0)),
                    };
                    self.set_status(format!("{who}: added {} at {where_}", isf::display_name(&path)));
                    ToolResult::text(format!("added as {where_}"))
                } else {
                    ToolResult::error(format!("not added (max {MAX_EFFECTS} effects per instance and on the output)"))
                }
            }
            "frame" => {
                let src = s("source").unwrap_or_else(|| "output".into());
                let width = a.get("width").and_then(|v| v.as_u64()).unwrap_or(640).clamp(16, 1920) as i32;
                let Some(r) = self.r.as_mut() else { return ToolResult::error("nothing rendered yet") };
                let (w, h) = r.size();
                let tex = if src == "output" {
                    Some((r.out.tex, w, h))
                } else if src == "preview" {
                    self.s.selected_instance.and_then(|id| r.output_of(id)).map(|t| (t, w, h))
                } else if src == "editor" {
                    r.output_of(crate::render::EDITOR_ID).map(|t| (t, w, h))
                } else if let Some(n) = src.strip_prefix("slot").and_then(|x| x.trim().parse::<usize>().ok()) {
                    let id = self.s.slots.get(n.wrapping_sub(1)).and_then(|x| x.as_ref()).map(|i| i.id);
                    id.and_then(|id| r.output_of(id).map(|t| (t, w, h)).or_else(|| r.thumb(id).map(|t| (t, crate::render::THUMB_W, crate::render::THUMB_H))))
                } else {
                    return ToolResult::error("source: output, preview, editor or \"slot N\"");
                };
                let Some((tex, tw, th)) = tex else { return ToolResult::error(format!("{src}: nothing rendered there (select it or load it into a layer)")) };
                let (pw, ph, px) = r.snapshot(gl, tex, tw, th, width);
                ToolResult { content: vec![Content::Png(crate::mcp::png(pw as u32, ph as u32, &px))], is_error: false }
            }
            "shader_errors" => {
                let errs = self.r.as_ref().map(|r| r.shader_errors()).unwrap_or_default();
                ToolResult::text(if errs.is_empty() {
                    "no shader errors".to_string()
                } else {
                    errs.iter().map(|(p, e)| format!("{}:\n{e}", p.display())).collect::<Vec<_>>().join("\n\n")
                })
            }
            "shader_info" => {
                let Some(p) = s("path") else { return ToolResult::error("path missing") };
                // only shaders of the finder: no reading of other files (their text would go to the ai)
                let path = std::fs::canonicalize(&p).unwrap_or_else(|_| PathBuf::from(&p));
                if !self.in_finder(&path) {
                    return ToolResult::error(format!("{p}: not a shader in the finder folders (see list_files)"));
                }
                let p = path.display().to_string();
                match isf::parse_info(Path::new(&p)) {
                    Ok((info, _)) => {
                        let mut lines = vec![format!("{} ({})", info.name, if info.is_effect { "effect" } else { "generator" })];
                        let shown = info.inputs.iter().filter(|i| !matches!(i.kind, InputKind::Image | InputKind::Audio));
                        for (k, i) in shown.enumerate() {
                            lines.push(format!("param/{} | {} | {} | {}", k + 1, i.name, i.label, input_text(&i.kind)));
                        }
                        let images: Vec<&str> = info.inputs.iter().filter(|i| matches!(i.kind, InputKind::Image)).map(|i| i.name.as_str()).collect();
                        if !images.is_empty() {
                            lines.push(format!("image inputs: {}", images.join(", ")));
                        }
                        // the documentation next to the shader (.md), when there is one
                        if let Ok(md) = std::fs::read_to_string(crate::shader_doc::md_path(Path::new(&p))) {
                            lines.push(format!("\ndocumentation ({}):\n{md}", crate::shader_doc::md_path(Path::new(&p)).display()));
                        }
                        ToolResult::text(lines.join("\n"))
                    }
                    Err(e) => ToolResult::error(e),
                }
            }
            _ => ToolResult::error(format!("unknown tool {name}")),
        }
    }

    /// A file the finder lists (path as given, or both resolved).
    fn in_finder(&self, path: &Path) -> bool {
        self.finder.iter().any(|e| e.path == path || std::fs::canonicalize(&e.path).is_ok_and(|c| c == path))
    }

    fn mcp_state(&self) -> String {
        let mut o = vec![];
        let fps = self.s.output.fps;
        o.push(format!(
            "output {}x{} @ {} fps (measured {}), GPU {:.1} ms per frame, {}{}",
            self.s.output.width,
            self.s.output.height,
            fps,
            self.frame_times.len(),
            self.r.as_ref().map(|r| r.meter.total_ms).unwrap_or(0.0),
            if self.r.as_ref().is_some_and(|r| r.rec.is_some()) { "RECORDING, " } else { "" },
            if self.cfg.mcp_read_only { "MCP read only" } else { "MCP may change things" }
        ));
        o.push(format!(
            "master: blackout {}, freeze {}, brightness {:.0} %; bpm {}; prepare page {} of {}",
            self.s.master.blackout,
            self.s.master.freeze,
            self.s.master.brightness * 100.0,
            self.s.bpm,
            self.s.page + 1,
            self.s.pages()
        ));
        o.push("mixer (layer 1 on top):".into());
        for (li, l) in self.s.layers.iter().enumerate() {
            let what = l
                .instance
                .and_then(|id| self.s.find(id).map(|i| (i, self.s.slot_of(id))))
                .map(|(i, sl)| format!("{} ({}, slot {})", i.name, source_text(&i.source), sl.map(|x| x + 1).unwrap_or(0)))
                .unwrap_or("empty".into());
            o.push(format!(
                "  L{}{}: {} | {} | mix {} {:.0} % | rgb {}{}{}",
                li + 1,
                if self.s.selected_layer == li { " (selected)" } else { "" },
                what,
                if l.active { "on" } else { "off" },
                l.mix.mix_type.short(),
                l.mix.amount * 100.0,
                if l.mix.rgb[0] { "R" } else { "-" },
                if l.mix.rgb[1] { "G" } else { "-" },
                if l.mix.rgb[2] { "B" } else { "-" },
            ));
        }
        o.push("slots:".into());
        for (si, slot) in self.s.slots.iter().enumerate() {
            let Some(i) = slot else { continue };
            let live = self.s.live_layer(i.id).map(|l| format!(", live in L{}", l + 1)).unwrap_or_default();
            let sel = if self.s.selected_instance == Some(i.id) { ", selected" } else { "" };
            o.push(format!("  slot {}: {} ({}{live}{sel})", si + 1, i.name, source_text(&i.source)));
            for (k, fx) in i.effects.iter().enumerate() {
                o.push(format!("    fx {}: {} ({}, {} {:.0} %)", k + 1, isf::display_name(&fx.path), if fx.active { "on" } else { "off" }, fx.mix.mix_type.short(), fx.mix.amount * 100.0));
            }
        }
        if self.s.output_effects.is_empty() {
            o.push("output effects: none".into());
        } else {
            o.push("output effects:".into());
            for (k, fx) in self.s.output_effects.iter().enumerate() {
                o.push(format!("  fx {}: {} ({}, {} {:.0} %)", k + 1, isf::display_name(&fx.path), if fx.active { "on" } else { "off" }, fx.mix.mix_type.short(), fx.mix.amount * 100.0));
            }
        }
        o.join("\n")
    }
}

pub(super) fn value_text(v: &CValue) -> String {
    match v {
        CValue::Bool(b) => b.to_string(),
        CValue::Num(n) => format!("{n}"),
        CValue::Text(t) => t.clone(),
    }
}

fn kind_name(k: &FileKind) -> &'static str {
    match k {
        FileKind::Clip => "clip",
        FileKind::Generator => "generator",
        FileKind::Effect => "effect",
        FileKind::Ndi => "ndi",
        FileKind::Videopong => "videopong",
    }
}

fn source_text(s: &SourceKind) -> &'static str {
    match s {
        SourceKind::Clip { .. } => "clip",
        SourceKind::Generator { .. } => "generator",
        SourceKind::Ndi { .. } => "NDI",
    }
}

fn input_text(k: &InputKind) -> String {
    match k {
        InputKind::Float { min, max, default } => format!("float {min}..{max}, default {default}"),
        InputKind::Long { min, max, default, labels } => {
            let l: Vec<String> = labels.iter().map(|(v, n)| format!("{v}={n}")).collect();
            format!("long {min}..{max}, default {default}{}", if l.is_empty() { String::new() } else { format!(" ({})", l.join(", ")) })
        }
        InputKind::Bool { default } => format!("bool, default {default}"),
        InputKind::Event => "event (not controllable yet)".into(),
        InputKind::Color { default } => format!("color, default {default:?} (not controllable yet)"),
        InputKind::Point2D { min, max, default } => format!("point2D {min:?}..{max:?}, default {default:?} (/x and /y)"),
        InputKind::Image | InputKind::Audio => "image".into(),
    }
}
