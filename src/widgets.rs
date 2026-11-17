//! Custom widgets: range slider, animation controls, clip timeline, small toggles.

use crate::state::{AnimParam, ClipTransport, LoopMode, Mix, MixType, PlayMode, Speed};
use egui::{pos2, vec2, Color32, Rect, Sense, Stroke, StrokeKind, Ui};

pub const ACCENT: Color32 = Color32::from_rgb(230, 40, 140);

/// 0..1 value shown and edited as integer percent 0..100.
pub fn percent_slider(ui: &mut Ui, v: &mut f32, label: &str) -> egui::Response {
    let mut pct = (*v * 100.0).round();
    let r = ui.add(egui::Slider::new(&mut pct, 0.0..=100.0).step_by(1.0).fixed_decimals(0).suffix(" %").text(label));
    if r.changed() {
        *v = pct / 100.0;
    }
    r
}

/// Small toggle button that lights up in `on_color` when active.
pub fn toggle(ui: &mut Ui, on: &mut bool, label: &str, on_color: Color32) -> egui::Response {
    let fill = if *on { on_color } else { ui.visuals().widgets.inactive.weak_bg_fill };
    let text_color = if *on { readable_on(on_color) } else { ui.visuals().text_color() };
    let r = ui.add(egui::Button::new(egui::RichText::new(label).color(text_color)).fill(fill).small());
    if r.clicked() {
        *on = !*on;
    }
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
    let text_color = if selected { Color32::BLACK } else { ui.visuals().text_color() };
    ui.add(egui::Button::new(egui::RichText::new(label).color(text_color)).fill(fill).small())
}

pub fn layer_color(i: usize) -> Color32 {
    let c = crate::state::LAYER_COLORS[i % crate::state::NUM_LAYERS];
    Color32::from_rgb(c[0], c[1], c[2])
}

#[derive(Clone, Copy, PartialEq)]
enum Handle {
    Lo,
    Hi,
    Value,
}

