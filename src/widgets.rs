//! Custom widgets: range slider, animation controls, clip timeline, small toggles.

use crate::state::{AnimParam, ClipTransport, LoopMode, Mix, MixType, PlayMode, Speed};
use egui::{pos2, vec2, Color32, Rect, Sense, Stroke, StrokeKind, Ui};

/// Accent color (selected buttons, limiters, range sliders, effect checkbox): muted plum #8B4A87,
/// chosen to sit in the hue gap furthest away from the layer colors (blue, green, orange).
pub const ACCENT: Color32 = Color32::from_rgb(0x8B, 0x4A, 0x87);

/// 0..1 value shown and edited as integer percent 0..100.
pub fn percent_slider(ui: &mut Ui, v: &mut f32, label: &str) -> egui::Response {
    let mut pct = (*v * 100.0).round();
    let r = ui.add(egui::Slider::new(&mut pct, 0.0..=100.0).step_by(1.0).fixed_decimals(0).suffix(" %").text(label));
    if r.changed() {
        *v = pct / 100.0;
    }
    r
}

/// Minimum size of every button (= the R G B buttons).
pub const BTN: egui::Vec2 = vec2(30.0, 18.0);
/// Text size of buttons.
pub const BTN_TEXT: f32 = 12.0;
/// Corner radius of buttons, fields and checkboxes.
pub const BTN_RADIUS: u8 = 2;

/// Plain button, same size as the R G B buttons.
pub fn btn(ui: &mut Ui, text: &str) -> egui::Response {
    ui.add(egui::Button::new(egui::RichText::new(text).size(BTN_TEXT)).min_size(BTN))
}

/// Big checkbox (18x18): filled with `on_color` and a check mark when on.
/// Used for effects (active) and mixer layers (on).
pub fn check(ui: &mut Ui, on: &mut bool, on_color: Color32) -> egui::Response {
    let (rect, mut r) = ui.allocate_exact_size(vec2(BTN.y, BTN.y), Sense::click());
    if r.clicked() {
        *on = !*on;
        r.mark_changed();
    }
    let v = ui.style().interact(&r);
    let p = ui.painter();
    if *on {
        p.rect_filled(rect, BTN_RADIUS as f32, on_color);
        p.text(rect.center(), egui::Align2::CENTER_CENTER, "✔", egui::FontId::proportional(13.0), readable_on(on_color));
    } else {
        p.rect_filled(rect, BTN_RADIUS as f32, ui.visuals().widgets.inactive.weak_bg_fill);
    }
    p.rect_stroke(rect, BTN_RADIUS as f32, Stroke::new(1.0, v.fg_stroke.color.gamma_multiply(0.6)), StrokeKind::Inside);
    r
}

/// Black or white text, whichever reads better on `bg`.
pub fn readable_on(bg: Color32) -> Color32 {
    let l = 0.299 * bg.r() as f32 + 0.587 * bg.g() as f32 + 0.114 * bg.b() as f32;
    if l > 140.0 { Color32::BLACK } else { Color32::WHITE }
}

/// Button that is highlighted when `selected`.
pub fn choice(ui: &mut Ui, selected: bool, label: &str) -> egui::Response {
    let fill = if selected { ACCENT } else { ui.visuals().widgets.inactive.weak_bg_fill };
    let text_color = if selected { readable_on(ACCENT) } else { ui.visuals().text_color() };
    ui.add(egui::Button::new(egui::RichText::new(label).color(text_color).size(BTN_TEXT)).fill(fill).min_size(BTN))
}

pub fn layer_color(i: usize) -> Color32 {
    let c = crate::state::LAYER_COLORS[i % crate::state::NUM_LAYERS];
    Color32::from_rgb(c[0], c[1], c[2])
}

// ------------------------------------------------------------ bracket handles
// Shared by the clip timeline and the parameter range sliders: "[" at the start, "]" at the
// end of a limited part. What is grabbed is decided at the press position (not at "drag
// started", when the pointer has already moved), and the grab offset is kept.

/// Width of the grab handles.
const HANDLE_W: f32 = 9.0;
/// Height of timelines and range sliders.
const BAR_H: f32 = 22.0;

/// Grabbed part of a bar: 0 = body (scrub / set value), 1 = start "[", 2 = end "]".
#[derive(Clone, Copy)]
struct Grab {
    what: u8,
    offset: f32,
}

struct Brackets {
    start: Rect,
    end: Rect,
    over_start: bool,
    over_end: bool,
}

