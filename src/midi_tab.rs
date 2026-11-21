//! The tab "midi mapping": devices (left), bindings of the selected device (middle), the control
//! registry addresses with "learn" (right). The MIDI engine is generic (midi.rs); the Bhoreal is
//! a built-in profile that can be edited here like any other.

use super::*;
use crate::midi::{Feedback, Learn, Mode};

const MODES: [Mode; 5] = [Mode::Trigger, Mode::Toggle, Mode::Momentary, Mode::Absolute, Mode::Relative];

fn mode_name(m: Mode) -> &'static str {
    match m {
        Mode::Trigger => "trigger",
        Mode::Toggle => "toggle",
        Mode::Momentary => "momentary",
        Mode::Absolute => "absolute",
        Mode::Relative => "relative",
    }
}

fn kind_name(k: &crate::control::Kind) -> String {
    use crate::control::Kind;
    match k {
        Kind::Trigger => "trigger".into(),
        Kind::Toggle => "on / off".into(),
        Kind::Range { min, max, integer } => {
            if *integer {
                format!("{min:.0} .. {max:.0}")
            } else {
                format!("{min} .. {max}")
            }
        }
        Kind::Choice(c) => c.join(" "),
    }
}

impl App {
    pub(super) fn midi_tab(&mut self, ui: &mut Ui) {
        let Some(mut m) = self.midi.take() else { return };
        let devices = m.devices();
        if self.midi_sel_port.as_ref().is_none_or(|p| !devices.iter().any(|d| &d.0 == p)) {
            self.midi_sel_port = devices.first().map(|d| d.0.clone());
        }

        // ---- left: devices + monitor
        egui::Panel::left("midi_devices").resizable(true).default_size(320.0).size_range(220.0..=600.0).show(ui, |ui| {
            ui.label("devices");
            if devices.is_empty() {
                ui.label(egui::RichText::new("no MIDI device found. Plug one in: it is found within 2 s.").weak());
            }
            for (port, _) in &devices {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                    let c = if m.active(port) { ACCENT } else { Color32::from_gray(60) };
                    ui.painter().circle_filled(r.center(), 4.0, c);
                    let sel = self.midi_sel_port.as_deref() == Some(port.as_str());
                    if ui.selectable_label(sel, crate::midi::port_base(port)).on_hover_text(port).clicked() {
                        self.midi_sel_port = Some(port.clone());
                    }
                });
                ui.horizontal(|ui| {
                    ui.add_space(18.0);
                    let cur = m.device_profile(port);
                    let names: Vec<String> = m.profiles().iter().map(|p| p.name.clone()).collect();
                    let text = cur.map(|i| names[i].clone()).unwrap_or("no profile".into());
                    egui::ComboBox::from_id_salt(("midi_prof", port)).selected_text(text).show_ui(ui, |ui| {
                        if ui.selectable_label(cur.is_none(), "no profile").clicked() {
                            m.set_device_profile(port, None);
                        }
                        for (i, n) in names.iter().enumerate() {
                            if ui.selectable_label(cur == Some(i), n).clicked() {
                                m.set_device_profile(port, Some(i));
                            }
                        }
                    });
                });
            }
            if let Some(e) = &m.error {
                ui.label(egui::RichText::new(e).color(Color32::RED));
            }
            ui.add_space(8.0);
            ui.separator();
            ui.label("monitor");
            for (port, text) in m.monitor.iter().take(16) {
                ui.label(egui::RichText::new(format!("{port:<14.14} {text}")).monospace().size(11.0).weak());
            }
        });

        // ---- right: addresses + learn
        egui::Panel::right("midi_addresses").resizable(true).default_size(400.0).size_range(260.0..=700.0).show(ui, |ui| {
            ui.label("addresses");
            ui.label(egui::RichText::new("choose an address, press learn, move a control on the selected device").weak().size(widgets::BTN_TEXT));
            ui.horizontal(|ui| {
                let learning = m.learn.is_some();
                let ready = self.midi_sel_port.is_some() && self.midi_sel_addr.is_some();
                if learning {
                    if widgets::choice(ui, true, "learning …").on_hover_text("move a control on the device; click to cancel").clicked() {
                        m.learn = None;
                    }
                } else if ui.add_enabled(ready, egui::Button::new(egui::RichText::new("learn").size(widgets::BTN_TEXT)).min_size(widgets::BTN)).clicked() {
                    let addr = self.midi_sel_addr.clone().unwrap_or_default();
                    let kind = self.control_list().into_iter().find(|d| d.addr == addr).map(|d| d.kind);
                    if let (Some(port), Some(kind)) = (self.midi_sel_port.clone(), kind) {
                        m.learned = None;
                        m.learn = Some(Learn { port, addr, kind });
                    }
                }
                if let Some(l) = &m.learned {
                    ui.label(egui::RichText::new(l).weak().size(widgets::BTN_TEXT));
                }
            });
            egui::CollapsingHeader::new(egui::RichText::new("what the addresses mean").size(widgets::BTN_TEXT)).default_open(false).show(ui, |ui| {
                for (a, t) in [
                    ("/layer/2/…", "mixer layer 2: select, active, mix, mixtype, r g b, unload"),
                    ("/layer/2/fx/3/…", "effect 3 of whatever plays in layer 2 now"),
                    ("/layer/2/param/4", "parameter 4 of the generator playing in layer 2"),
                    ("/slot/9/…", "slot 9 of the grid (top-down per column, page 2 starts at 33): load, select, state, its fx and params"),
                    ("/page/slot/9/…", "slot 9 of the page shown (a controller follows the page)"),
                    ("…/param/4", "the 4th parameter, counted top-down like the parameter list in the ui"),
                    ("/out/fx/1/…", "output effect 1"),
                    ("/master/…  /bpm  /record  /page", "master, tempo, recording, the prepare page"),
                ] {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(a).monospace().size(11.0));
                        ui.label(egui::RichText::new(t).weak().size(11.0));
                    });
                }
                ui.label(egui::RichText::new("a binding to a slot controls whatever is in that slot; a binding to a layer whatever plays in it").weak().size(11.0));
            });
            ui.add(egui::TextEdit::singleline(&mut self.midi_search).hint_text("search").desired_width(ui.available_width()));
            let q = self.midi_search.to_lowercase();
            let list: Vec<crate::control::Desc> =
                self.control_list().into_iter().filter(|d| d.writable && (q.is_empty() || d.addr.to_lowercase().contains(&q) || d.label.to_lowercase().contains(&q))).collect();
            egui::ScrollArea::vertical().id_salt("midi_addr_list").auto_shrink([false, false]).show(ui, |ui| {
                for d in list.iter().take(500) {
                    let sel = self.midi_sel_addr.as_deref() == Some(d.addr.as_str());
                    let tip = format!("{}\n{}", d.label, kind_name(&d.kind));
                    ui.horizontal(|ui| {
                        if ui.selectable_label(sel, egui::RichText::new(&d.addr).monospace().size(11.0)).on_hover_text(&tip).clicked() {
                            self.midi_sel_addr = Some(d.addr.clone());
                        }
                        ui.label(egui::RichText::new(kind_name(&d.kind)).weak().size(11.0));
                    });
                    // what it is, in words (slot / layer, instance, effect, parameter name)
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        ui.add(egui::Label::new(egui::RichText::new(&d.label).weak().size(11.0)).truncate()).on_hover_text(&tip);
                    });
                }
                if list.len() > 500 {
                    ui.label(egui::RichText::new(format!("… {} more, narrow the search", list.len() - 500)).weak());
                }
            });
        });

        // ---- middle: bindings of the selected device's profile
        egui::CentralPanel::default().show(ui, |ui| {
            let Some(port) = self.midi_sel_port.clone() else {
                ui.label(egui::RichText::new("no device selected").weak());
                return;
            };
            let Some(pi) = m.device_profile(&port) else {
                ui.label(format!("{}: no profile", crate::midi::port_base(&port)));
                if widgets::btn(ui, "new profile").on_hover_text("an empty profile for this device; then learn controls").clicked() {
                    m.new_profile(&port);
                }
                return;
            };
            let name = m.profiles()[pi].name.clone();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("profile {name}")).strong());
                if m.dirty.contains(&name) {
                    ui.label(egui::RichText::new("● not saved").weak());
                }
                if widgets::btn(ui, "save").on_hover_text("~/.config/videoping/midi/").clicked() {
                    match m.save_profile(pi) {
                        Ok(p) => self.set_status(format!("saved {}", p.display())),
                        Err(e) => self.set_status(e),
                    }
                }
                if m.is_builtin(pi) && widgets::btn(ui, "built-in").on_hover_text("back to the built-in version of this profile").clicked() {
                    m.revert_profile(pi);
                }
            });
            ui.add_space(4.0);
            let mut b = m.profiles()[pi].bindings.clone();
            let mut remove = None;
            let hit = m.last_hit.clone().filter(|h| h.port == port && h.at.elapsed().as_secs_f32() < 0.6);
            let sel_addr = self.midi_sel_addr.clone();
            let labels: HashMap<String, String> = self.control_list().into_iter().map(|d| (d.addr, d.label)).collect();
            egui::ScrollArea::vertical().id_salt("midi_bindings").auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new("midi_grid").striped(true).num_columns(5).spacing(vec2(10.0, 4.0)).show(ui, |ui| {
                    for h in ["control", "mode", "target", "feedback", ""] {
                        ui.label(egui::RichText::new(h).weak());
                    }
                    ui.end_row();
                    for (k, x) in b.iter_mut().enumerate() {
                        let hot = hit.as_ref().is_some_and(|h| h.channel == x.channel && h.msg == x.msg);
                        let ctl = format!("ch {} {}", x.channel + 1, crate::midi::msg_text(x.msg));
                        let t = egui::RichText::new(ctl).monospace().size(11.0);
                        ui.label(if hot { t.color(Color32::WHITE).background_color(ACCENT) } else { t });
                        egui::ComboBox::from_id_salt(("midi_mode", k)).selected_text(mode_name(x.mode)).width(96.0).show_ui(ui, |ui| {
                            for md in MODES {
                                ui.selectable_value(&mut x.mode, md, mode_name(md));
                            }
                        });
                        ui.horizontal(|ui| {
                            // what the target is right now; page slots resolve to the slot shown
                            let shown = self.page_slot_resolve(&x.addr);
                            let meaning = labels.get(&shown).cloned();
                            let r = ui.add(egui::TextEdit::singleline(&mut x.addr).font(egui::TextStyle::Monospace).desired_width(260.0));
                            match meaning {
                                Some(m) => {
                                    r.on_hover_text(m);
                                }
                                None => {
                                    r.on_hover_text("not there now (e.g. no effect with this number in that layer, empty slot)");
                                    ui.label(egui::RichText::new("not there now").weak().size(11.0));
                                }
                            }
                            if let Some(a) = &sel_addr {
                                if widgets::btn(ui, "←").on_hover_text(format!("use the address chosen on the right: {a}")).clicked() {
                                    x.addr = a.clone();
                                }
                            }
                        });
                        ui.horizontal(|ui| match &mut x.feedback {
                            Feedback::None => {
                                ui.label(egui::RichText::new("none").weak());
                            }
                            Feedback::OnOff { off, on } => {
                                ui.label(egui::RichText::new("off").weak());
                                ui.add(egui::DragValue::new(off).range(0..=127));
                                ui.label(egui::RichText::new("on").weak());
                                ui.add(egui::DragValue::new(on).range(0..=127));
                            }
                            Feedback::Values { map, .. } => {
                                let t: Vec<String> = map.iter().map(|(v, n)| format!("{v}={n}")).collect();
                                ui.label(egui::RichText::new(t.join(" ")).weak().size(11.0));
                            }
                            Feedback::Range => {
                                ui.label(egui::RichText::new("value").weak());
                            }
                        });
                        if widgets::btn(ui, "×").on_hover_text("remove this binding").clicked() {
                            remove = Some(k);
                        }
                        ui.end_row();
                    }
                });
                if b.is_empty() {
                    ui.label(egui::RichText::new("no bindings yet: choose an address on the right and press learn").weak());
                }
            });
            if let Some(k) = remove {
                b.remove(k);
            }
            if b != m.profiles()[pi].bindings {
                m.edit_profile(pi, |p| p.bindings = b);
            }
        });
        self.midi = Some(m);
    }
}
