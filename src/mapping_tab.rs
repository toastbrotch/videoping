//! The "projection mapping" tab (features.md "mapping"): presets, slices and masks, edited on an input view
//! (the composition) and an output view (the projector image). Child module of app.

use super::*;
use crate::mapping::{Mapping, Mask, Slice};

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Sel {
    Slice(usize),
    Mask(usize),
}

/// A grabbed handle. Corner index 0..3 (TL, TR, BR, BL), 4 = the whole thing.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Handle {
    In(usize, u8),
    Out(usize, u8),
    Point(usize, usize),
    MaskMove(usize),
}

#[derive(Default)]
pub(super) struct MapUi {
    sel: Option<Sel>,
    /// the handle the arrow keys move
    handle: Option<Handle>,
    drag: Option<Handle>,
    /// pointer position at the last drag frame (normalized)
    last: Option<[f32; 2]>,
    rename: Option<String>,
    confirm_delete: bool,
    /// the input and output view as last drawn (self-test)
    pub rects: Option<[Rect; 2]>,
}

/// Catch distance of a handle in screen pixels (also outside the view: corners on its border).
const HANDLE: f32 = 16.0;

/// The handle under `p`: the nearest corner / point within HANDLE (the selected item wins a tie),
/// else the inside of a mask or slice (= move it).
fn pick(m: &Mapping, sel: Option<Sel>, p: egui::Pos2, rin: Rect, rout: Rect) -> Option<Handle> {
    let mut best: Option<(f32, Handle)> = None;
    let mut consider = |d: f32, h: Handle, selected: bool| {
        let d = if selected { d - 4.0 } else { d };
        if d <= HANDLE && best.is_none_or(|(b, _)| d < b) {
            best = Some((d, h));
        }
    };
    let dist = |r: Rect, q: [f32; 2]| (to_screen(r, q) - p).length();
    if rin.expand(HANDLE).contains(p) {
        for (i, s) in m.slices.iter().enumerate() {
            for (k, q) in rect_corners(s.input).iter().enumerate() {
                consider(dist(rin, *q), Handle::In(i, k as u8), sel == Some(Sel::Slice(i)));
            }
        }
        if best.is_none() {
            let n = to_norm(rin, p);
            // the selected slice first
            let order = (0..m.slices.len()).filter(|i| sel == Some(Sel::Slice(*i))).chain((0..m.slices.len()).rev());
            for i in order {
                if inside(&rect_corners(m.slices[i].input), n) {
                    return Some(Handle::In(i, 4));
                }
            }
        }
    } else if rout.expand(HANDLE).contains(p) {
        for (i, s) in m.slices.iter().enumerate() {
            for (k, q) in s.output.iter().enumerate() {
                consider(dist(rout, *q), Handle::Out(i, k as u8), sel == Some(Sel::Slice(i)));
            }
        }
        for (mi, k) in m.masks.iter().enumerate() {
            for (pi, q) in k.points.iter().enumerate() {
                consider(dist(rout, *q), Handle::Point(mi, pi), sel == Some(Sel::Mask(mi)));
            }
        }
        if best.is_none() {
            let n = to_norm(rout, p);
            if let Some(mi) = (0..m.masks.len()).find(|i| sel == Some(Sel::Mask(*i)) && inside(&m.masks[*i].points, n)).or_else(|| m.masks.iter().position(|k| k.points.len() >= 3 && inside(&k.points, n))) {
                return Some(Handle::MaskMove(mi));
            }
            let order = (0..m.slices.len()).filter(|i| sel == Some(Sel::Slice(*i))).chain((0..m.slices.len()).rev());
            for i in order {
                if inside(&m.slices[i].output, n) {
                    return Some(Handle::Out(i, 4));
                }
            }
        }
    }
    best.map(|(_, h)| h)
}

fn to_screen(r: Rect, p: [f32; 2]) -> egui::Pos2 {
    egui::pos2(r.left() + p[0] * r.width(), r.top() + p[1] * r.height())
}

fn to_norm(r: Rect, p: egui::Pos2) -> [f32; 2] {
    [(p.x - r.left()) / r.width(), (p.y - r.top()) / r.height()]
}

