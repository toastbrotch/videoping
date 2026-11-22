//! The "ai" tab: chat with an LLM that controls videoping through the MCP tools
//! (features.md "ai tab"). Child module of app: the tools run in the app (mcp_tools.rs).
//!
//! - the request runs in a worker thread (ai.rs); the answer is picked up at a frame
//! - tool calls run at a frame, one after the other; every call is shown in the chat with its
//!   result; `set` can be undone; with "ask before changes" the changing tools wait for apply
//! - the loop runs on when another tab is open; at most MAX_ROUNDS requests per message
//! - the conversation is not saved (a new start = a new conversation)

use super::*;
use crate::ai::{self, Provider, Reply, Settings, ToolCall, ToolOut};
use crate::control::Value as CValue;
use crate::mcp::Content;
use serde_json::Value as J;
use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

const MAX_ROUNDS: u32 = 25;

/// Added to the MCP instructions for the chat inside videoping.
const CHAT: &str = "You are the assistant inside videoping's \"ai\" tab. The user is a VJ, maybe in the middle of a live show: \
answer short, act directly with the tools instead of explaining how, and say in one line what you changed. Every change \
shows on the output at once. Use `frame` to look at the output when what you see matters. Do not change things the user \
did not ask for.";

/// The tools that change something (they wait for "apply" with "ask before changes").
fn changes(name: &str) -> bool {
    matches!(name, "set" | "load_file" | "add_effect")
}

pub(super) struct ToolItem {
    name: String,
    args: String,
    result: String,
    error: bool,
    /// `set`: the address and its value before (undo)
    undo: Option<(String, CValue)>,
    undone: bool,
    image: Option<egui::TextureHandle>,
    /// waiting for apply / skip
    asking: bool,
}

pub(super) enum Item {
    User(String),
    Text(String),
    Tool(ToolItem),
    Note(String),
    Error(String),
}

#[derive(Default)]
pub(super) struct AiChat {
    /// the conversation in the provider's format
    msgs: Vec<J>,
    provider: Option<Provider>,
    view: Vec<Item>,
    input: String,
    waiting: Option<(Receiver<Result<Reply, String>>, Instant)>,
    /// tool calls still to run (in order)
    pending: VecDeque<ToolCall>,
    /// the chat item of the call that waits for apply / skip
    asking: Option<usize>,
    decision: Option<bool>,
    apply_all: bool,
    outs: Vec<ToolOut>,
    rounds: u32,
    /// tokens of the whole chat (input, output)
    usage: (u64, u64),
    /// for the model with the next message (undo)
    notes: Vec<String>,
    show_key: bool,
}

impl AiChat {
    fn busy(&self) -> bool {
        self.waiting.is_some() || !self.pending.is_empty()
    }
}

fn env_key(p: Provider) -> &'static str {
    match p {
        Provider::Anthropic => "ANTHROPIC_API_KEY",
        Provider::OpenAi => "OPENAI_API_KEY",
    }
}

/// Short text of a tool call: `set /layer/1/mix 0.5`.
fn call_text(c: &ToolCall) -> String {
    let i = &c.input;
    let s = |k: &str| i.get(k).map(|v| v.as_str().map(String::from).unwrap_or_else(|| v.to_string()));
    let t = match c.name.as_str() {
        "set" => format!("{} {}", s("address").unwrap_or_default(), s("value").unwrap_or_default()),
        "get" => i["addresses"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(" ")).unwrap_or_default(),
        _ => {
            let t = i.to_string();
            if t == "{}" { String::new() } else { t }
        }
    };
    t.chars().take(200).collect()
}

fn decode_png(bytes: &[u8]) -> Option<egui::ColorImage> {
    let dec = ::png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = dec.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    (info.color_type == ::png::ColorType::Rgba).then(|| egui::ColorImage::from_rgba_unmultiplied([info.width as usize, info.height as usize], &buf[..info.buffer_size()]))
}

impl App {
    fn ai_settings(&self) -> Settings {
        let c = &self.cfg;
        let provider = Provider::from_str(&c.ai_provider);
        let mut st = match provider {
            Provider::Anthropic => Settings { provider, model: c.ai_model_anthropic.trim().to_string(), key: c.ai_key_anthropic.clone(), url: c.ai_url_anthropic.clone(), effort: c.ai_effort.clone() },
            Provider::OpenAi => Settings { provider, model: c.ai_model_openai.trim().to_string(), key: c.ai_key_openai.clone(), url: c.ai_url_openai.clone(), effort: c.ai_effort.clone() },
        };
        // empty key field: the usual environment variable
        if st.key.trim().is_empty() {
            st.key = std::env::var(env_key(provider)).unwrap_or_default();
        }
        st
    }

