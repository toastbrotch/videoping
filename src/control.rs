//! Control registry: every controllable thing in videoping has a stable address, a kind (trigger,
//! toggle, range, choice) and can be read and set. MIDI (Bhoreal, Twister, MIDI learn), MCP and
//! later OSC all sit on top of this, so they control exactly the same things.
//!
//! Addresses (numbers count from 1, like the ui shows them):
//!
//! ```text
//! /master/blackout  /master/freeze           toggle
//! /master/brightness                         range 0..1
//! /bpm                                       range 20..300 (whole bpm)
//! /bpm/tap  /bpm/half  /bpm/double           trigger
//! /record                                    toggle (needs the app: Effect::Record)
//! /page                                      range 1..pages (whole numbers)
//! /layer/N/select                            trigger, reads "is selected"
//! /layer/N/active  /layer/N/r|g|b            toggle
//! /layer/N/mix                               range 0..1
//! /layer/N/mixtype                           choice A Lk iLk Ad
//! /layer/N/unload                            trigger, reads "something loaded"
//! /slot/N/load                               trigger: into the selected layer
//! /slot/N/select                             trigger, reads "is selected"
//! /slot/N/state                              read only: empty | loaded | L1 | L2 | L3
//! /slot/N/fx/K/active|mix|mixtype            effect K of the instance in slot N
//! /slot/N/fx/K/param/P[/x|/y]                its parameter P (counted top-down like the ui)
//! /slot/N/param/P[/x|/y]                     parameter P of the generator in slot N
//! /page/slot/N/...                           slot N (1..32) of the page shown (controllers
//!                                            follow the page)
//! /layer/N/fx/K/...  /layer/N/param/P        the same for whatever is loaded in layer N
//! /out/fx/K/...                              output effect K
//! /sel/param/P[/x|/y]   /sel/param/P/reset    parameter P of the focus in the selected instance:
//!                                            the effect opened last (focus_fx), else the
//!                                            generator, else effect 1; reset = isf default
//! /sel/fx/K/...                              effect K of the selected instance
//! .../param/P/reset                          any parameter back to its isf default (trigger)
//!
//! Slots count like the grid: top-down per column, 32 per page (page 2 starts at 33). A binding
//! to a slot controls whatever is in that slot. Parameters are numbered in the order the ui
//! shows them (the ISF header order, image inputs left out); colors and events keep their number
//! but are not controllable yet.
//! ```
//!
//! Parameter kinds: float / long -> range (MIN..MAX of the ISF header, long rounded), bool ->
//! toggle, point2D -> /x and /y ranges.

use crate::isf::{InputKind, IsfInfo};
use crate::state::{EffectInst, MixType, Param, ParamValue, Session, SourceKind, NUM_LAYERS, SLOTS_PER_PAGE};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// fires an action; may still read a state (e.g. "is selected")
    Trigger,
    Toggle,
    Range { min: f32, max: f32, integer: bool },
    Choice(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Desc {
    pub addr: String,
    pub label: String,
    pub kind: Kind,
    pub writable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Num(f32),
    Text(String),
}

impl Value {
    fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::Num(n) => Some(*n >= 0.5),
            Value::Text(t) => match t.as_str() {
                "true" | "on" | "1" => Some(true),
                "false" | "off" | "0" => Some(false),
                _ => None,
            },
        }
    }
    fn as_num(&self) -> Option<f32> {
        match self {
            Value::Num(n) => Some(*n),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::Text(t) => t.parse().ok(),
        }
    }
}

/// Things a set needs the running app for (beat clock, renderer).
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    None,
    Tap,
    Record(bool),
}

/// State that lives outside the session.
#[derive(Clone, Copy, Debug, Default)]
pub struct Ext {
    pub recording: bool,
}

pub type Infos = HashMap<PathBuf, IsfInfo>;

const MIXTYPES: [&str; 4] = ["A", "Lk", "iLk", "Ad"];

fn mixtype_name(m: MixType) -> &'static str {
    MIXTYPES[m.shader_id() as usize]
}

fn mixtype_from(t: &str) -> Option<MixType> {
    MIXTYPES.iter().position(|m| m.eq_ignore_ascii_case(t)).map(|i| MixType::ALL[i])
}

fn choice(items: &[&str]) -> Kind {
    Kind::Choice(items.iter().map(|s| s.to_string()).collect())
}

fn d(addr: String, label: &str, kind: Kind) -> Desc {
    Desc { addr, label: label.to_string(), kind, writable: true }
}

// ---------------------------------------------------------------------------------- list