fn rect_corners(i: [f32; 4]) -> [[f32; 2]; 4] {
    [[i[0], i[1]], [i[0] + i[2], i[1]], [i[0] + i[2], i[1] + i[3]], [i[0], i[1] + i[3]]]
}

fn inside(poly: &[[f32; 2]], p: [f32; 2]) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + n - 1) % n]);
        if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0] {
            c = !c;
        }
    }
    c
}

/// Index of the polygon edge (i -> i+1) nearest to `p` within `max` (screen pixels).
fn near_edge(r: Rect, poly: &[[f32; 2]], p: egui::Pos2, max: f32) -> Option<usize> {
    let n = poly.len();
    (0..n)
        .map(|i| {
            let a = to_screen(r, poly[i]);
            let b = to_screen(r, poly[(i + 1) % n]);
            let e = b - a;
            let t = ((p - a).dot(e) / e.length_sq().max(1e-6)).clamp(0.0, 1.0);
            (i, (p - (a + e * t)).length())
        })
        .filter(|(_, d)| *d <= max)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

impl App {
    /// Self-test (VIDEOPING_SELFTEST_MAP with the mapping tab): press 12 px beside the top right
    /// corner of slice 2 in the output view, release, → → →, Shift+↓; then print where it is.
    pub(super) fn selftest_map_input(&mut self, raw: &mut egui::RawInput, step: u32) {
        let Some([_, r]) = self.map_ui.rects else { return };
        let corner = |app: &App| app.maps.get(&app.s.output.mapping).and_then(|m| m.slices.get(1)).map(|s| s.output[1]);
        thread_local!(static START: std::cell::Cell<Option<[f32; 2]>> = const { std::cell::Cell::new(None) });
        let Some(now) = corner(self) else { return };
        if step == 0 {
            START.with(|c| c.set(Some(now)));
        }
        let c0 = START.with(|c| c.get()).unwrap_or(now);
        let p = to_screen(r, c0) + vec2(-8.5, 8.5); // 12 px away
        let btn = |pressed| egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
        let key = |k, shift: bool| egui::Event::Key { key: k, physical_key: None, pressed: true, repeat: false, modifiers: if shift { egui::Modifiers::SHIFT } else { egui::Modifiers::NONE } };
        match step {
            0 => {
                self.map_ui.handle = None;
                raw.events.push(egui::Event::PointerMoved(p));
            }
            1 => raw.events.push(btn(true)),
            2 => raw.events.push(btn(false)),
            3..=5 => raw.events.push(key(egui::Key::ArrowRight, false)),
            6 => raw.events.push(key(egui::Key::ArrowDown, true)),
            8 => {
                let (w, h) = (self.s.output.width as f32, self.s.output.height as f32);
                let c1 = corner(self).unwrap_or(c0);
                eprintln!(
                    "selftest mapping input: grabbed {:?} from 12 px away; corner moved by {:.1}, {:.1} px (expected 3, 10)",
                    self.map_ui.handle,
                    (c1[0] - c0[0]) * w,
                    (c1[1] - c0[1]) * h
                );
            }
            _ => {}
        }
    }

    /// The preset being edited (= the active one).
    fn map_current(&mut self) -> Option<&mut Mapping> {
        let name = self.s.output.mapping.clone();
        self.maps.get_mut(&name)
    }

    pub(super) fn mapping_tab(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        // a missing preset (deleted, other computer): the first one
        if self.maps.get(&self.s.output.mapping).is_none() {
            self.s.output.mapping = self.maps.list.first().map(|m| m.name.clone()).unwrap_or_default();
        }
        self.mapping_bar(ui);
        egui::Panel::left("map_items").resizable(true).default_size(260.0).size_range(200.0..=500.0).show(ui, |ui| {
            self.mapping_items(ui);
        });
        egui::CentralPanel::default().show(ui, |ui| {
            if self.maps.get(&self.s.output.mapping).is_none() {
                ui.label(egui::RichText::new("no mapping preset yet: \"new\" above makes one (one slice over the whole output)").weak());
                return;
            }
            self.mapping_views(ui, frame);
        });
    }

    fn mapping_bar(&mut self, ui: &mut Ui) {
        egui::Panel::top("map_bar").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let o = &mut self.s.output;
                ui.checkbox(&mut o.mapping_on, egui::RichText::new("mapping on").strong()).on_hover_text("the screen output shows the mapping (off: the plain composition)");
                ui.separator();
                ui.label("preset");
                let cur = o.mapping.clone();
                egui::ComboBox::from_id_salt("map_preset").selected_text(if cur.is_empty() { "—".to_string() } else { cur.clone() }).show_ui(ui, |ui| {
                    for m in &self.maps.list {
                        if ui.selectable_label(m.name == cur, &m.name).clicked() {
                            self.s.output.mapping = m.name.clone();
                            self.map_ui.sel = None;
                            self.map_ui.handle = None;
                        }
                    }
                });
                if widgets::btn(ui, "new").on_hover_text("a new preset: one slice over the whole output").clicked() {
                    let name = self.maps.free_name("mapping");
                    self.s.output.mapping = self.maps.add(Mapping::new(&name));
                    self.map_ui.sel = Some(Sel::Slice(0));
                }
                let has = self.maps.get(&cur).is_some();
                if has && widgets::btn(ui, "duplicate").clicked() {
                    if let Some(mut m) = self.maps.get(&cur).cloned() {
                        m.name = self.maps.free_name(&format!("{cur} copy"));
                        self.s.output.mapping = self.maps.add(m);
                    }
                }
                if has && widgets::btn(ui, "rename").clicked() {
                    self.map_ui.rename = Some(cur.clone());
                }
                if has && widgets::btn(ui, "delete").clicked() {
                    self.map_ui.confirm_delete = true;
                }
                ui.separator();
                let o = &mut self.s.output;
                ui.checkbox(&mut self.map_test, "test grid").on_hover_text("the slices show a grid instead of the composition: for lining up on the real surface (on the screen while the mapping is on)");
                ui.checkbox(&mut o.map_ndi, "also on NDI").on_hover_text("NDI sends the mapped output (off: the clean composition)");
                ui.checkbox(&mut o.map_rec, "also on the recording").on_hover_text("REC records the mapped output (off: the clean composition)");
            });
            if let Some(mut name) = self.map_ui.rename.take() {
                let mut keep = true;
                ui.horizontal(|ui| {
                    ui.label("new name");
                    let r = ui.add(egui::TextEdit::singleline(&mut name).desired_width(200.0));
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if widgets::btn(ui, "ok").clicked() || enter {
                        let old = self.s.output.mapping.clone();
                        match self.maps.rename(&old, &name) {
                            Ok(()) => {
                                self.s.output.mapping = name.trim().to_string();
                                keep = false;
                            }
                            Err(e) => self.set_status(e),
                        }
                    }
                    if widgets::btn(ui, "cancel").clicked() {
                        keep = false;
                    }
                });
                if keep {
                    self.map_ui.rename = Some(name);
                }
            }
            if self.map_ui.confirm_delete {
                ui.horizontal(|ui| {
                    ui.label(format!("delete the preset \"{}\"?", self.s.output.mapping));
                    if widgets::btn(ui, "delete").clicked() {
                        let name = self.s.output.mapping.clone();
                        self.maps.delete(&name);
                        self.s.output.mapping = self.maps.list.first().map(|m| m.name.clone()).unwrap_or_default();
                        self.map_ui.confirm_delete = false;
                        self.map_ui.sel = None;
                    }
                    if widgets::btn(ui, "cancel").clicked() {
                        self.map_ui.confirm_delete = false;
                    }
                });
            }
            ui.add_space(2.0);
        });
    }

    /// Left: the slices and masks, and the numbers of the selected one.
    fn mapping_items(&mut self, ui: &mut Ui) {
        let (w, h) = (self.s.output.width.max(1) as f32, self.s.output.height.max(1) as f32);
        let mut sel = self.map_ui.sel;
        let Some(m) = self.map_current() else { return };
        let mut remove: Option<Sel> = None;
        ui.horizontal(|ui| {
            ui.label("slices");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::btn(ui, "+").on_hover_text("add a slice (the whole composition, on the whole output)").clicked() {
                    let n = m.slices.len() + 1;
                    m.slices.push(Slice::full(&format!("slice {n}")));
                    sel = Some(Sel::Slice(m.slices.len() - 1));
                }
            });
        });
        for (i, s) in m.slices.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let on = sel == Some(Sel::Slice(i));
                if on {
                    ui.add(egui::TextEdit::singleline(&mut s.name).desired_width(ui.available_width() - 30.0));
                } else if ui.add(egui::Button::selectable(false, &s.name)).clicked() {
                    sel = Some(Sel::Slice(i));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if widgets::btn(ui, "×").clicked() {
                        remove = Some(Sel::Slice(i));
                    }
                });
            });
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label("masks");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::btn(ui, "+").on_hover_text("add a mask (a square in the middle: drag its points)").clicked() {
                    let n = m.masks.len() + 1;
                    m.masks.push(Mask::new(&format!("mask {n}")));
                    sel = Some(Sel::Mask(m.masks.len() - 1));
                }
            });
        });
        for (i, k) in m.masks.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let on = sel == Some(Sel::Mask(i));
                if on {
                    ui.add(egui::TextEdit::singleline(&mut k.name).desired_width(ui.available_width() - 30.0));
                } else if ui.add(egui::Button::selectable(false, &k.name)).clicked() {
                    sel = Some(Sel::Mask(i));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if widgets::btn(ui, "×").clicked() {
                        remove = Some(Sel::Mask(i));
                    }
                });
            });
        }
        ui.separator();
        // the numbers of the selected item, in pixels of the output
        match sel {
            Some(Sel::Slice(i)) if i < m.slices.len() => {
                let s = &mut m.slices[i];
                ui.label(egui::RichText::new("input (composition)").weak());
                let mut px = [s.input[0] * w, s.input[1] * h, s.input[2] * w, s.input[3] * h];
                let mut changed = false;
                egui::Grid::new("map_in").num_columns(4).show(ui, |ui| {
                    for (k, l) in ["x", "y", "w", "h"].iter().enumerate() {
                        ui.label(*l);
                        changed |= ui.add(egui::DragValue::new(&mut px[k]).speed(1.0).max_decimals(1)).changed();
                        if k % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
                if changed {
                    s.input = [px[0] / w, px[1] / h, (px[2] / w).max(0.001), (px[3] / h).max(0.001)];
                }
                ui.label(egui::RichText::new("output corners").weak());
                egui::Grid::new("map_out").num_columns(3).show(ui, |ui| {
                    for (k, l) in ["top left", "top right", "bottom right", "bottom left"].iter().enumerate() {
                        ui.label(*l);
                        let mut x = s.output[k][0] * w;
                        let mut y = s.output[k][1] * h;
                        if ui.add(egui::DragValue::new(&mut x).speed(1.0).max_decimals(1)).changed() {
                            s.output[k][0] = x / w;
                        }
                        if ui.add(egui::DragValue::new(&mut y).speed(1.0).max_decimals(1)).changed() {
                            s.output[k][1] = y / h;
                        }
                        ui.end_row();
                    }
                });
                ui.horizontal(|ui| {
                    if widgets::btn(ui, "output = input").on_hover_text("the corners back onto the input rectangle (no distortion)").clicked() {
                        s.output = rect_corners(s.input);
                    }
                    if widgets::btn(ui, "full").on_hover_text("input and output: the whole picture").clicked() {
                        let name = s.name.clone();
                        *s = Slice::full(&name);
                    }
                });
            }
            Some(Sel::Mask(i)) if i < m.masks.len() => {
                let k = &mut m.masks[i];
                ui.horizontal(|ui| {
                    ui.label("soft edge");
                    ui.add(egui::Slider::new(&mut k.feather, 0.0..=200.0).suffix(" px"));
                });
                ui.checkbox(&mut k.invert, "invert: keep only the inside");
                ui.label(egui::RichText::new(format!("{} points · double click on an edge = new point, right click on a point = remove it", k.points.len())).weak());
            }
            _ => {
                ui.label(egui::RichText::new("select a slice or mask; drag its handles in the views. Arrow keys move the last grabbed handle by 1 px (Shift: 10 px).").weak());
            }
        }
        match remove {
            Some(Sel::Slice(i)) => {
                m.slices.remove(i);
                sel = None;
            }
            Some(Sel::Mask(i)) => {
                m.masks.remove(i);
                sel = None;
            }
            None => {}
        }
        if sel != self.map_ui.sel {
            self.map_ui.sel = sel;
            self.map_ui.handle = None;
        }
    }

    /// Input view (composition + slice rectangles) and output view (projector + corners, masks).
    fn mapping_views(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let aspect = self.output_aspect;
        let avail = ui.available_size();
        // side by side when there is room, else one above the other
        let side = avail.x / avail.y > aspect * 1.2;
        let vw = if side { (avail.x - 16.0) / 2.0 } else { avail.x };
        let vh = if side { avail.y - 20.0 } else { (avail.y - 40.0) / 2.0 };
        let size = {
            let w = vw.min(vh * aspect);
            vec2(w, w / aspect)
        };
        let in_tex = self.r.as_ref().map(|r| if self.map_test { r.grid_tex() } else { r.out.tex });
        let out_tex = self.r.as_ref().map(|r| r.mapped.tex);
        let in_id = self.tex_id(frame, in_tex);
        let out_id = self.tex_id(frame, out_tex);
        let layout = if side { egui::Layout::left_to_right(egui::Align::Min) } else { egui::Layout::top_down(egui::Align::Min) };
        let mut rects = [Rect::NOTHING; 2];
        ui.with_layout(layout, |ui| {
            for (k, (title, id)) in [("input · composition", in_id), ("output · projector", out_id)].into_iter().enumerate() {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(title).weak());
                    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                    paint_tex(ui, rect, id, aspect);
                    rects[k] = rect;
                });
                ui.add_space(8.0);
            }
        });
        self.map_ui.rects = Some(rects);
        self.mapping_interact(ui, rects[0], rects[1]);
    }

    fn mapping_interact(&mut self, ui: &mut Ui, rin: Rect, rout: Rect) {
        let (w, h) = (self.s.output.width.max(1) as f32, self.s.output.height.max(1) as f32);
        let sel = self.map_ui.sel;
        let ptr = ui.input(|i| i.pointer.hover_pos());
        let pressed = ui.input(|i| i.pointer.primary_pressed());
        let down = ui.input(|i| i.pointer.primary_down());
        let dbl = ui.input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary));
        let right = ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Secondary));
        let id_in = ui.scope_id().with("map_in_view");
        let id_out = ui.scope_id().with("map_out_view");
        // the views take the mouse (no window dragging etc.)
        let resp_in = ui.interact(rin.expand(HANDLE), id_in, Sense::click_and_drag());
        let resp_out = ui.interact(rout.expand(HANDLE), id_out, Sense::click_and_drag());
        let Some(m) = self.maps.get(&self.s.output.mapping).cloned() else { return };
        let mut m2 = m.clone();
        let mut new_sel = sel;
        // ---- grab
        if pressed {
            if let Some(p) = ptr {
                let grab = pick(&m, sel, p, rin, rout);
                if let Some(g) = grab {
                    new_sel = Some(match g {
                        Handle::In(i, _) | Handle::Out(i, _) => Sel::Slice(i),
                        Handle::Point(i, _) | Handle::MaskMove(i) => Sel::Mask(i),
                    });
                    self.map_ui.handle = Some(g);
                    self.map_ui.drag = Some(g);
                    self.map_ui.last = None;
                }
            }
        }
        if !down {
            self.map_ui.drag = None;
            self.map_ui.last = None;
        }
        // ---- drag
        if let (Some(g), Some(p)) = (self.map_ui.drag, ptr) {
            let r = match g {
                Handle::In(..) => rin,
                _ => rout,
            };
            let n = to_norm(r, p);
            let d = self.map_ui.last.map(|l| [n[0] - l[0], n[1] - l[1]]).unwrap_or([0.0, 0.0]);
            self.map_ui.last = Some(n);
            // relative: a handle grabbed beside its point does not jump to the mouse
            move_handle(&mut m2, g, None, d);
        }
        // ---- masks: double click on an edge = new point, right click on a point = remove it
        if let (Some(Sel::Mask(mi)), Some(p)) = (new_sel, ptr) {
            if rout.contains(p) {
                if let Some(k) = m2.masks.get_mut(mi) {
                    if dbl && k.points.len() < crate::mapping::MAX_MASK_POINTS {
                        if let Some(e) = near_edge(rout, &k.points, p, HANDLE) {
                            k.points.insert(e + 1, to_norm(rout, p));
                        }
                    }
                    if right && k.points.len() > 3 {
                        if let Some(pi) = k.points.iter().position(|q| (to_screen(rout, *q) - p).length() <= HANDLE) {
                            k.points.remove(pi);
                            self.map_ui.handle = None;
                        }
                    }
                }
            }
        }
        // ---- arrow keys: the last grabbed handle, 1 px (Shift 10 px)
        if let Some(g) = self.map_ui.handle {
            if !ui.ctx().egui_wants_keyboard_input() {
                // consumed: the arrows must not also move the keyboard focus or scroll
                let (dx, dy) = ui.input_mut(|i| {
                    let mut d = (0.0, 0.0);
                    i.events.retain(|e| {
                        let egui::Event::Key { key, pressed: true, modifiers, .. } = e else { return true };
                        let v = match key {
                            egui::Key::ArrowLeft => (-1.0, 0.0),
                            egui::Key::ArrowRight => (1.0, 0.0),
                            egui::Key::ArrowUp => (0.0, -1.0),
                            egui::Key::ArrowDown => (0.0, 1.0),
                            _ => return true,
                        };
                        let step = if modifiers.shift { 10.0 } else { 1.0 };
                        d.0 += v.0 * step;
                        d.1 += v.1 * step;
                        false
                    });
                    d
                });
                if dx != 0.0 || dy != 0.0 {
                    move_handle(&mut m2, g, None, [dx / w, dy / h]);
                }
            }
        }
        // ---- the mouse shows what a press would grab
        let hover = self.map_ui.drag.or_else(|| ptr.and_then(|p| pick(&m2, new_sel, p, rin, rout)));
        if let Some(h) = hover {
            let icon = match (self.map_ui.drag.is_some(), h) {
                (true, _) => egui::CursorIcon::Grabbing,
                (false, Handle::In(_, 4) | Handle::Out(_, 4) | Handle::MaskMove(_)) => egui::CursorIcon::Move,
                (false, _) => egui::CursorIcon::Crosshair,
            };
            ui.ctx().set_cursor_icon(icon);
        }
        // ---- draw
        let painter = ui.painter();
        // the handle the arrow keys move: a ring and its position in pixels
        let handle_pos = |m: &Mapping, h: Handle| -> Option<(Rect, [f32; 2])> {
            match h {
                Handle::In(i, k) if k < 4 => m.slices.get(i).map(|s| (rin, rect_corners(s.input)[k as usize])),
                Handle::Out(i, k) if k < 4 => m.slices.get(i).map(|s| (rout, s.output[k as usize])),
                Handle::Point(i, pi) => m.masks.get(i).and_then(|k| k.points.get(pi)).map(|q| (rout, *q)),
                _ => None,
            }
        };
        if let Some((r, q)) = self.map_ui.handle.and_then(|h| handle_pos(&m2, h)) {
            let c = to_screen(r, q);
            painter.circle_stroke(c, 10.0, egui::Stroke::new(1.5, Color32::WHITE));
            painter.text(
                c + vec2(14.0, -14.0),
                egui::Align2::LEFT_BOTTOM,
                format!("{:.0}, {:.0}  ← → ↑ ↓", q[0] * w, q[1] * h),
                egui::FontId::monospace(11.0),
                Color32::WHITE,
            );
        }
        let dim = Color32::from_gray(150);
        for (i, s) in m2.slices.iter().enumerate() {
            let on = new_sel == Some(Sel::Slice(i));
            let c = if on { ACCENT } else { dim };
            let stroke = egui::Stroke::new(if on { 2.0 } else { 1.0 }, c);
            for (r, poly) in [(rin, rect_corners(s.input)), (rout, s.output)] {
                let pts: Vec<egui::Pos2> = poly.iter().map(|q| to_screen(r, *q)).collect();
                painter.add(egui::Shape::closed_line(pts.clone(), stroke));
                let center = pts.iter().fold(vec2(0.0, 0.0), |a, p| a + p.to_vec2()) / 4.0;
                painter.text(center.to_pos2(), egui::Align2::CENTER_CENTER, &s.name, egui::FontId::proportional(11.0), c);
                if on {
                    for p in &pts {
                        painter.rect_filled(Rect::from_center_size(*p, vec2(7.0, 7.0)), 1.0, c);
                    }
                }
            }
        }
        for (i, k) in m2.masks.iter().enumerate() {
            let on = new_sel == Some(Sel::Mask(i));
            let c = if on { Color32::from_gray(240) } else { Color32::from_gray(120) };
            let pts: Vec<egui::Pos2> = k.points.iter().map(|q| to_screen(rout, *q)).collect();
            painter.add(egui::Shape::closed_line(pts.clone(), egui::Stroke::new(if on { 1.5 } else { 1.0 }, c)));
            if on {
                for p in &pts {
                    painter.circle_filled(*p, 4.0, c);
                }
            }
        }
        // ---- keep
        if m2 != m {
            if let Some(cur) = self.map_current() {
                *cur = m2;
            }
        }
        if new_sel != sel {
            self.map_ui.sel = new_sel;
        }
        let _ = (resp_in, resp_out);
    }
}