    fn ai_request(&mut self) {
        let s = self.ai_settings();
        let system = format!("{}\n\n{CHAT}", crate::mcp::INSTRUCTIONS);
        self.ai.waiting = Some((ai::spawn(s, system, self.ai.msgs.clone()), Instant::now()));
    }

    fn ai_send(&mut self) {
        let text = self.ai.input.trim().to_string();
        if text.is_empty() || self.ai.busy() {
            return;
        }
        let s = self.ai_settings();
        if s.model.is_empty() {
            self.ai.view.push(Item::Error("no model set (above)".into()));
            return;
        }
        if s.provider == Provider::Anthropic && s.key.trim().is_empty() {
            self.ai.view.push(Item::Error("no Anthropic API key: paste it into \"key\" above (\"get a key\" opens the Anthropic console), or set ANTHROPIC_API_KEY".into()));
            return;
        }
        // another provider: its own message format, a new conversation
        if self.ai.provider.is_some_and(|p| p != s.provider) && !self.ai.msgs.is_empty() {
            self.ai.msgs.clear();
            self.ai.view.push(Item::Note("other provider: new conversation".into()));
        }
        self.ai.provider = Some(s.provider);
        let mut full = text.clone();
        if !self.ai.notes.is_empty() {
            full = format!("[{}]\n{full}", self.ai.notes.join("; "));
            self.ai.notes.clear();
        }
        ai::push_user_text(s.provider, &mut self.ai.msgs, &full);
        self.ai.view.push(Item::User(text));
        self.ai.input.clear();
        self.ai.rounds = 0;
        self.ai.apply_all = false;
        self.ai_request();
    }

    /// Stop: the request is dropped, waiting calls are answered "stopped".
    fn ai_stop(&mut self) {
        self.ai.waiting = None;
        let p = self.ai.provider.unwrap_or(Provider::Anthropic);
        let mut outs = std::mem::take(&mut self.ai.outs);
        for c in self.ai.pending.drain(..) {
            outs.push(ToolOut { id: c.id, text: "not run: stopped by the user".into(), is_error: true, png: None });
        }
        if let Some(i) = self.ai.asking.take() {
            if let Some(Item::Tool(t)) = self.ai.view.get_mut(i) {
                t.asking = false;
                t.result = "stopped".into();
            }
        }
        if !outs.is_empty() {
            ai::push_tool_results(p, &mut self.ai.msgs, &outs);
        }
        self.ai.view.push(Item::Note("stopped".into()));
    }

