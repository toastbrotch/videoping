//! MIDI engine: generic for any controller. It knows no devices, only profiles of bindings
//! between MIDI messages and control registry addresses (control.rs). The Bhoreal (and later the
//! Midi Fighter Twister) are built-in profiles of the same engine.
//!
//! - ports are scanned every 2 s (hot plug): a port whose name matches a profile is connected
//!   (input, and the output of the same device for the LED feedback)
//! - incoming messages are queued by the MIDI thread and applied at the start of the next video
//!   frame (`poll`), not through the ui
//! - feedback: for every binding with feedback the current value is read from the registry and
//!   sent when it changed (so the LEDs follow mouse changes too)
//! - profiles are setup, not set content: user profiles live in ~/.config/videoping/midi/*.json
//!   and replace a built-in profile of the same name

use crate::control::Value;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Msg {
    Note(u8),
    Cc(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    /// press = fire once
    Trigger,
    /// press = on / off
    Toggle,
    /// on while held
    Momentary,
    /// cc 0..127 -> the address range
    Absolute,
    /// endless encoder: value 64 = no move, 65 = +1, 63 = -1 (Twister "ENC 3FH/41H")
    Relative,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Feedback {
    None,
    /// bool state -> velocity (note) or value (cc); no value (e.g. no effect there) -> 0
    OnOff { off: u8, on: u8 },
    /// choice value -> velocity, anything else -> `other`
    Values { map: Vec<(String, u8)>, other: u8 },
    /// range value -> 0..127
    Range,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    /// 0..15 (MIDI channel 1..16)
    pub channel: u8,
    pub msg: Msg,
    pub mode: Mode,
    pub addr: String,
    pub feedback: Feedback,
    /// where the feedback is read from, when it is not `addr` (e.g. a slot pad triggers
    /// /page/slot/N/load, its color comes from /page/slot/N/state)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feedback_addr: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    /// a port whose name contains one of these (any case) uses this profile
    pub ports: Vec<String>,
    pub bindings: Vec<Binding>,
}

impl Profile {
    fn matches(&self, port: &str) -> bool {
        let p = port.to_lowercase();
        self.ports.iter().any(|m| p.contains(&m.to_lowercase()))
    }
}

/// What the engine controls (the app: control registry).
pub trait Target {
    fn get(&self, addr: &str) -> Option<Value>;
    fn set(&mut self, addr: &str, value: &Value) -> Result<(), String>;
    /// min / max of a range address
    fn range(&self, addr: &str) -> Option<(f32, f32)>;
}

// ------------------------------------------------------------------------------ bhoreal

/// Bhoreal LED velocity for a color: the firmware maps velocity * 8 onto a hue wheel
/// (0..340 red -> green, 341..681 green -> blue, 682..1023 blue -> red), velocity 0 = off.
/// The LEDs can only show full brightness, no dimming.
pub fn bhoreal_hue(rgb: [u8; 3]) -> u8 {
    let [r, g, b] = rgb.map(|c| c as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d <= 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / d).rem_euclid(6.0))
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    ((h / 360.0 * 1023.0 / 8.0).round() as i32).clamp(1, 127) as u8
}

/// The Bhoreal Slim (8x8) as in features.md. Pad note = column * 8 + row (column 0..7 from the
/// left, row 0..7 from the top; to be checked on the hardware, the firmware does not say).
/// Colors: layer colors for live / on / selected, plum (the accent) = "there, but not on",
/// red = blackout, cyan = freeze, off = nothing.
pub fn bhoreal_profile() -> Profile {
    let layer = [bhoreal_hue([0x1B, 0x68, 0x98]), bhoreal_hue([0x38, 0x8E, 0x3C]), bhoreal_hue([0xC8, 0x64, 0x0A])];
    let plum = bhoreal_hue([0x8B, 0x4A, 0x87]);
    let red = 1;
    let cyan = bhoreal_hue([0x00, 0xFF, 0xFF]);
    let pad = |col: u8, row: u8| Msg::Note(col * 8 + row);
    let b = |msg, mode, addr: String, feedback| Binding { channel: 0, msg, mode, addr, feedback, feedback_addr: None };
    let mut v = vec![];
    // column 1: select layer 1-3, layer 1-3 on/off, blackout, freeze
    for l in 0..3u8 {
        v.push(b(pad(0, l), Mode::Trigger, format!("/layer/{}/select", l + 1), Feedback::OnOff { off: 0, on: layer[l as usize] }));
        v.push(b(pad(0, l + 3), Mode::Toggle, format!("/layer/{}/active", l + 1), Feedback::OnOff { off: 0, on: layer[l as usize] }));
    }
    v.push(b(pad(0, 6), Mode::Toggle, "/master/blackout".into(), Feedback::OnOff { off: 0, on: red }));
    v.push(b(pad(0, 7), Mode::Toggle, "/master/freeze".into(), Feedback::OnOff { off: 0, on: cyan }));
    // columns 2-5: the 32 slots of the current page (top-down per column, like the grid)
    for col in 1..5u8 {
        for row in 0..8u8 {
            let n = (col - 1) * 8 + row + 1;
            let map = vec![("loaded".into(), plum), ("L1".into(), layer[0]), ("L2".into(), layer[1]), ("L3".into(), layer[2])];
            let mut slot = b(pad(col, row), Mode::Trigger, format!("/page/slot/{n}/load"), Feedback::Values { map, other: 0 });
            slot.feedback_addr = Some(format!("/page/slot/{n}/state"));
            v.push(slot);
        }
    }
    // columns 6-8: effects 1-8 of what is loaded in layer 1 / 2 / 3
    for l in 0..3u8 {
        for k in 0..8u8 {
            v.push(b(pad(5 + l, k), Mode::Toggle, format!("/layer/{}/fx/{}/active", l + 1, k + 1), Feedback::OnOff { off: plum, on: layer[l as usize] }));
        }
    }
    Profile { name: "Bhoreal".into(), ports: vec!["bhoreal".into(), "leonardo".into()], bindings: v }
}

pub fn builtin_profiles() -> Vec<Profile> {
    vec![bhoreal_profile()]
}

// ------------------------------------------------------------------------------ engine

struct Device {
    id: u64,
    port: String,
    /// None: connected, but no profile (yet): its messages show in the monitor and can be learned
    profile: Option<usize>,
    _input: MidiInputConnection<()>,
    output: Option<MidiOutputConnection>,
    /// last value sent per binding (feedback only when it changes)
    sent: HashMap<usize, u8>,
    last_msg: Option<Instant>,
}

/// "learn": the next message on `port` becomes a binding to `addr`.
#[derive(Clone, Debug, PartialEq)]
pub struct Learn {
    pub port: String,
    pub addr: String,
    pub kind: crate::control::Kind,
}

/// The last message that hit a binding (to highlight its row in the tab).
#[derive(Clone, Debug)]
pub struct Hit {
    pub port: String,
    pub channel: u8,
    pub msg: Msg,
    pub at: Instant,
}

pub struct Midi {
    profiles: Vec<Profile>,
    devices: Vec<Device>,
    queue: Arc<Mutex<Vec<(u64, Vec<u8>)>>>,
    next_id: u64,
    last_scan: Option<Instant>,
    last_feedback: Option<Instant>,
    /// last incoming messages, newest first: (port, text)
    pub monitor: VecDeque<(String, String)>,
    pub error: Option<String>,
    pub learn: Option<Learn>,
    /// what the last learn did (shown in the tab)
    pub learned: Option<String>,
    pub last_hit: Option<Hit>,
    /// profiles changed since they were loaded / saved (by name)
    pub dirty: std::collections::HashSet<String>,
}

impl Midi {
    pub fn new() -> Midi {
        Midi {
            profiles: load_profiles(),
            devices: vec![],
            queue: Default::default(),
            next_id: 1,
            last_scan: None,
            last_feedback: None,
            monitor: VecDeque::new(),
            error: None,
            learn: None,
            learned: None,
            last_hit: None,
            dirty: Default::default(),
        }
    }

    /// Connected devices: (port name, profile name or None).
    pub fn devices(&self) -> Vec<(String, Option<String>)> {
        self.devices.iter().map(|d| (d.port.clone(), d.profile.map(|p| self.profiles[p].name.clone()))).collect()
    }

    /// A message arrived from this port within the last 0.3 s (activity light).
    pub fn active(&self, port: &str) -> bool {
        self.devices.iter().any(|d| d.port == port && d.last_msg.is_some_and(|t| t.elapsed().as_secs_f32() < 0.3))
    }

    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    pub fn device_profile(&self, port: &str) -> Option<usize> {
        self.devices.iter().find(|d| d.port == port).and_then(|d| d.profile)
    }

    /// Use profile `pi` (None: none) for this port; the profile then also matches the port's
    /// device name next time.
    pub fn set_device_profile(&mut self, port: &str, pi: Option<usize>) {
        if let Some(i) = pi {
            let base = port_base(port);
            let p = &mut self.profiles[i];
            if !p.matches(port) {
                p.ports.push(base);
                self.dirty.insert(p.name.clone());
            }
        }
        if let Some(d) = self.devices.iter_mut().find(|d| d.port == port) {
            d.profile = pi;
            d.sent.clear();
        }
    }

    /// Edit a profile (bindings, ports); the leds are refreshed.
    pub fn edit_profile(&mut self, pi: usize, f: impl FnOnce(&mut Profile)) {
        let before = self.profiles[pi].clone();
        f(&mut self.profiles[pi]);
        if self.profiles[pi] != before {
            self.dirty.insert(self.profiles[pi].name.clone());
            for d in &mut self.devices {
                if d.profile == Some(pi) {
                    d.sent.clear();
                }
            }
        }
    }

    /// A new empty profile for this port (named after the device).
    pub fn new_profile(&mut self, port: &str) -> usize {
        let mut name = port_base(port);
        while self.profiles.iter().any(|p| p.name == name) {
            name.push('+');
        }
        self.profiles.push(Profile { name: name.clone(), ports: vec![port_base(port)], bindings: vec![] });
        self.dirty.insert(name);
        let pi = self.profiles.len() - 1;
        self.set_device_profile(port, Some(pi));
        pi
    }

    /// Save to ~/.config/videoping/midi/<name>.json
    pub fn save_profile(&mut self, pi: usize) -> Result<std::path::PathBuf, String> {
        let p = &self.profiles[pi];
        let dir = profiles_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("{}.json", file_name(&p.name)));
        let txt = serde_json::to_string_pretty(p).map_err(|e| e.to_string())?;
        std::fs::write(&path, txt).map_err(|e| format!("{}: {e}", path.display()))?;
        self.dirty.remove(&p.name);
        Ok(path)
    }

    pub fn is_builtin(&self, pi: usize) -> bool {
        builtin_profiles().iter().any(|b| b.name == self.profiles[pi].name)
    }

    /// Back to the built-in version (removes the saved user file).
    pub fn revert_profile(&mut self, pi: usize) {
        let name = self.profiles[pi].name.clone();
        if let Some(b) = builtin_profiles().into_iter().find(|b| b.name == name) {
            let _ = std::fs::remove_file(profiles_dir().join(format!("{}.json", file_name(&name))));
            self.profiles[pi] = b;
            self.dirty.remove(&name);
            for d in &mut self.devices {
                if d.profile == Some(pi) {
                    d.sent.clear();
                }
            }
        }
    }

    /// Every 2 s: connect new ports, drop vanished ones.
    fn scan(&mut self) {
        if self.last_scan.is_some_and(|t| t.elapsed().as_secs_f32() < 2.0) {
            return;
        }
        self.last_scan = Some(Instant::now());
        let Ok(probe) = MidiInput::new("videoping scan") else {
            self.error = Some("MIDI (ALSA) not available".into());
            return;
        };
        let names: Vec<String> = probe.ports().iter().filter_map(|p| probe.port_name(p).ok()).collect();
        self.devices.retain(|d| names.contains(&d.port));
        for name in names {
            let low = name.to_lowercase();
            // our own ports and the system loopback are no controllers
            if low.starts_with("videoping") || low.starts_with("midi through") || self.devices.iter().any(|d| d.port == name) {
                continue;
            }
            let pi = self.profiles.iter().position(|p| p.matches(&name));
            if let Some(dev) = self.connect(&name, pi) {
                match pi {
                    Some(pi) => eprintln!("midi: {name} -> profile {}", self.profiles[pi].name),
                    None => eprintln!("midi: {name} (no profile)"),
                }
                self.devices.push(dev);
            }
        }
    }

    fn connect(&mut self, port_name: &str, profile: Option<usize>) -> Option<Device> {
        let mut input = MidiInput::new("videoping").ok()?;
        input.ignore(Ignore::All);
        let port = input.ports().into_iter().find(|p| input.port_name(p).ok().as_deref() == Some(port_name))?;
        let id = self.next_id;
        self.next_id += 1;
        let q = self.queue.clone();
        let conn = input
            .connect(
                &port,
                "videoping-in",
                move |_, bytes, _| {
                    if let Ok(mut q) = q.lock() {
                        q.push((id, bytes.to_vec()));
                    }
                },
                (),
            )
            .map_err(|e| self.error = Some(format!("midi in {port_name}: {e}")))
            .ok()?;
        // the output of the same device for the leds: the same name, or the same device name
        let base = port_base(port_name).to_lowercase();
        let output = MidiOutput::new("videoping").ok().and_then(|out| {
            let ports = out.ports();
            let p = ports
                .iter()
                .find(|p| out.port_name(p).ok().as_deref() == Some(port_name))
                .or_else(|| {
                    ports.iter().find(|p| {
                        out.port_name(p).is_ok_and(|n| {
                            let n = n.to_lowercase();
                            !n.starts_with("videoping") && (n.contains(&base) || profile.is_some_and(|pi| self.profiles[pi].matches(&n)))
                        })
                    })
                })?
                .clone();
            out.connect(&p, "videoping-out").ok()
        });
        Some(Device { id, port: port_name.to_string(), profile, _input: conn, output, sent: HashMap::new(), last_msg: None })
    }

    /// Once per video frame: scan ports, apply queued messages, send changed feedback.
    pub fn poll(&mut self, t: &mut impl Target) {
        self.scan();
        let msgs: Vec<(u64, Vec<u8>)> = self.queue.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();
        for (id, bytes) in msgs {
            self.handle(id, &bytes, t);
        }
        // feedback at most ~30 times per second
        if self.last_feedback.is_none_or(|f| f.elapsed().as_millis() >= 33) {
            self.last_feedback = Some(Instant::now());
            self.feedback(t);
        }
    }

    fn handle(&mut self, id: u64, bytes: &[u8], t: &mut impl Target) {
        let Some((ch, msg, val, release)) = parse(bytes) else { return };
        let Some(di) = self.devices.iter().position(|d| d.id == id) else { return };
        self.devices[di].last_msg = Some(Instant::now());
        let port = self.devices[di].port.clone();
        self.monitor.push_front((port_base(&port), format!("ch {} {} {}", ch + 1, msg_text(msg), val)));
        self.monitor.truncate(32);
        // learn: the first press / move on the learning device becomes a binding
        if let Some(l) = self.learn.clone() {
            if l.port == port && !(matches!(msg, Msg::Note(_)) && release) {
                match learned_binding(&l, ch, msg, val) {
                    Ok(b) => {
                        let pi = match self.devices[di].profile {
                            Some(pi) => pi,
                            None => self.new_profile(&port),
                        };
                        self.edit_profile(pi, |p| {
                            // one control, one target: replace an older binding of the same control
                            p.bindings.retain(|x| !(x.channel == b.channel && x.msg == b.msg));
                            p.bindings.push(b.clone());
                        });
                        self.learned = Some(format!("{} -> {} ({:?})", msg_text(msg), b.addr, b.mode));
                    }
                    Err(e) => self.learned = Some(e),
                }
                self.learn = None;
                return;
            }
        }
        let Some(pi) = self.devices[di].profile else { return };
        let bindings: Vec<Binding> = self.profiles[pi].bindings.iter().filter(|b| b.channel == ch && b.msg == msg).cloned().collect();
        if !bindings.is_empty() {
            self.last_hit = Some(Hit { port, channel: ch, msg, at: Instant::now() });
        }
        for b in &bindings {
            // targets that do not exist right now (e.g. no effect 5 in that layer) are normal
            let _ = apply(b, val, release, t);
        }
    }

    fn feedback(&mut self, t: &impl Target) {
        for dev in &mut self.devices {
            let (Some(out), Some(pi)) = (dev.output.as_mut(), dev.profile) else { continue };
            for (i, b) in self.profiles[pi].bindings.iter().enumerate() {
                let fa = b.feedback_addr.as_deref().unwrap_or(&b.addr);
                let v = match &b.feedback {
                    Feedback::None => continue,
                    Feedback::OnOff { off, on } => match t.get(fa) {
                        Some(Value::Bool(true)) => *on,
                        Some(_) => *off,
                        None => 0,
                    },
                    Feedback::Values { map, other } => match t.get(fa) {
                        Some(Value::Text(s)) => map.iter().find(|m| m.0 == s).map(|m| m.1).unwrap_or(*other),
                        _ => *other,
                    },
                    Feedback::Range => match (t.get(fa), t.range(fa)) {
                        (Some(Value::Num(x)), Some((lo, hi))) if hi > lo => (((x - lo) / (hi - lo)) * 127.0).round().clamp(0.0, 127.0) as u8,
                        _ => 0,
                    },
                };
                if dev.sent.get(&i) == Some(&v) {
                    continue;
                }
                let bytes = match b.msg {
                    Msg::Note(n) => [0x90 | b.channel, n, v],
                    Msg::Cc(c) => [0xB0 | b.channel, c, v],
                };
                if out.send(&bytes).is_ok() {
                    dev.sent.insert(i, v);
                }
            }
        }
    }
}