/// Every address that exists right now (depends on the session: slots, layers, effects).
pub fn list(s: &Session, infos: &Infos) -> Vec<Desc> {
    let mut v = vec![
        d("/master/blackout".into(), "blackout", Kind::Toggle),
        d("/master/freeze".into(), "freeze", Kind::Toggle),
        d("/master/brightness".into(), "brightness", Kind::Range { min: 0.0, max: 1.0, integer: false }),
        d("/bpm".into(), "bpm", Kind::Range { min: 20.0, max: 300.0, integer: true }),
        d("/bpm/tap".into(), "tap", Kind::Trigger),
        d("/bpm/half".into(), "bpm ½x", Kind::Trigger),
        d("/bpm/double".into(), "bpm 2x", Kind::Trigger),
        d("/record".into(), "record", Kind::Toggle),
        d("/page".into(), "prepare page", Kind::Range { min: 1.0, max: s.pages() as f32, integer: true }),
    ];
    for n in 1..=NUM_LAYERS {
        let p = format!("/layer/{n}");
        v.push(d(format!("{p}/select"), &format!("select layer {n}"), Kind::Trigger));
        v.push(d(format!("{p}/active"), &format!("layer {n} on"), Kind::Toggle));
        v.push(d(format!("{p}/mix"), &format!("layer {n} mix"), Kind::Range { min: 0.0, max: 1.0, integer: false }));
        v.push(d(format!("{p}/mixtype"), &format!("layer {n} mix-type"), choice(&MIXTYPES)));
        for c in ["r", "g", "b"] {
            v.push(d(format!("{p}/{c}"), &format!("layer {n} {}", c.to_uppercase()), Kind::Toggle));
        }
        v.push(d(format!("{p}/unload"), &format!("unload layer {n}"), Kind::Trigger));
        // aliases for what is loaded in the layer (stable for a controller, whatever is loaded)
        if let Some(inst) = s.layers[n - 1].instance.and_then(|id| s.find(id)) {
            list_instance(&mut v, &p, &format!("layer {n} ({})", inst.name), inst, infos);
        }
    }
    for (i, slot) in s.slots.iter().enumerate() {
        let n = i + 1;
        v.push(d(format!("/slot/{n}/load"), &format!("load slot {n}"), Kind::Trigger));
        v.push(d(format!("/slot/{n}/select"), &format!("select slot {n}"), Kind::Trigger));
        let mut st = d(format!("/slot/{n}/state"), &format!("slot {n}"), choice(&["empty", "loaded", "L1", "L2", "L3"]));
        st.writable = false;
        v.push(st);
        if let Some(inst) = slot {
            list_instance(&mut v, &format!("/slot/{n}"), &format!("slot {n} ({})", inst.name), inst, infos);
        }
    }
    for n in 1..=SLOTS_PER_PAGE {
        v.push(d(format!("/page/slot/{n}/load"), &format!("load slot {n} of the page"), Kind::Trigger));
        v.push(d(format!("/page/slot/{n}/select"), &format!("select slot {n} of the page"), Kind::Trigger));
        let mut st = d(format!("/page/slot/{n}/state"), &format!("slot {n} of the page"), choice(&["empty", "loaded", "L1", "L2", "L3"]));
        st.writable = false;
        v.push(st);
    }
    for (k, fx) in s.output_effects.iter().enumerate() {
        list_fx(&mut v, &format!("/out/fx/{}", k + 1), "output", fx, infos);
    }
    // the selected instance's focus: the same parameters once more under /sel/param, with reset
    if let Some(prefix) = sel_param_prefix(s) {
        let pre = format!("{prefix}/");
        let sel: Vec<Desc> = v
            .iter()
            .filter(|d| d.addr.starts_with(&pre))
            .map(|d| Desc { addr: format!("/sel/param/{}", &d.addr[pre.len()..]), label: format!("selected · {}", d.label), kind: d.kind.clone(), writable: d.writable })
            .collect();
        for d in sel {
            let is_xy = d.addr.ends_with("/x") || d.addr.ends_with("/y");
            if !is_xy {
                v.push(Desc { addr: format!("{}/reset", d.addr), label: format!("{} reset", d.label), kind: Kind::Trigger, writable: true });
            }
            v.push(d);
        }
    }
    v
}

/// "/page/slot/N/x" -> "/slot/<page * 32 + N>/x"
pub fn page_alias(s: &Session, addr: &str) -> Option<String> {
    match parts(addr).as_slice() {
        ["page", "slot", n, rest @ ..] if !rest.is_empty() => {
            let n = n.parse::<usize>().ok().filter(|n| (1..=SLOTS_PER_PAGE).contains(n))?;
            Some(format!("/slot/{}/{}", s.page * SLOTS_PER_PAGE + n, rest.join("/")))
        }
        _ => None,
    }
}