/// Range slider: lo/hi limiter handles, current value marker. In pause mode dragging sets the value.
pub fn range_slider(ui: &mut Ui, a: &mut AnimParam, min: f32, max: f32, integer: bool) -> egui::Response {
    let h = 16.0;
    let (rect, mut resp) = ui.allocate_exact_size(vec2(ui.available_width().max(60.0), h), Sense::click_and_drag());
    let span = (max - min).abs().max(1e-6);
    let to_x = |v: f32| rect.left() + ((v - min) / span).clamp(0.0, 1.0) * rect.width();
    let to_v = |x: f32| {
        let v = min + ((x - rect.left()) / rect.width()).clamp(0.0, 1.0) * span;
        if integer {
            v.round()
        } else {
            v
        }
    };
    let id = resp.id.with("handle");
    if resp.drag_started() {
        // use the press position: when egui reports "drag started" the pointer has already moved
        if let Some(p) = ui.input(|i| i.pointer.press_origin()).or(resp.interact_pointer_pos()) {
            let dl = (p.x - to_x(a.lo)).abs();
            let dh = (p.x - to_x(a.hi)).abs();
            let handle = if a.mode == PlayMode::Pause && dl > 6.0 && dh > 6.0 {
                Handle::Value
            } else if dl <= dh {
                Handle::Lo
            } else {
                Handle::Hi
            };
            ui.data_mut(|d| d.insert_temp(id, handle as u8));
        }
    }
    if resp.dragged() || resp.clicked() {
        if let Some(p) = resp.interact_pointer_pos() {
            let handle: u8 = if resp.clicked() {
                if a.mode == PlayMode::Pause { Handle::Value as u8 } else { 255 }
            } else {
                ui.data(|d| d.get_temp(id)).unwrap_or(Handle::Value as u8)
            };
            let v = to_v(p.x);
            match handle {
                x if x == Handle::Lo as u8 => a.lo = v,
                x if x == Handle::Hi as u8 => a.hi = v,
                x if x == Handle::Value as u8 => {
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
    }
    let p = ui.painter();
    let bg = ui.visuals().extreme_bg_color;
    p.rect_filled(rect, 3.0, bg);
    let (l, r) = (to_x(a.lo.min(a.hi)), to_x(a.lo.max(a.hi)));
    let range_rect = Rect::from_min_max(pos2(l, rect.top() + 3.0), pos2(r, rect.bottom() - 3.0));
    p.rect_filled(range_rect, 2.0, ACCENT.gamma_multiply(0.35));
    for x in [to_x(a.lo), to_x(a.hi)] {
        p.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], Stroke::new(2.0, ACCENT));
    }
    let vx = to_x(a.value);
    p.line_segment([pos2(vx, rect.top()), pos2(vx, rect.bottom())], Stroke::new(2.0, Color32::WHITE));
    let txt = if integer { format!("{:.0}", a.value) } else { format!("{:.2}", a.value) };
    p.text(
        rect.right_center() - vec2(4.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        txt,
        egui::FontId::monospace(10.0),
        ui.visuals().weak_text_color(),
    );
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
    if ui.small_button("½x").clicked() {
        speed.half();
    }
    if ui.small_button("2x").clicked() {
        speed.double();
    }
}

/// Width of the grab handles of the timeline limiters.
const HANDLE_W: f32 = 9.0;

/// Timeline with limiters and position marker. Returns Some(norm) if the user scrubbed.
/// The limiters are wide bracket-shaped handles ([ ]) that enclose the limited part, so they are
/// easy to catch.
pub fn timeline(ui: &mut Ui, t: &mut ClipTransport, position: f64, duration: Option<f64>) -> Option<f32> {
    let h = 22.0;
    let (outer, resp) = ui.allocate_exact_size(vec2(ui.available_width().max(60.0), h), Sense::click_and_drag());
    let rect = outer.shrink2(vec2(2.0, 0.0));
    let to_x = |n: f32| rect.left() + n.clamp(0.0, 1.0) * rect.width();
    let to_n = |x: f32| ((x - rect.left()) / rect.width()).clamp(0.0, 1.0);
    let in_handle = Rect::from_min_max(pos2(to_x(t.in_pt) - 2.0, outer.top()), pos2(to_x(t.in_pt) + HANDLE_W, outer.bottom()));
    let out_handle = Rect::from_min_max(pos2(to_x(t.out_pt) - HANDLE_W, outer.top()), pos2(to_x(t.out_pt) + 2.0, outer.bottom()));
    let grab = |r: Rect, p: egui::Pos2| r.expand2(vec2(4.0, 0.0)).contains(p);
    let id = resp.id.with("tl");
    let mut scrub = None;
    let hover = resp.hover_pos();
    let over_in = hover.is_some_and(|p| grab(in_handle, p));
    let over_out = hover.is_some_and(|p| grab(out_handle, p));
    if over_in || over_out {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    // Decide what is grabbed at the moment of the press (press origin), not when egui reports
    // "drag started": by then the pointer has already moved a few pixels off the handle.
    // 0 = scrub, 1 = in-handle, 2 = out-handle
    let down = resp.is_pointer_button_down_on();
    // (what, grab offset in px between handle point and press position)
    let mut which: Option<(u8, f32)> = ui.data(|d| d.get_temp(id));
    if down && which.is_none() {
        if let Some(o) = ui.input(|i| i.pointer.press_origin()).or(resp.interact_pointer_pos()) {
            // both handles under the pointer (very short range): the nearer one wins
            let mid = (to_x(t.in_pt) + to_x(t.out_pt)) * 0.5;
            let w = if grab(in_handle, o) && (!grab(out_handle, o) || o.x < mid) {
                1
            } else if grab(out_handle, o) {
                2
            } else {
                0
            };
            let off = match w {
                1 => to_x(t.in_pt) - o.x,
                2 => to_x(t.out_pt) - o.x,
                _ => 0.0,
            };
            which = Some((w, off));
            ui.data_mut(|d| d.insert_temp(id, (w, off)));
        }
    }
    if let (Some((w, off)), Some(pp)) = (which, resp.interact_pointer_pos()) {
        match w {
            1 => t.in_pt = to_n(pp.x + off).min(t.out_pt - 0.01),
            2 => t.out_pt = to_n(pp.x + off).max(t.in_pt + 0.01),
            _ => {
                if resp.dragged() || resp.clicked() {
                    scrub = Some(to_n(pp.x));
                }
            }
        }
    }
    if !down {
        ui.data_mut(|d| d.remove::<(u8, f32)>(id));
    }
    if which.is_some_and(|w| w.0 != 0) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    let p = ui.painter();
    let track = rect.shrink2(vec2(0.0, 3.0));
    p.rect_filled(track, 2.0, ui.visuals().extreme_bg_color);
    // limited part: neutral grey (colors are reserved for the layer color-code)
    let lim = Rect::from_min_max(pos2(to_x(t.in_pt), track.top()), pos2(to_x(t.out_pt), track.bottom()));
    p.rect_filled(lim, 0.0, Color32::from_gray(62));
    // bracket handles
    for (r, active, left) in [(in_handle, over_in, true), (out_handle, over_out, false)] {
        let c = if active { Color32::WHITE } else { ACCENT };
        // "[" at the in-point, "]" at the out-point
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
    if let Some(d) = duration {
        let n = (position / d.max(1e-3)) as f32;
        let x = to_x(n);
        p.line_segment([pos2(x, track.top()), pos2(x, track.bottom())], Stroke::new(2.0, Color32::WHITE));
        p.text(
            rect.right_center() - vec2(4.0, 0.0),
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
    let color = if on { Color32::BLACK } else { ui.visuals().text_color() };
    ui.add_sized(vec2(30.0, 18.0), egui::Button::new(egui::RichText::new(text).color(color).size(12.0)).fill(fill))
}

/// Right margin of the "Ch R G B" group, keeps it away from the × of effect rows / layers.
const MIX_RIGHT_MARGIN: f32 = 28.0;

/// mix-type, r/g/b channel and mix percentage, as buttons like resolume 2:
///   A  Lk  iLk  Ad          Ch  R  G  B
///   mix ───────────────────────── %
pub fn mix_controls(ui: &mut Ui, mix: &mut Mix, id_salt: impl std::hash::Hash + std::fmt::Debug) {
    ui.push_id(id_salt, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
            for m in MixType::ALL {
                if cell(ui, mix.mix_type == m, m.short()).on_hover_text(m.label()).clicked() {
                    mix.mix_type = m;
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(MIX_RIGHT_MARGIN);
                for i in (0..3).rev() {
                    if cell(ui, mix.rgb[i], ["R", "G", "B"][i]).on_hover_text("channel on/off").clicked() {
                        mix.rgb[i] = !mix.rgb[i];
                    }
                }
                ui.add_space(2.0);
                ui.label(egui::RichText::new("Ch").weak());
            });
        });
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