/// "nanoKONTROL2:nanoKONTROL2 _ CTRL 20:0" -> "nanoKONTROL2" (the device name before the port)
pub fn port_base(port: &str) -> String {
    port.split(':').next().unwrap_or(port).trim().to_string()
}

pub fn msg_text(m: Msg) -> String {
    match m {
        Msg::Note(n) => format!("note {n}"),
        Msg::Cc(c) => format!("cc {c}"),
    }
}

fn file_name(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
}

/// The binding "learn" makes, from the kind of the address and what was moved.
fn learned_binding(l: &Learn, channel: u8, msg: Msg, val: u8) -> Result<Binding, String> {
    use crate::control::Kind;
    let (mode, feedback) = match (&l.kind, msg) {
        (Kind::Trigger, _) => (Mode::Trigger, Feedback::OnOff { off: 0, on: 127 }),
        (Kind::Toggle, _) => (Mode::Toggle, Feedback::OnOff { off: 0, on: 127 }),
        // encoders send values around 64 (63 / 65) or 1 / 127; faders and knobs anything
        (Kind::Range { .. }, Msg::Cc(_)) if matches!(val, 1 | 63 | 65 | 127) => (Mode::Relative, Feedback::Range),
        (Kind::Range { .. }, Msg::Cc(_)) => (Mode::Absolute, Feedback::Range),
        (Kind::Range { .. } | Kind::Choice(_), Msg::Note(_)) => return Err(format!("{}: needs a knob or fader, not a button", l.addr)),
        (Kind::Choice(_), Msg::Cc(_)) => return Err(format!("{}: choices can not be mapped yet", l.addr)),
    };
    Ok(Binding { channel, msg, mode, addr: l.addr.clone(), feedback, feedback_addr: None })
}

