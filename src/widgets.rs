//! Custom widgets: range slider, animation controls, clip timeline, small toggles.

use crate::state::{AnimParam, ClipTransport, LoopMode, Mix, MixType, PlayMode, Speed};
use egui::{pos2, vec2, Color32, Rect, Sense, Stroke, StrokeKind, Ui};

pub const ACCENT: Color32 = Color32::from_rgb(230, 40, 140);

/// Small toggle button that lights up in `on_color` when active.
pub fn toggle(ui: &mut Ui, on: &mut bool, label: &str, on_color: Color32) -> egui::Response {
    let fill = if *on { on_color } else { ui.visuals().widgets.inactive.weak_bg_fill };
    let text_color = if *on { Color32::BLACK } else { ui.visuals().text_color() };
    let r = ui.add(egui::Button::new(egui::RichText::new(label).color(text_color)).fill(fill).small());
    if r.clicked() {
        *on = !*on;
    }
    r
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
        if let Some(p) = resp.interact_pointer_pos() {
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

/// Timeline with limiters and position marker. Returns Some(norm) if the user scrubbed.
pub fn timeline(ui: &mut Ui, t: &mut ClipTransport, position: f64, duration: Option<f64>) -> Option<f32> {
    let h = 18.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width().max(60.0), h), Sense::click_and_drag());
    let to_x = |n: f32| rect.left() + n.clamp(0.0, 1.0) * rect.width();
    let to_n = |x: f32| ((x - rect.left()) / rect.width()).clamp(0.0, 1.0);
    let id = resp.id.with("tl");
    let mut scrub = None;
    if resp.drag_started() {
        if let Some(p) = resp.interact_pointer_pos() {
            let di = (p.x - to_x(t.in_pt)).abs();
            let d_o = (p.x - to_x(t.out_pt)).abs();
            let which: u8 = if di < 7.0 && di <= d_o {
                1
            } else if d_o < 7.0 {
                2
            } else {
                0
            };
            ui.data_mut(|d| d.insert_temp(id, which));
        }
    }
    if resp.dragged() {
        if let Some(p) = resp.interact_pointer_pos() {
            match ui.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) {
                1 => t.in_pt = to_n(p.x).min(t.out_pt - 0.01),
                2 => t.out_pt = to_n(p.x).max(t.in_pt + 0.01),
                _ => scrub = Some(to_n(p.x)),
            }
        }
    } else if resp.clicked() {
        if let Some(p) = resp.interact_pointer_pos() {
            scrub = Some(to_n(p.x));
        }
    }
    let p = ui.painter();
    p.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
    let lim = Rect::from_min_max(pos2(to_x(t.in_pt), rect.top()), pos2(to_x(t.out_pt), rect.bottom()));
    p.rect_filled(lim, 2.0, Color32::from_rgb(40, 90, 40));
    for x in [to_x(t.in_pt), to_x(t.out_pt)] {
        p.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], Stroke::new(2.0, ACCENT));
    }
    if let Some(d) = duration {
        let n = (position / d.max(1e-3)) as f32;
        let x = to_x(n);
        p.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], Stroke::new(2.0, Color32::WHITE));
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

/// mix-type, mix percentage, r/g/b
pub fn mix_controls(ui: &mut Ui, mix: &mut Mix, id_salt: impl std::hash::Hash + std::fmt::Debug) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        egui::ComboBox::from_id_salt(id_salt)
            .width(78.0)
            .selected_text(mix.mix_type.label())
            .show_ui(ui, |ui| {
                for m in MixType::ALL {
                    ui.selectable_value(&mut mix.mix_type, m, m.label());
                }
            });
        toggle(ui, &mut mix.rgb[0], "R", Color32::from_rgb(230, 60, 60));
        toggle(ui, &mut mix.rgb[1], "G", Color32::from_rgb(60, 200, 90));
        toggle(ui, &mut mix.rgb[2], "B", Color32::from_rgb(70, 120, 255));
    });
    ui.add(egui::Slider::new(&mut mix.amount, 0.0..=1.0).show_value(true).text("mix"));
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