    /// Once per frame (in every tab): pick up an answer, run the tool calls.
    pub(super) fn ai_poll(&mut self, gl: &glow::Context, ctx: &egui::Context) {
        if let Some((rx, _)) = &self.ai.waiting {
            match rx.try_recv() {
                Ok(res) => {
                    self.ai.waiting = None;
                    match res {
                        Ok(r) => self.ai_reply(r),
                        Err(e) => self.ai.view.push(Item::Error(e)),
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(_) => {
                    self.ai.waiting = None;
                    self.ai.view.push(Item::Error("the request ended without an answer".into()));
                }
            }
        }
        while let Some(c) = self.ai.pending.front().cloned() {
            if self.cfg.ai_ask && changes(&c.name) && !self.ai.apply_all {
                match self.ai.decision.take() {
                    None => {
                        if self.ai.asking.is_none() {
                            self.ai.view.push(Item::Tool(ToolItem { name: c.name.clone(), args: call_text(&c), result: String::new(), error: false, undo: None, undone: false, image: None, asking: true }));
                            self.ai.asking = Some(self.ai.view.len() - 1);
                        }
                        return;
                    }
                    Some(false) => {
                        self.ai.pending.pop_front();
                        if let Some(Item::Tool(t)) = self.ai.asking.take().and_then(|i| self.ai.view.get_mut(i)) {
                            t.asking = false;
                            t.result = "skipped".into();
                            t.error = true;
                        }
                        self.ai.outs.push(ToolOut { id: c.id, text: "the user did not apply this change".into(), is_error: true, png: None });
                        continue;
                    }
                    Some(true) => {}
                }
            }
            self.ai.pending.pop_front();
            self.ai_run(gl, ctx, c);
        }
        // all calls of this answer done: send the results
        if self.ai.waiting.is_none() && !self.ai.outs.is_empty() {
            let outs = std::mem::take(&mut self.ai.outs);
            let p = self.ai.provider.unwrap_or(Provider::Anthropic);
            ai::push_tool_results(p, &mut self.ai.msgs, &outs);
            self.ai.rounds += 1;
            if self.ai.rounds >= MAX_ROUNDS {
                self.ai.view.push(Item::Note(format!("stopped after {MAX_ROUNDS} rounds of tool calls; send a message to go on")));
            } else {
                self.ai_request();
            }
        }
    }

    fn ai_reply(&mut self, r: Reply) {
        self.ai.usage.0 += r.usage.0;
        self.ai.usage.1 += r.usage.1;
        if let Some(why) = &r.refusal {
            self.ai.view.push(Item::Error(format!("the model declined{}", if why.is_empty() { String::new() } else { format!(": {why}") })));
            return;
        }
        if let Some(m) = r.message {
            self.ai.msgs.push(m);
        }
        for t in r.text {
            self.ai.view.push(Item::Text(t));
        }
        if r.stop == "max_tokens" && r.calls.is_empty() {
            self.ai.view.push(Item::Note("answer cut off (too long)".into()));
        }
        self.ai.pending.extend(r.calls);
    }

    fn ai_run(&mut self, gl: &glow::Context, ctx: &egui::Context, c: ToolCall) {
        // undo for `set`: the value before (triggers have none)
        let undo = (c.name == "set")
            .then(|| c.input["address"].as_str().map(String::from))
            .flatten()
            .and_then(|a| self.control_get(&a).map(|v| (a, v)));
        let r = self.mcp_call(gl, &c.name, &c.input, true);
        let text: Vec<String> = r.content.iter().filter_map(|x| if let Content::Text(t) = x { Some(t.clone()) } else { None }).collect();
        let mut png = r.content.iter().find_map(|x| if let Content::Png(b) = x { Some(b.clone()) } else { None });
        let image = png.as_ref().and_then(|b| decode_png(b)).map(|img| ctx.load_texture(format!("ai{}", self.ai.view.len()), img, Default::default()));
        let mut text = text.join("\n");
        if png.is_some() && !self.cfg.ai_images {
            png = None;
            text += "\n(the image is not sent: images are off in the ai tab)";
        }
        let item = ToolItem { name: c.name.clone(), args: call_text(&c), result: text.clone(), error: r.is_error, undo: undo.filter(|_| !r.is_error), undone: false, image, asking: false };
        match self.ai.asking.take() {
            Some(i) if i < self.ai.view.len() => self.ai.view[i] = Item::Tool(item),
            _ => self.ai.view.push(Item::Tool(item)),
        }
        self.ai.outs.push(ToolOut { id: c.id, text, is_error: r.is_error, png });
    }

    pub(super) fn ai_selftest_send(&mut self, text: &str) {
        self.ai.input = text.into();
        self.ai_send();
    }

    /// The chat as one line (self-test).
    pub(super) fn ai_selftest_summary(&self) -> String {
        let items: Vec<String> = self
            .ai
            .view
            .iter()
            .map(|it| match it {
                Item::User(t) => format!("user {t:?}"),
                Item::Text(t) => format!("text {t:?}"),
                Item::Tool(t) => format!("tool {} [{}] -> {:?}{}{}", t.name, t.args, t.result.lines().next().unwrap_or(""), if t.image.is_some() { " +image" } else { "" }, if t.undo.is_some() { " +undo" } else { "" }),
                Item::Note(t) => format!("note {t:?}"),
                Item::Error(t) => format!("ERROR {t:?}"),
            })
            .collect();
        format!("{} | tokens {:?} | {} messages", items.join(" | "), self.ai.usage, self.ai.msgs.len())
    }

    pub(super) fn ai_tab(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let before = self.cfg.clone();
        // ---- right: the output (what the ai changes)
        egui::Panel::right("ai_out").resizable(true).default_size(420.0).size_range(240.0..=900.0).show(ui, |ui| {
            self.output_preview(ui, frame);
            ui.add_space(6.0);
            ui.label(egui::RichText::new("The assistant uses the same tools as the MCP server: it reads the state, sets addresses, loads files, adds effects and looks at the output.").weak());
        });
        // ---- top: provider, model, key
        egui::Panel::top("ai_settings").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let p = Provider::from_str(&self.cfg.ai_provider);
                if widgets::choice(ui, p == Provider::Anthropic, "Anthropic").clicked() {
                    self.cfg.ai_provider = "anthropic".into();
                }
                if widgets::choice(ui, p == Provider::OpenAi, "OpenAI-compatible").on_hover_text("OpenAI, or a local server: Ollama (http://localhost:11434/v1), LM Studio (http://localhost:1234/v1) ...").clicked() {
                    self.cfg.ai_provider = "openai".into();
                }
                ui.separator();
                ui.label("model");
                match p {
                    Provider::Anthropic => {
                        ui.add(egui::TextEdit::singleline(&mut self.cfg.ai_model_anthropic).desired_width(150.0));
                        ui.menu_button("▾", |ui| {
                            for m in ai::ANTHROPIC_MODELS {
                                if ui.selectable_label(self.cfg.ai_model_anthropic == m, m).clicked() {
                                    self.cfg.ai_model_anthropic = m.into();
                                    ui.close();
                                }
                            }
                        });
                        ui.label("effort");
                        for e in ["low", "medium", "high"] {
                            if widgets::choice(ui, self.cfg.ai_effort == e, e).on_hover_text("how much the model thinks: low = fastest and cheapest").clicked() {
                                self.cfg.ai_effort = e.into();
                            }
                        }
                    }
                    Provider::OpenAi => {
                        ui.add(egui::TextEdit::singleline(&mut self.cfg.ai_model_openai).desired_width(150.0).hint_text("e.g. gpt-5, qwen3"));
                    }
                }
            });
            ui.horizontal(|ui| {
                let p = Provider::from_str(&self.cfg.ai_provider);
                let (key, url, hint) = match p {
                    Provider::Anthropic => (&mut self.cfg.ai_key_anthropic, &mut self.cfg.ai_url_anthropic, ai::ANTHROPIC_URL),
                    Provider::OpenAi => (&mut self.cfg.ai_key_openai, &mut self.cfg.ai_url_openai, ai::OPENAI_URL),
                };
                ui.label("key");
                let from_env = key.trim().is_empty() && std::env::var(env_key(p)).is_ok_and(|v| !v.trim().is_empty());
                let key_hint = if from_env {
                    format!("from {}", env_key(p))
                } else if p == Provider::OpenAi {
                    "not needed for local servers".into()
                } else {
                    "paste your key: sk-ant-...".into()
                };
                ui.add(egui::TextEdit::singleline(key).password(!self.ai.show_key).desired_width(220.0).hint_text(key_hint))
                    .on_hover_text(format!("stored in ~/.config/videoping/config.json (readable by you only); empty = the environment variable {}", env_key(p)));
                if widgets::choice(ui, self.ai.show_key, "show").clicked() {
                    self.ai.show_key = !self.ai.show_key;
                }
                if p == Provider::Anthropic {
                    ui.hyperlink_to("get a key", "https://console.anthropic.com/settings/keys").on_hover_text("Anthropic console → API keys → create key (usage is billed to that account)");
                }
                ui.label("url");
                ui.add(egui::TextEdit::singleline(url).desired_width(220.0).hint_text(hint));
                ui.separator();
                ui.checkbox(&mut self.cfg.ai_ask, "ask before changes").on_hover_text("set / load / add effect wait for apply in the chat (for shows)");
                ui.checkbox(&mut self.cfg.ai_images, "images").on_hover_text("send output frames to the model (some local models cannot take images)");
            });
            ui.add_space(2.0);
        });
        // ---- bottom: the message
        let mut send = false;
        let mut stop = false;
        let mut clear = false;
        egui::Panel::bottom("ai_input").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let busy = self.ai.busy();
                let w = ui.available_width() - 130.0;
                let r = ui.add(
                    egui::TextEdit::multiline(&mut self.ai.input)
                        .desired_rows(2)
                        .desired_width(w)
                        .hint_text("e.g. \"put a slow feedback effect on the output\" · Enter = send, Shift+Enter = new line")
                        .return_key(Some(egui::KeyboardShortcut::new(egui::Modifiers::SHIFT, egui::Key::Enter))),
                );
                if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
                    send = true;
                }
                ui.vertical(|ui| {
                    if busy {
                        if widgets::btn(ui, "stop").clicked() {
                            stop = true;
                        }
                    } else if widgets::btn(ui, "send").clicked() {
                        send = true;
                    }
                    if widgets::btn(ui, "new chat").on_hover_text("forget the conversation").clicked() {
                        clear = true;
                    }
                });
            });
            ui.horizontal(|ui| {
                let status = match &self.ai.waiting {
                    Some((_, t)) => format!("thinking … {:.0} s", t.elapsed().as_secs_f32()),
                    None if self.ai.asking.is_some() => "waiting for apply / skip".into(),
                    None => String::new(),
                };
                ui.label(egui::RichText::new(status).weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("tokens: {} in, {} out", self.ai.usage.0, self.ai.usage.1)).weak().size(widgets::BTN_TEXT))
                        .on_hover_text("this chat so far (every message costs API usage; input includes the cached part)");
                });
            });
        });
        // ---- the chat
        let mut undo: Option<usize> = None;
        let mut decide: Option<bool> = None;
        let mut apply_all = false;
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
                if self.ai.view.is_empty() {
                    ui.label(egui::RichText::new("Ask for something, e.g. \"what is playing?\", \"make layer 2 slowly fade out\", \"load a synthwave generator into layer 3\". The assistant changes videoping directly; every change is listed here.").weak());
                }
                for (i, it) in self.ai.view.iter().enumerate() {
                    match it {
                        Item::User(t) => {
                            ui.add_space(6.0);
                            egui::Frame::new().fill(Color32::from_gray(38)).corner_radius(4.0).inner_margin(6.0).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(t);
                            });
                        }
                        Item::Text(t) => {
                            egui_commonmark::CommonMarkViewer::new().show(ui, &mut self.md_cache, t);
                        }
                        Item::Note(t) => {
                            ui.label(egui::RichText::new(t).weak().italics());
                        }
                        Item::Error(t) => {
                            ui.colored_label(Color32::from_rgb(220, 80, 80), t);
                        }
                        Item::Tool(t) => {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                let c = if t.error { Color32::from_rgb(220, 80, 80) } else { ACCENT };
                                ui.label(egui::RichText::new(format!("⚙ {}", t.name)).color(c).monospace().size(widgets::BTN_TEXT));
                                ui.label(egui::RichText::new(&t.args).monospace().size(widgets::BTN_TEXT));
                                if t.asking {
                                    if widgets::btn(ui, "apply").clicked() {
                                        decide = Some(true);
                                    }
                                    if widgets::btn(ui, "skip").clicked() {
                                        decide = Some(false);
                                    }
                                    if widgets::btn(ui, "apply all").on_hover_text("apply this and the rest of this answer").clicked() {
                                        apply_all = true;
                                    }
                                } else if let Some((addr, _)) = &t.undo {
                                    if t.undone {
                                        ui.label(egui::RichText::new("undone").weak());
                                    } else if widgets::btn(ui, "undo").on_hover_text(format!("{addr} back to its value before")).clicked() {
                                        undo = Some(i);
                                    }
                                }
                            });
                            if !t.result.is_empty() {
                                let short: String = t.result.lines().take(4).collect::<Vec<_>>().join("\n");
                                let short = if short.chars().count() > 400 { format!("{}…", short.chars().take(400).collect::<String>()) } else { short };
                                let more = short.len() < t.result.len();
                                let r = ui.label(egui::RichText::new(short).weak().monospace().size(10.0));
                                if more {
                                    r.on_hover_ui(|ui| {
                                        ui.set_max_width(600.0);
                                        ui.label(egui::RichText::new(&t.result).monospace().size(10.0));
                                    });
                                }
                            }
                            if let Some(tex) = &t.image {
                                let s = tex.size_vec2();
                                let w = s.x.min(320.0);
                                ui.image((tex.id(), vec2(w, w * s.y / s.x.max(1.0))));
                            }
                        }
                    }
                }
            });
        });
        let undo = undo.and_then(|i| match self.ai.view.get(i) {
            Some(Item::Tool(t)) => t.undo.clone().map(|u| (i, u)),
            _ => None,
        });
        if let Some((i, (addr, v))) = undo {
            match self.control_set(&addr, &v) {
                Ok(()) => {
                    if let Some(Item::Tool(t)) = self.ai.view.get_mut(i) {
                        t.undone = true;
                    }
                    self.ai.notes.push(format!("the user undid your change: {addr} is back to {}", super::mcp_tools::value_text(&v)));
                }
                Err(e) => self.ai.view.push(Item::Error(format!("undo: {e}"))),
            }
        }
        if apply_all {
            self.ai.apply_all = true;
            self.ai.decision = Some(true);
        } else if decide.is_some() {
            self.ai.decision = decide;
        }
        if stop {
            self.ai_stop();
        }
        if clear {
            self.ai_stop();
            self.ai = AiChat { show_key: self.ai.show_key, ..Default::default() };
        }
        if send {
            self.ai_send();
        }
        if self.cfg != before && self.selftest.is_none() {
            if let Err(e) = save_config(&self.cfg) {
                self.set_status(e);
            }
        }
    }
}