/// (channel 0..15, message, velocity / value, is a release)
fn parse(bytes: &[u8]) -> Option<(u8, Msg, u8, bool)> {
    let [status, d1, d2] = match bytes {
        [s, a, b, ..] => [*s, *a, *b],
        _ => return None,
    };
    let ch = status & 0x0F;
    match status & 0xF0 {
        0x90 if d2 > 0 => Some((ch, Msg::Note(d1), d2, false)),
        0x90 | 0x80 => Some((ch, Msg::Note(d1), 0, true)),
        // cc buttons send 127 / 0: value 0 = release for trigger / toggle / momentary
        0xB0 => Some((ch, Msg::Cc(d1), d2, d2 == 0)),
        _ => None,
    }
}

/// One incoming message on one binding.
fn apply(b: &Binding, val: u8, release: bool, t: &mut impl Target) -> Result<(), String> {
    match (b.mode, release) {
        (Mode::Trigger, false) => t.set(&b.addr, &Value::Bool(true)),
        (Mode::Toggle, false) => {
            let on = matches!(t.get(&b.addr), Some(Value::Bool(true)));
            t.set(&b.addr, &Value::Bool(!on))
        }
        (Mode::Momentary, rel) => t.set(&b.addr, &Value::Bool(!rel)),
        (Mode::Absolute, _) => match t.range(&b.addr) {
            Some((lo, hi)) => t.set(&b.addr, &Value::Num(lo + val as f32 / 127.0 * (hi - lo))),
            None => Err(format!("{}: not a range", b.addr)),
        },
        // one encoder step = 1/200 of the range
        (Mode::Relative, _) => match (t.range(&b.addr), t.get(&b.addr)) {
            (Some((lo, hi)), Some(Value::Num(cur))) => t.set(&b.addr, &Value::Num((cur + (val as f32 - 64.0) * (hi - lo) / 200.0).clamp(lo, hi))),
            _ => Err(format!("{}: not a range", b.addr)),
        },
        _ => Ok(()),
    }
}