/// Where /sel/param points to right now: "/slot/N/fx/K/param" or "/slot/N/param".
pub fn sel_param_prefix(s: &Session) -> Option<String> {
    let id = s.selected_instance?;
    let slot = s.slot_of(id)? + 1;
    let inst = s.find(id)?;
    if let Some(k) = s.focus_fx.and_then(|uid| inst.effects.iter().position(|e| e.uid == uid)) {
        return Some(format!("/slot/{slot}/fx/{}/param", k + 1));
    }
    if matches!(inst.source, SourceKind::Generator { .. }) {
        return Some(format!("/slot/{slot}/param"));
    }
    (!inst.effects.is_empty()).then(|| format!("/slot/{slot}/fx/1/param"))
}

/// "/sel/param/..." and "/sel/fx/..." -> the absolute slot address of the selected instance
fn sel_alias(s: &Session, addr: &str) -> Option<String> {
    match parts(addr).as_slice() {
        ["sel", "param", rest @ ..] if !rest.is_empty() => Some(format!("{}/{}", sel_param_prefix(s)?, rest.join("/"))),
        ["sel", "fx", rest @ ..] if !rest.is_empty() => {
            let slot = s.slot_of(s.selected_instance?)? + 1;
            Some(format!("/slot/{slot}/fx/{}", rest.join("/")))
        }
        _ => None,
    }
}

/// The page and selection aliases resolved (other addresses as they are).
pub fn resolve(s: &Session, addr: &str) -> String {
    sel_alias(s, addr).or_else(|| page_alias(s, addr)).unwrap_or_else(|| addr.to_string())
}

fn list_instance(v: &mut Vec<Desc>, prefix: &str, owner: &str, inst: &crate::state::Instance, infos: &Infos) {
    if let SourceKind::Generator { path, params } = &inst.source {
        if let Some(info) = infos.get(path) {
            list_params(v, &format!("{prefix}/param"), owner, params, info);
        }
    }
    for (k, fx) in inst.effects.iter().enumerate() {
        list_fx(v, &format!("{prefix}/fx/{}", k + 1), owner, fx, infos);
    }
}

fn list_fx(v: &mut Vec<Desc>, prefix: &str, owner: &str, fx: &EffectInst, infos: &Infos) {
    let name = format!("{owner} · {}", crate::isf::display_name(&fx.path));
    v.push(d(format!("{prefix}/active"), &format!("{name} on"), Kind::Toggle));
    v.push(d(format!("{prefix}/mix"), &format!("{name} mix"), Kind::Range { min: 0.0, max: 1.0, integer: false }));
    v.push(d(format!("{prefix}/mixtype"), &format!("{name} mix-type"), choice(&MIXTYPES)));
    if let Some(info) = infos.get(&fx.path) {
        list_params(v, &format!("{prefix}/param"), &name, &fx.params, info);
    }
}

/// Parameter number (1-based) -> ISF input, counted like the ui shows them (no image inputs).
fn nth_input(info: &IsfInfo, n: usize) -> Option<&crate::isf::Input> {
    info.inputs.iter().filter(|i| !matches!(i.kind, InputKind::Image | InputKind::Audio)).nth(n.checked_sub(1)?)
}