impl Brackets {
    fn new(outer: Rect, x_start: f32, x_end: f32, hover: Option<egui::Pos2>) -> Brackets {
        let start = Rect::from_min_max(pos2(x_start - 2.0, outer.top()), pos2(x_start + HANDLE_W, outer.bottom()));
        let end = Rect::from_min_max(pos2(x_end - HANDLE_W, outer.top()), pos2(x_end + 2.0, outer.bottom()));
        let mut b = Brackets { start, end, over_start: false, over_end: false };
        if let Some(p) = hover {
            b.over_start = b.hit(b.start, p);
            b.over_end = b.hit(b.end, p);
        }
        b
    }
    fn hit(&self, r: Rect, p: egui::Pos2) -> bool {
        r.expand2(vec2(4.0, 0.0)).contains(p)
    }
    /// what is under `p` (both handles: the nearer one wins)
    fn pick(&self, p: egui::Pos2, x_start: f32, x_end: f32) -> Grab {
        let mid = (x_start + x_end) * 0.5;
        if self.hit(self.start, p) && (!self.hit(self.end, p) || p.x < mid) {
            Grab { what: 1, offset: x_start - p.x }
        } else if self.hit(self.end, p) {
            Grab { what: 2, offset: x_end - p.x }
        } else {
            Grab { what: 0, offset: 0.0 }
        }
    }
    fn paint(&self, p: &egui::Painter) {
        for (r, active, left) in [(self.start, self.over_start, true), (self.end, self.over_end, false)] {
            let c = if active { Color32::WHITE } else { ACCENT };
            let bar = if left {
                Rect::from_min_max(pos2(r.left() + 2.0, r.top()), pos2(r.left() + 5.0, r.bottom()))
            } else {
                Rect::from_min_max(pos2(r.right() - 5.0, r.top()), pos2(r.right() - 2.0, r.bottom()))
            };
            p.rect_filled(bar, 1.0, c);
            let (x0, x1) = if left { (bar.left(), r.right()) } else { (r.left(), bar.right()) };
            p.rect_filled(Rect::from_min_max(pos2(x0, r.top()), pos2(x1, r.top() + 3.0)), 1.0, c);
            p.rect_filled(Rect::from_min_max(pos2(x0, r.bottom() - 3.0), pos2(x1, r.bottom())), 1.0, c);
        }
    }
}

/// Press-time grab handling. Returns the active grab while the button is down on `resp`.
fn track_grab(ui: &Ui, resp: &egui::Response, b: &Brackets, x_start: f32, x_end: f32) -> Option<Grab> {
    let id = resp.id.with("grab");
    let down = resp.is_pointer_button_down_on();
    let mut g: Option<(u8, f32)> = ui.data(|d| d.get_temp(id));
    if down && g.is_none() {
        if let Some(o) = ui.input(|i| i.pointer.press_origin()).or(resp.interact_pointer_pos()) {
            let pick = b.pick(o, x_start, x_end);
            g = Some((pick.what, pick.offset));
            ui.data_mut(|d| d.insert_temp(id, (pick.what, pick.offset)));
        }
    }
    if !down {
        ui.data_mut(|d| d.remove::<(u8, f32)>(id));
    }
    if b.over_start || b.over_end || g.is_some_and(|g| g.0 != 0) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    g.map(|(what, offset)| Grab { what, offset })
}