fn profiles_dir() -> std::path::PathBuf {
    dirs::config_dir().unwrap_or_else(|| ".".into()).join("videoping").join("midi")
}

/// Built-in profiles, replaced by user profiles of the same name from ~/.config/videoping/midi.
fn load_profiles() -> Vec<Profile> {
    let mut v = builtin_profiles();
    if std::env::var_os("VIDEOPING_SELFTEST").is_some() {
        return v; // the self-test uses the built-in profiles only
    }
    if let Ok(rd) = std::fs::read_dir(profiles_dir()) {
        for e in rd.flatten() {
            let Ok(txt) = std::fs::read_to_string(e.path()) else { continue };
            match serde_json::from_str::<Profile>(&txt) {
                Ok(p) => match v.iter().position(|b| b.name == p.name) {
                    Some(i) => v[i] = p,
                    None => v.push(p),
                },
                Err(err) => eprintln!("midi: {}: {err}", e.path().display()),
            }
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bhoreal_colors() {
        // layer colors land in the right thirds of the firmware hue wheel
        let blue = bhoreal_hue([0x1B, 0x68, 0x98]);
        let green = bhoreal_hue([0x38, 0x8E, 0x3C]);
        let orange = bhoreal_hue([0xC8, 0x64, 0x0A]);
        assert!((5..=15).contains(&orange), "orange {orange}");
        assert!((40..=48).contains(&green), "green {green}");
        assert!((68..=76).contains(&blue), "blue {blue}");
        assert_eq!(bhoreal_hue([255, 0, 0]), 1, "red, never 0 (= off)");
    }

    #[test]
    fn bhoreal_layout() {
        let p = bhoreal_profile();
        assert_eq!(p.bindings.len(), 8 + 32 + 24);
        let find = |n: u8| p.bindings.iter().find(|b| b.msg == Msg::Note(n)).unwrap().addr.clone();
        assert_eq!(find(0), "/layer/1/select");
        assert_eq!(find(5), "/layer/3/active");
        assert_eq!(find(6), "/master/blackout");
        assert_eq!(find(8), "/page/slot/1/load");
        assert_eq!(find(39), "/page/slot/32/load");
        assert_eq!(find(40), "/layer/1/fx/1/active");
        assert_eq!(find(63), "/layer/3/fx/8/active");
        // every pad exactly once
        let mut notes: Vec<u8> = p.bindings.iter().map(|b| match b.msg { Msg::Note(n) => n, Msg::Cc(c) => c }).collect();
        notes.sort();
        assert_eq!(notes, (0..64).collect::<Vec<_>>());
    }

    /// A fake target for the binding modes.
    struct Fake(HashMap<String, Value>);
    impl Target for Fake {
        fn get(&self, a: &str) -> Option<Value> {
            self.0.get(a).cloned()
        }
        fn set(&mut self, a: &str, v: &Value) -> Result<(), String> {
            self.0.insert(a.into(), v.clone());
            Ok(())
        }
        fn range(&self, _: &str) -> Option<(f32, f32)> {
            Some((0.0, 2.0))
        }
    }

    #[test]
    fn modes() {
        let bind = |msg, mode, addr: &str| Binding { channel: 0, msg, mode, addr: addr.into(), feedback: Feedback::None, feedback_addr: None };
        let bindings = [
            bind(Msg::Note(1), Mode::Toggle, "/t"),
            bind(Msg::Note(2), Mode::Momentary, "/m"),
            bind(Msg::Cc(3), Mode::Absolute, "/a"),
            bind(Msg::Cc(4), Mode::Relative, "/r"),
        ];
        let mut f = Fake(HashMap::new());
        f.0.insert("/r".into(), Value::Num(1.0));
        let send = |f: &mut Fake, bytes: [u8; 3]| {
            let (ch, msg, val, release) = parse(&bytes).unwrap();
            for b in bindings.iter().filter(|b| b.channel == ch && b.msg == msg) {
                apply(b, val, release, f).unwrap();
            }
        };
        send(&mut f, [0x90, 1, 64]);
        send(&mut f, [0x90, 1, 0]); // release (note on, velocity 0, like the bhoreal)
        assert_eq!(f.get("/t"), Some(Value::Bool(true)), "toggle on press, release ignored");
        send(&mut f, [0x90, 1, 64]);
        assert_eq!(f.get("/t"), Some(Value::Bool(false)));
        send(&mut f, [0x90, 2, 64]);
        assert_eq!(f.get("/m"), Some(Value::Bool(true)));
        send(&mut f, [0x80, 2, 0]);
        assert_eq!(f.get("/m"), Some(Value::Bool(false)), "momentary off on release");
        send(&mut f, [0xB0, 3, 127]);
        assert_eq!(f.get("/a"), Some(Value::Num(2.0)));
        send(&mut f, [0xB0, 4, 65]);
        assert_eq!(f.get("/r"), Some(Value::Num(1.01)));
        send(&mut f, [0xB1, 4, 65]); // other channel: no binding
        assert_eq!(f.get("/r"), Some(Value::Num(1.01)));
        assert!(parse(&[0xF8]).is_none(), "clock / short messages ignored");
    }

    #[test]
    fn learn_guesses_the_mode() {
        use crate::control::Kind;
        let l = |kind: Kind| Learn { port: "x".into(), addr: "/a".into(), kind };
        let range = Kind::Range { min: 0.0, max: 1.0, integer: false };
        assert_eq!(learned_binding(&l(Kind::Toggle), 0, Msg::Note(5), 64).unwrap().mode, Mode::Toggle);
        assert_eq!(learned_binding(&l(Kind::Trigger), 2, Msg::Cc(7), 127).unwrap().channel, 2);
        assert_eq!(learned_binding(&l(range.clone()), 0, Msg::Cc(7), 40).unwrap().mode, Mode::Absolute, "fader");
        assert_eq!(learned_binding(&l(range.clone()), 0, Msg::Cc(7), 65).unwrap().mode, Mode::Relative, "encoder step");
        assert!(learned_binding(&l(range), 0, Msg::Note(5), 64).is_err(), "a button can not drive a range");
        // a cc button: 127 = press, 0 = release
        assert_eq!(parse(&[0xB0, 9, 127]).unwrap().3, false);
        assert_eq!(parse(&[0xB0, 9, 0]).unwrap().3, true);
    }
}