fn list_params(v: &mut Vec<Desc>, prefix: &str, owner: &str, params: &[Param], info: &IsfInfo) {
    let shown = info.inputs.iter().filter(|i| !matches!(i.kind, InputKind::Image | InputKind::Audio));
    for (k, inp) in shown.enumerate() {
        if !params.iter().any(|p| p.name == inp.name) {
            continue;
        }
        let a = format!("{prefix}/{}", k + 1);
        let label = format!("{owner} · {}", inp.label);
        match &inp.kind {
            InputKind::Float { min, max, .. } => v.push(d(a, &label, Kind::Range { min: *min, max: *max, integer: false })),
            InputKind::Long { min, max, .. } => v.push(d(a, &label, Kind::Range { min: *min, max: *max, integer: true })),
            InputKind::Bool { .. } => v.push(d(a, &label, Kind::Toggle)),
            InputKind::Point2D { min, max, .. } => {
                v.push(d(format!("{a}/x"), &format!("{label} x"), Kind::Range { min: min[0], max: max[0], integer: false }));
                v.push(d(format!("{a}/y"), &format!("{label} y"), Kind::Range { min: min[1], max: max[1], integer: false }));
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------------- addressing

/// Where an effect / parameter address points to.
enum Owner {
    Inst(u64),
    Out,
}

fn parts(addr: &str) -> Vec<&str> {
    addr.trim().trim_matches('/').split('/').filter(|p| !p.is_empty()).collect()
}

fn num(p: Option<&&str>) -> Option<usize> {
    p.and_then(|x| x.parse::<usize>().ok())
}

/// "/layer/N/fx|param..." or "/slot/N/fx|param..." -> instance id and the rest after the prefix
fn resolve_owner<'a>(s: &Session, p: &'a [&'a str]) -> Option<(Owner, &'a [&'a str])> {
    match p {
        ["slot", n, rest @ ..] if matches!(rest.first(), Some(&"fx") | Some(&"param")) => {
            let si = n.parse::<usize>().ok()?.checked_sub(1)?;
            Some((Owner::Inst(s.slots.get(si)?.as_ref()?.id), rest))
        }
        ["layer", n, rest @ ..] if matches!(rest.first(), Some(&"fx") | Some(&"param")) => {
            let li = n.parse::<usize>().ok()?.checked_sub(1)?;
            Some((Owner::Inst(s.layers.get(li)?.instance?), rest))
        }
        ["out", rest @ ..] if rest.first() == Some(&"fx") => Some((Owner::Out, rest)),
        _ => None,
    }
}

fn effects_of<'a>(s: &'a Session, o: &Owner) -> Option<&'a Vec<EffectInst>> {
    match o {
        Owner::Inst(id) => s.find(*id).map(|i| &i.effects),
        Owner::Out => Some(&s.output_effects),
    }
}

fn effects_of_mut<'a>(s: &'a mut Session, o: &Owner) -> Option<&'a mut Vec<EffectInst>> {
    match o {
        Owner::Inst(id) => s.find_mut(*id).map(|i| &mut i.effects),
        Owner::Out => Some(&mut s.output_effects),
    }
}

// ---------------------------------------------------------------------------------- get

pub fn get(s: &Session, infos: &Infos, ext: Ext, addr: &str) -> Option<Value> {
    if let Some(a) = sel_alias(s, addr).or_else(|| page_alias(s, addr)) {
        return get(s, infos, ext, &a);
    }
    let p = parts(addr);
    match p.as_slice() {
        ["master", "blackout"] => Some(Value::Bool(s.master.blackout)),
        ["master", "freeze"] => Some(Value::Bool(s.master.freeze)),
        ["master", "brightness"] => Some(Value::Num(if s.master.blackout { 0.0 } else { s.master.brightness })),
        ["bpm"] => Some(Value::Num(s.bpm)),
        ["bpm", "tap" | "half" | "double"] => None,
        ["record"] => Some(Value::Bool(ext.recording)),
        ["page"] => Some(Value::Num((s.page + 1) as f32)),
        ["layer", n, field] => {
            let li = n.parse::<usize>().ok()?.checked_sub(1)?;
            let l = s.layers.get(li)?;
            match *field {
                "select" => Some(Value::Bool(s.selected_layer == li)),
                "active" => Some(Value::Bool(l.active)),
                "mix" => Some(Value::Num(l.mix.amount)),
                "mixtype" => Some(Value::Text(mixtype_name(l.mix.mix_type).into())),
                "r" => Some(Value::Bool(l.mix.rgb[0])),
                "g" => Some(Value::Bool(l.mix.rgb[1])),
                "b" => Some(Value::Bool(l.mix.rgb[2])),
                "unload" => Some(Value::Bool(l.instance.is_some())),
                _ => None,
            }
        }
        ["slot", n, field] => {
            let si = n.parse::<usize>().ok()?.checked_sub(1)?;
            let inst = s.slots.get(si)?.as_ref();
            match *field {
                "load" => Some(Value::Bool(inst.is_some_and(|i| s.live_layer(i.id).is_some()))),
                "select" => Some(Value::Bool(inst.is_some_and(|i| s.selected_instance == Some(i.id)))),
                "state" => Some(Value::Text(match inst {
                    None => "empty".into(),
                    Some(i) => s.live_layer(i.id).map(|l| format!("L{}", l + 1)).unwrap_or("loaded".into()),
                })),
                _ => None,
            }
        }
        _ => {
            let (owner, rest) = resolve_owner(s, &p)?;
            match rest {
                ["fx", k, field] => {
                    let fx = effects_of(s, &owner)?.get(num(Some(k))?.checked_sub(1)?)?;
                    match *field {
                        "active" => Some(Value::Bool(fx.active)),
                        "mix" => Some(Value::Num(fx.mix.amount)),
                        "mixtype" => Some(Value::Text(mixtype_name(fx.mix.mix_type).into())),
                        _ => None,
                    }
                }
                ["fx", k, "param", n, sub @ ..] => {
                    let fx = effects_of(s, &owner)?.get(num(Some(k))?.checked_sub(1)?)?;
                    let inp = nth_input(infos.get(&fx.path)?, n.parse().ok()?)?;
                    get_param(&fx.params, &inp.name, sub.first().copied())
                }
                ["param", n, sub @ ..] => {
                    let Owner::Inst(id) = owner else { return None };
                    match &s.find(id)?.source {
                        SourceKind::Generator { path, params } => {
                            let inp = nth_input(infos.get(path)?, n.parse().ok()?)?;
                            get_param(params, &inp.name, sub.first().copied())
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        }
    }
}

fn get_param(params: &[Param], name: &str, sub: Option<&str>) -> Option<Value> {
    let p = params.iter().find(|p| p.name == name)?;
    match (&p.value, sub) {
        (ParamValue::Anim(a), None) => Some(Value::Num(a.first()?.value)),
        (ParamValue::Anim(a), Some("x")) => Some(Value::Num(a.first()?.value)),
        (ParamValue::Anim(a), Some("y")) => Some(Value::Num(a.get(1)?.value)),
        (ParamValue::Bool(b), None) => Some(Value::Bool(*b)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------------- set

/// Set an address. Triggers ignore the value. Ranges are clamped to their limits.
pub fn set(s: &mut Session, infos: &Infos, addr: &str, value: &Value) -> Result<Effect, String> {
    if let Some(a) = sel_alias(s, addr).or_else(|| page_alias(s, addr)) {
        return set(s, infos, &a, value);
    }
    let p = parts(addr);
    let bad = || format!("unknown or read-only address: {addr}");
    let want_bool = || value.as_bool().ok_or_else(|| format!("{addr}: needs on / off"));
    let want_num = || value.as_num().ok_or_else(|| format!("{addr}: needs a number"));
    match p.as_slice() {
        ["master", "blackout"] => s.master.blackout = want_bool()?,
        ["master", "freeze"] => s.master.freeze = want_bool()?,
        ["master", "brightness"] => {
            // like the slider: pulling it up during a blackout ends the blackout (fade in)
            let v = want_num()?.clamp(0.0, 1.0);
            if s.master.blackout && v > 0.0 {
                s.master.blackout = false;
            }
            s.master.brightness = v;
        }
        ["bpm"] => s.bpm = want_num()?.round().clamp(20.0, 300.0),
        ["bpm", "tap"] => return Ok(Effect::Tap),
        ["bpm", "half"] => s.bpm = (s.bpm * 0.5).round().max(20.0),
        ["bpm", "double"] => s.bpm = (s.bpm * 2.0).round().min(300.0),
        ["record"] => return Ok(Effect::Record(want_bool()?)),
        ["page"] => s.page = (want_num()?.round() as usize).clamp(1, s.pages()) - 1,
        ["layer", n, field] => {
            let li = n.parse::<usize>().ok().and_then(|n| n.checked_sub(1)).filter(|i| *i < NUM_LAYERS).ok_or_else(bad)?;
            match *field {
                "select" => {
                    // like a click on the layer: select it and show its instance in the prepare
                    // detail; an empty layer empties the detail
                    s.selected_layer = li;
                    s.selected_instance = s.layers[li].instance;
                    if let Some(slot) = s.layers[li].instance.and_then(|id| s.slot_of(id)) {
                        s.page = slot / SLOTS_PER_PAGE;
                    }
                }
                "active" => s.layers[li].active = want_bool()?,
                "mix" => s.layers[li].mix.amount = want_num()?.clamp(0.0, 1.0),
                "mixtype" => s.layers[li].mix.mix_type = mixtype_value(value).ok_or_else(|| format!("{addr}: A, Lk, iLk or Ad"))?,
                "r" => s.layers[li].mix.rgb[0] = want_bool()?,
                "g" => s.layers[li].mix.rgb[1] = want_bool()?,
                "b" => s.layers[li].mix.rgb[2] = want_bool()?,
                "unload" => s.layers[li].instance = None,
                _ => return Err(bad()),
            }
        }
        ["slot", n, field] => {
            let si = n.parse::<usize>().ok().and_then(|n| n.checked_sub(1)).filter(|i| *i < s.slots.len()).ok_or_else(bad)?;
            let id = s.slots[si].as_ref().map(|i| i.id);
            match *field {
                "load" => {
                    if let Some(id) = id {
                        s.load_into_selected_layer(id);
                    }
                }
                "select" => s.selected_instance = id,
                _ => return Err(bad()),
            }
        }
        _ => {
            let (owner, rest) = resolve_owner(s, &p).ok_or_else(bad)?;
            match rest {
                ["fx", k, field] => {
                    let ki = num(Some(k)).and_then(|k| k.checked_sub(1)).ok_or_else(bad)?;
                    let fx = effects_of_mut(s, &owner).and_then(|v| v.get_mut(ki)).ok_or_else(bad)?;
                    match *field {
                        "active" => fx.active = want_bool()?,
                        "mix" => fx.mix.amount = want_num()?.clamp(0.0, 1.0),
                        "mixtype" => fx.mix.mix_type = mixtype_value(value).ok_or_else(|| format!("{addr}: A, Lk, iLk or Ad"))?,
                        _ => return Err(bad()),
                    }
                }
                ["fx", k, "param", n, sub @ ..] => {
                    let ki = num(Some(k)).and_then(|k| k.checked_sub(1)).ok_or_else(bad)?;
                    let fx = effects_of_mut(s, &owner).and_then(|v| v.get_mut(ki)).ok_or_else(bad)?;
                    let info = infos.get(&fx.path).ok_or_else(|| format!("{addr}: shader not loaded yet"))?;
                    let name = nth_input(info, n.parse().map_err(|_| bad())?).ok_or_else(bad)?.name.clone();
                    set_param(&mut fx.params, info, &name, sub.first().copied(), value).ok_or_else(bad)?;
                }
                ["param", n, sub @ ..] => {
                    let Owner::Inst(id) = owner else { return Err(bad()) };
                    let inst = s.find_mut(id).ok_or_else(bad)?;
                    let SourceKind::Generator { path, params } = &mut inst.source else { return Err(bad()) };
                    let info = infos.get(path).ok_or_else(|| format!("{addr}: shader not loaded yet"))?;
                    let name = nth_input(info, n.parse().map_err(|_| bad())?).ok_or_else(bad)?.name.clone();
                    set_param(params, info, &name, sub.first().copied(), value).ok_or_else(bad)?;
                }
                _ => return Err(bad()),
            }
        }
    }
    Ok(Effect::None)
}

fn mixtype_value(v: &Value) -> Option<MixType> {
    match v {
        Value::Text(t) => mixtype_from(t),
        Value::Num(n) => MixType::ALL.get(n.round().max(0.0) as usize).copied(),
        Value::Bool(_) => None,
    }
}

/// Clamped to the ISF range; long rounded. With an animation running, the animation moves it on.
fn set_param(params: &mut [Param], info: &IsfInfo, name: &str, sub: Option<&str>, value: &Value) -> Option<()> {
    let inp = info.inputs.iter().find(|i| i.name == name)?;
    let p = params.iter_mut().find(|p| p.name == name)?;
    // back to the isf default, as freshly loaded (value, range, animation off): like a right click
    if sub == Some("reset") {
        p.value = crate::isf::default_params(info).into_iter().find(|d| d.name == name)?.value;
        return Some(());
    }
    match (&inp.kind, &mut p.value, sub) {
        (InputKind::Float { min, max, .. }, ParamValue::Anim(a), None) => a.first_mut()?.value = value.as_num()?.clamp(*min, *max),
        (InputKind::Long { min, max, .. }, ParamValue::Anim(a), None) => a.first_mut()?.value = value.as_num()?.round().clamp(*min, *max),
        (InputKind::Bool { .. }, ParamValue::Bool(b), None) => *b = value.as_bool()?,
        (InputKind::Point2D { min, max, .. }, ParamValue::Anim(a), Some(c @ ("x" | "y"))) => {
            let i = if c == "x" { 0 } else { 1 };
            a.get_mut(i)?.value = value.as_num()?.clamp(min[i], max[i]);
        }
        _ => return None,
    }
    Some(())
}

/// min / max of a range address (None: not a range or does not exist)
pub fn range(s: &Session, infos: &Infos, addr: &str) -> Option<(f32, f32)> {
    let addr = resolve(s, addr);
    list(s, infos).into_iter().find(|d| d.addr == addr).and_then(|d| match d.kind {
        Kind::Range { min, max, .. } => Some((min, max)),
        _ => None,
    })
}

// ---------------------------------------------------------------------------------- feedback

/// The value of every readable address (for change notifications: LEDs, MCP).
pub fn snapshot(s: &Session, infos: &Infos, ext: Ext) -> BTreeMap<String, Value> {
    list(s, infos).into_iter().filter_map(|d| get(s, infos, ext, &d.addr).map(|v| (d.addr, v))).collect()
}

/// Addresses whose value changed (or appeared) between two snapshots.
pub fn changes(before: &BTreeMap<String, Value>, now: &BTreeMap<String, Value>) -> Vec<(String, Value)> {
    now.iter().filter(|(k, v)| before.get(*k) != Some(*v)).map(|(k, v)| (k.clone(), v.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{ClipTransport, Instance, Mix};
    use std::path::Path;

    fn setup() -> (Session, Infos, u64) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fx_path = root.join("isf-pack/effects/color-adjust.fs");
        let gen_path = root.join("isf-pack/generators/hex-pulse.fs");
        let fx_info = crate::isf::parse_info(&fx_path).unwrap().0;
        let gen_info = crate::isf::parse_info(&gen_path).unwrap().0;
        let mut s = Session::default();
        let clip = s.new_id();
        let uid = s.new_id();
        s.slots[0] = Some(Instance {
            id: clip,
            name: "clip".into(),
            source: SourceKind::Clip { path: "x.mp4".into(), transport: ClipTransport::default() },
            effects: vec![EffectInst {
                uid,
                path: fx_path.clone(),
                active: true,
                mix: Mix::default(),
                params: crate::isf::default_params(&fx_info),
                seed: 0.1,
                open: false,
            }],
            seed: 0.2,
        });
        let gen_id = s.new_id();
        s.slots[1] = Some(Instance {
            id: gen_id,
            name: "gen".into(),
            source: SourceKind::Generator { path: gen_path.clone(), params: crate::isf::default_params(&gen_info) },
            effects: vec![],
            seed: 0.3,
        });
        let mut infos = Infos::new();
        infos.insert(fx_path, fx_info);
        infos.insert(gen_path, gen_info);
        (s, infos, clip)
    }

    #[test]
    fn master_bpm_layers() {
        let (mut s, infos, _) = setup();
        set(&mut s, &infos, "/master/blackout", &Value::Bool(true)).unwrap();
        assert_eq!(get(&s, &infos, Ext::default(), "/master/brightness"), Some(Value::Num(0.0)));
        set(&mut s, &infos, "/master/brightness", &Value::Num(0.4)).unwrap();
        assert!(!s.master.blackout, "pulling brightness up ends the blackout");
        set(&mut s, &infos, "/bpm", &Value::Num(127.6)).unwrap();
        assert_eq!(s.bpm, 128.0);
        set(&mut s, &infos, "/bpm/half", &Value::Bool(true)).unwrap();
        assert_eq!(s.bpm, 64.0);
        assert_eq!(set(&mut s, &infos, "/bpm/tap", &Value::Bool(true)).unwrap(), Effect::Tap);
        assert_eq!(set(&mut s, &infos, "/record", &Value::Bool(true)).unwrap(), Effect::Record(true));
        set(&mut s, &infos, "/layer/3/mixtype", &Value::Text("ilk".into())).unwrap();
        assert_eq!(s.layers[2].mix.mix_type, MixType::InvLumaKey);
        set(&mut s, &infos, "/layer/2/mix", &Value::Num(7.0)).unwrap();
        assert_eq!(s.layers[1].mix.amount, 1.0, "clamped");
        set(&mut s, &infos, "/layer/2/g", &Value::Bool(false)).unwrap();
        assert_eq!(get(&s, &infos, Ext::default(), "/layer/2/g"), Some(Value::Bool(false)));
        assert!(set(&mut s, &infos, "/layer/4/mix", &Value::Num(0.5)).is_err());
        assert!(set(&mut s, &infos, "/slot/1/state", &Value::Text("L1".into())).is_err(), "read only");
    }

    #[test]
    fn slots_and_layer_aliases() {
        let (mut s, infos, clip) = setup();
        set(&mut s, &infos, "/layer/2/select", &Value::Bool(true)).unwrap();
        set(&mut s, &infos, "/slot/1/load", &Value::Bool(true)).unwrap();
        assert_eq!(s.layers[1].instance, Some(clip));
        assert_eq!(get(&s, &infos, Ext::default(), "/slot/1/state"), Some(Value::Text("L2".into())));
        assert_eq!(get(&s, &infos, Ext::default(), "/slot/2/state"), Some(Value::Text("loaded".into())));
        assert_eq!(get(&s, &infos, Ext::default(), "/slot/3/state"), Some(Value::Text("empty".into())));
        // the layer alias reaches the effect of whatever is loaded in layer 2
        set(&mut s, &infos, "/layer/2/fx/1/active", &Value::Bool(false)).unwrap();
        assert!(!s.find(clip).unwrap().effects[0].active);
        assert_eq!(get(&s, &infos, Ext::default(), "/slot/1/fx/1/active"), Some(Value::Bool(false)), "slot 1 holds the clip");
        let _ = clip;
        // page alias: slot 1 of page 1 is slot 1; on page 2 it is slot 33 (empty)
        assert_eq!(get(&s, &infos, Ext::default(), "/page/slot/1/state"), Some(Value::Text("L2".into())));
        s.slots.extend(std::iter::repeat_n(None, SLOTS_PER_PAGE));
        set(&mut s, &infos, "/page", &Value::Num(2.0)).unwrap();
        assert_eq!(get(&s, &infos, Ext::default(), "/page/slot/1/state"), Some(Value::Text("empty".into())));
        set(&mut s, &infos, "/page", &Value::Num(1.0)).unwrap();
        // selecting an empty layer empties the prepare detail (and /sel)
        s.selected_instance = Some(clip);
        set(&mut s, &infos, "/layer/3/select", &Value::Bool(true)).unwrap();
        assert_eq!((s.selected_layer, s.selected_instance), (2, None));
        assert_eq!(get(&s, &infos, Ext::default(), "/sel/param/1"), None);
        set(&mut s, &infos, "/layer/2/select", &Value::Bool(true)).unwrap();
        assert_eq!(s.selected_instance, Some(clip));
        set(&mut s, &infos, "/layer/2/unload", &Value::Bool(true)).unwrap();
        assert!(s.layers[1].instance.is_none());
        assert!(set(&mut s, &infos, "/layer/2/fx/1/active", &Value::Bool(true)).is_err(), "nothing loaded");
    }

    #[test]
    fn selected_focus() {
        let (mut s, infos, clip) = setup();
        // nothing selected: no /sel
        assert_eq!(get(&s, &infos, Ext::default(), "/sel/param/1"), None);
        // the clip (slot 1) selected, no focus: its first effect
        s.selected_instance = Some(clip);
        assert_eq!(sel_param_prefix(&s).as_deref(), Some("/slot/1/fx/1/param"));
        set(&mut s, &infos, "/sel/param/1", &Value::Num(0.5)).unwrap();
        assert_eq!(get(&s, &infos, Ext::default(), "/slot/1/fx/1/param/1"), Some(Value::Num(0.5)));
        assert_eq!(range(&s, &infos, "/sel/param/1"), Some((-1.0, 1.0)));
        // reset: back to the isf default (0)
        set(&mut s, &infos, "/sel/param/1/reset", &Value::Bool(true)).unwrap();
        assert_eq!(get(&s, &infos, Ext::default(), "/sel/param/1"), Some(Value::Num(0.0)));
        // the generator (slot 2) selected: its own parameters
        let gen_id = s.slots[1].as_ref().unwrap().id;
        s.selected_instance = Some(gen_id);
        assert_eq!(sel_param_prefix(&s).as_deref(), Some("/slot/2/param"));
        assert!(list(&s, &infos).iter().any(|d| d.addr == "/sel/param/1/reset"));
        // focus on an effect that is not in the selected instance: ignored
        s.focus_fx = Some(9999);
        assert_eq!(sel_param_prefix(&s).as_deref(), Some("/slot/2/param"));
    }

    #[test]
    fn params_clamped_and_listed() {
        let (mut s, infos, clip) = setup();
        let _ = clip;
        // parameter 1 of color-adjust is brightness (-1..1)
        let a = "/slot/1/fx/1/param/1".to_string();
        set(&mut s, &infos, &a, &Value::Num(9999.0)).unwrap();
        assert_eq!(get(&s, &infos, Ext::default(), &a), Some(Value::Num(1.0)), "clamped to the ISF MAX");
        let l = list(&s, &infos);
        let desc = l.iter().find(|d| d.addr == a).expect("listed");
        assert!(desc.label.contains("slot 1 (clip)") && desc.label.contains("color-adjust"), "{}", desc.label);
        // the page alias reaches the same parameter
        assert_eq!(get(&s, &infos, Ext::default(), "/page/slot/1/fx/1/param/1"), Some(Value::Num(1.0)));
        assert!(set(&mut s, &infos, "/slot/1/fx/1/param/99", &Value::Num(1.0)).is_err(), "no parameter 99");
        assert_eq!(desc.kind, Kind::Range { min: -1.0, max: 1.0, integer: false });
        // every listed, readable address round-trips through get
        let snap = snapshot(&s, &infos, Ext::default());
        assert!(snap.len() > 100);
        // a change shows up in the diff, nothing else
        let mut s2 = s.clone();
        set(&mut s2, &infos, "/layer/1/active", &Value::Bool(false)).unwrap();
        let ch = changes(&snap, &snapshot(&s2, &infos, Ext::default()));
        assert_eq!(ch, vec![("/layer/1/active".to_string(), Value::Bool(false))]);
    }
}