/// Move a handle: to the pointer `to` (normalized) when given, else by `d`.
fn move_handle(m: &mut Mapping, g: Handle, to: Option<[f32; 2]>, d: [f32; 2]) {
    let clamp = |p: [f32; 2]| [p[0].clamp(-0.5, 1.5), p[1].clamp(-0.5, 1.5)];
    match g {
        Handle::In(i, k) => {
            let Some(s) = m.slices.get_mut(i) else { return };
            let [x, y, w, h] = s.input;
            if k == 4 {
                s.input[0] = (x + d[0]).clamp(0.0, 1.0 - w);
                s.input[1] = (y + d[1]).clamp(0.0, 1.0 - h);
                return;
            }
            // a corner: the opposite one stays
            let c = rect_corners(s.input);
            let opp = c[((k + 2) % 4) as usize];
            let mut p = to.unwrap_or([c[k as usize][0] + d[0], c[k as usize][1] + d[1]]);
            p = [p[0].clamp(0.0, 1.0), p[1].clamp(0.0, 1.0)];
            let (x0, x1) = (p[0].min(opp[0]), p[0].max(opp[0]));
            let (y0, y1) = (p[1].min(opp[1]), p[1].max(opp[1]));
            s.input = [x0, y0, (x1 - x0).max(0.005), (y1 - y0).max(0.005)];
        }
        Handle::Out(i, k) => {
            let Some(s) = m.slices.get_mut(i) else { return };
            if k == 4 {
                for q in s.output.iter_mut() {
                    *q = clamp([q[0] + d[0], q[1] + d[1]]);
                }
            } else {
                let q = &mut s.output[k as usize];
                *q = clamp(to.unwrap_or([q[0] + d[0], q[1] + d[1]]));
            }
        }
        Handle::Point(i, pi) => {
            if let Some(q) = m.masks.get_mut(i).and_then(|k| k.points.get_mut(pi)) {
                *q = clamp(to.unwrap_or([q[0] + d[0], q[1] + d[1]]));
            }
        }
        Handle::MaskMove(i) => {
            if let Some(k) = m.masks.get_mut(i) {
                for q in k.points.iter_mut() {
                    *q = clamp([q[0] + d[0], q[1] + d[1]]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles() {
        let mut m = Mapping::new("t");
        // input corner: the opposite corner stays
        move_handle(&mut m, Handle::In(0, 2), Some([0.5, 0.5]), [0.0, 0.0]);
        assert_eq!(m.slices[0].input, [0.0, 0.0, 0.5, 0.5]);
        // move the rectangle, clamped inside the picture
        move_handle(&mut m, Handle::In(0, 4), None, [0.7, 0.1]);
        assert_eq!(m.slices[0].input, [0.5, 0.1, 0.5, 0.5]);
        // output corner by the arrow keys
        move_handle(&mut m, Handle::Out(0, 0), None, [0.01, 0.02]);
        assert_eq!(m.slices[0].output[0], [0.01, 0.02]);
        assert!(inside(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]], [0.5, 0.5]));
        assert!(!inside(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]], [1.5, 0.5]));
    }
}