/// Range slider of an animated parameter: "[" "]" limiter handles (the animation range) and the
/// current value (white). In pause mode a click / drag on the bar sets the value.
pub fn range_slider(ui: &mut Ui, a: &mut AnimParam, min: f32, max: f32, integer: bool) -> egui::Response {
    let (outer, mut resp) = ui.allocate_exact_size(vec2(ui.available_width().max(60.0), BAR_H), Sense::click_and_drag());
    let rect = outer.shrink2(vec2(2.0, 0.0));
    let span = (max - min).abs().max(1e-6);
    let to_x = |v: f32| rect.left() + ((v - min) / span).clamp(0.0, 1.0) * rect.width();
    let to_v = |x: f32| {
        let v = min + ((x - rect.left()) / rect.width()).clamp(0.0, 1.0) * span;
        if integer { v.round() } else { v }
    };
    if a.lo > a.hi {
        std::mem::swap(&mut a.lo, &mut a.hi);
    }
    let (xs, xe) = (to_x(a.lo), to_x(a.hi));
    let b = Brackets::new(outer, xs, xe, resp.hover_pos());
    let grab = track_grab(ui, &resp, &b, xs, xe);
    if let (Some(g), Some(pp)) = (grab.or(if resp.clicked() { Some(Grab { what: 0, offset: 0.0 }) } else { None }), resp.interact_pointer_pos()) {
        let v = to_v(pp.x + g.offset);
        match g.what {
            1 => a.lo = v.min(a.hi),
            2 => a.hi = v.max(a.lo),
            _ if a.mode == PlayMode::Pause => {
                a.value = v;
                // keep the animation phase in sync with a hand-set value
                let r = a.hi - a.lo;
                if r.abs() > 1e-6 {
                    a.phase = ((v - a.lo) / r).clamp(0.0, 1.0);
                }
            }
            _ => {}
        }
        resp.mark_changed();
    }
    let p = ui.painter();
    let track = rect.shrink2(vec2(0.0, 3.0));
    p.rect_filled(track, 2.0, ui.visuals().extreme_bg_color);
    p.rect_filled(Rect::from_min_max(pos2(xs, track.top()), pos2(xe, track.bottom())), 0.0, Color32::from_gray(62));
    b.paint(p);
    let vx = to_x(a.value);
    p.line_segment([pos2(vx, track.top()), pos2(vx, track.bottom())], Stroke::new(2.0, Color32::WHITE));
    let txt = if integer { format!("{:.0}", a.value) } else { format!("{:.2}", a.value) };
    p.text(rect.right_center() - vec2(HANDLE_W + 6.0, 0.0), egui::Align2::RIGHT_CENTER, txt, egui::FontId::monospace(10.0), ui.visuals().weak_text_color());
    resp
}

/// Play mode, loop mode, speed (free/bpm) and ½x/2x in one compact row.
pub fn transport_row(ui: &mut Ui, mode: &mut PlayMode, loop_mode: &mut LoopMode, speed: &mut Speed, free_label: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for m in PlayMode::ALL {
            if choice(ui, *mode == m, m.icon()).on_hover_text(m.label()).clicked() {
                *mode = m;
            }
        }
        ui.add_space(4.0);
        for l in LoopMode::ALL {
            if choice(ui, *loop_mode == l, l.label()).clicked() {
                *loop_mode = l;
            }
        }
        ui.add_space(4.0);
        speed_widget(ui, speed, free_label);
    });
}

pub fn speed_widget(ui: &mut Ui, speed: &mut Speed, free_label: &str) {
    if choice(ui, speed.bpm_mode, "bpm").on_hover_text("speed follows the global bpm (beats per part / cycle; also jumpcut interval)").clicked() {
        speed.bpm_mode = !speed.bpm_mode;
    }
    if speed.bpm_mode {
        ui.add(egui::DragValue::new(&mut speed.beats).range(0.25..=64.0).speed(0.05).suffix(" beats"));
    } else {
        ui.add(egui::DragValue::new(&mut speed.free).range(0.01..=16.0).speed(0.01).suffix(free_label));
    }
    if btn(ui, "½x").clicked() {
        speed.half();
    }
    if btn(ui, "2x").clicked() {
        speed.double();
    }
}

/// Timeline with limiters and position marker. Returns Some(norm) if the user scrubbed.
/// The limiters are the same bracket handles ([ ]) as on the range sliders.
pub fn timeline(ui: &mut Ui, t: &mut ClipTransport, position: f64, duration: Option<f64>) -> Option<f32> {
    let (outer, resp) = ui.allocate_exact_size(vec2(ui.available_width().max(60.0), BAR_H), Sense::click_and_drag());
    let rect = outer.shrink2(vec2(2.0, 0.0));
    let to_x = |n: f32| rect.left() + n.clamp(0.0, 1.0) * rect.width();
    let to_n = |x: f32| ((x - rect.left()) / rect.width()).clamp(0.0, 1.0);
    let (xs, xe) = (to_x(t.in_pt), to_x(t.out_pt));
    let b = Brackets::new(outer, xs, xe, resp.hover_pos());
    let grab = track_grab(ui, &resp, &b, xs, xe);
    let mut scrub = None;
    if let (Some(g), Some(pp)) = (grab, resp.interact_pointer_pos()) {
        match g.what {
            1 => t.in_pt = to_n(pp.x + g.offset).min(t.out_pt - 0.01),
            2 => t.out_pt = to_n(pp.x + g.offset).max(t.in_pt + 0.01),
            _ => {
                if resp.dragged() || resp.clicked() {
                    scrub = Some(to_n(pp.x));
                }
            }
        }
    } else if resp.clicked() {
        if let Some(pp) = resp.interact_pointer_pos() {
            if b.pick(pp, xs, xe).what == 0 {
                scrub = Some(to_n(pp.x));
            }
        }
    }
    let p = ui.painter();
    let track = rect.shrink2(vec2(0.0, 3.0));
    p.rect_filled(track, 2.0, ui.visuals().extreme_bg_color);
    // limited part: neutral grey (colors are reserved for the layer color-code)
    p.rect_filled(Rect::from_min_max(pos2(xs, track.top()), pos2(xe, track.bottom())), 0.0, Color32::from_gray(62));
    b.paint(p);
    if let Some(d) = duration {
        let x = to_x((position / d.max(1e-3)) as f32);
        p.line_segment([pos2(x, track.top()), pos2(x, track.bottom())], Stroke::new(2.0, Color32::WHITE));
        p.text(
            rect.right_center() - vec2(HANDLE_W + 6.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            format!("{:.1}/{:.1}s", position, d),
            egui::FontId::monospace(10.0),
            ui.visuals().weak_text_color(),
        );
    } else {
        p.text(rect.center(), egui::Align2::CENTER_CENTER, "loading…", egui::FontId::monospace(10.0), ui.visuals().weak_text_color());
    }
    scrub
}

/// One cell of the mix button grid (all cells same size, accent when on).
fn cell(ui: &mut Ui, on: bool, text: &str) -> egui::Response {
    let fill = if on { ACCENT } else { ui.visuals().widgets.inactive.weak_bg_fill };
    let color = if on { readable_on(ACCENT) } else { ui.visuals().text_color() };
    ui.add_sized(BTN, egui::Button::new(egui::RichText::new(text).color(color).size(BTN_TEXT)).fill(fill))
}

/// Right margin of the "Ch R G B" group, keeps it away from the × of effect rows / layers.
const MIX_RIGHT_MARGIN: f32 = 28.0;

/// mix-type, r/g/b channel and mix percentage, as buttons like resolume 2:
///   A  Lk  iLk  Ad          Ch  R  G  B
///   mix ───────────────────────── %
pub fn mix_controls(ui: &mut Ui, mix: &mut Mix, id_salt: impl std::hash::Hash + std::fmt::Debug) {
    ui.push_id(id_salt, |ui| {
        let gap = 2.0;
        let mix_w = 4.0 * (BTN.x + gap);
        let ch_w = 22.0 + 3.0 * (BTN.x + gap) + MIX_RIGHT_MARGIN;
        // not enough room for both groups on one line -> "Ch R G B" goes to its own line
        let one_line = ui.available_width() >= mix_w + ch_w + 12.0;
        let ch_group = |ui: &mut Ui, mix: &mut Mix| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing = vec2(gap, gap);
                ui.add_space(MIX_RIGHT_MARGIN);
                for i in (0..3).rev() {
                    if cell(ui, mix.rgb[i], ["R", "G", "B"][i]).on_hover_text("channel on/off").clicked() {
                        mix.rgb[i] = !mix.rgb[i];
                    }
                }
                ui.add_space(2.0);
                ui.label(egui::RichText::new("Ch").weak());
            });
        };
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, gap);
            for m in MixType::ALL {
                if cell(ui, mix.mix_type == m, m.short()).on_hover_text(m.label()).clicked() {
                    mix.mix_type = m;
                }
            }
            if one_line {
                ch_group(ui, mix);
            }
        });
        if !one_line {
            ui.horizontal(|ui| ch_group(ui, mix));
        }
        percent_slider(ui, &mut mix.amount, "mix");
    });
}

/// Paint a GL texture (bottom-up) into `rect`, keeping aspect `aspect` (letterbox).
pub fn paint_tex(ui: &Ui, rect: Rect, tex: Option<egui::TextureId>, aspect: f32) {
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 0.0, Color32::BLACK);
    if let Some(id) = tex {
        let r = fit(rect, aspect);
        p.image(id, r, Rect::from_min_max(pos2(0.0, 1.0), pos2(1.0, 0.0)), Color32::WHITE);
    }
}

pub fn fit(rect: Rect, aspect: f32) -> Rect {
    let ra = rect.width() / rect.height().max(1.0);
    let size = if ra > aspect { vec2(rect.height() * aspect, rect.height()) } else { vec2(rect.width(), rect.width() / aspect) };
    Rect::from_center_size(rect.center(), size)
}

/// Rectangle frame around a rect.
pub fn outline(ui: &Ui, rect: Rect, width: f32, color: Color32) {
    ui.painter().rect_stroke(rect, 2.0, Stroke::new(width, color), StrokeKind::Inside);
}
