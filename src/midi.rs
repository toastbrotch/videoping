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
    /// on connect and on quit, besides every bound control, notes 0..n on channel 1 are switched
    /// off; None = all 128 (every controller is cleared completely). The Bhoreal: 64, its firmware
    /// does not check higher note numbers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clear_notes: Option<u8>,
    /// the same for control changes 0..n on channel 1; None = all 128 (the Twister: 64 = the rings
    /// of its 4 banks)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clear_ccs: Option<u8>,
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

/// The Bhoreal Slim (8x8) as in features.md: column 0..7 from the left, row 0..7 from the top
/// (cable top right, see bhoreal_note).
/// Colors: layer colors for live / on / selected, plum (the accent) = "there, but not on",
/// red = blackout / recording, yellow = output effect on, off = nothing.
/// Note of a Bhoreal pad, column / row counted from the top left as you play it, with the usb
/// cable at the top right (checked on the hardware 2026-11-21, pads and leds alike: top left =
/// note 63, the pad right of it = 62, bottom right = note 0; the device counts row by row from
/// the corner opposite the cable).
pub fn bhoreal_note(col: u8, row: u8) -> u8 {
    63 - (row * 8 + col)
}

pub fn bhoreal_profile() -> Profile {
    let layer = [bhoreal_hue([0x1B, 0x68, 0x98]), bhoreal_hue([0x38, 0x8E, 0x3C]), bhoreal_hue([0xC8, 0x64, 0x0A])];
    let plum = bhoreal_hue([0x8B, 0x4A, 0x87]);
    let red = 1;
    let yellow = bhoreal_hue([0xFF, 0xE0, 0x00]);
    let pad = |col: u8, row: u8| Msg::Note(bhoreal_note(col, row));
    let b = |msg, mode, addr: String, feedback| Binding { channel: 0, msg, mode, addr, feedback, feedback_addr: None };
    let mut v = vec![];
    // column 1: select layer 1-3, layer 1-3 on/off, blackout, freeze
    for l in 0..3u8 {
        v.push(b(pad(0, l), Mode::Trigger, format!("/layer/{}/select", l + 1), Feedback::OnOff { off: 0, on: layer[l as usize] }));
        v.push(b(pad(0, l + 3), Mode::Toggle, format!("/layer/{}/active", l + 1), Feedback::OnOff { off: 0, on: layer[l as usize] }));
    }
    v.push(b(pad(0, 6), Mode::Toggle, "/master/blackout".into(), Feedback::OnOff { off: 0, on: red }));
    // bottom left: record (red while recording, like "● REC" in the ui)
    v.push(b(pad(0, 7), Mode::Toggle, "/record".into(), Feedback::OnOff { off: 0, on: red }));
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
    // columns 6-8, top 4 rows: effects 1-4 of what plays in layer 1 / 2 / 3
    for l in 0..3u8 {
        for k in 0..4u8 {
            v.push(b(pad(5 + l, k), Mode::Toggle, format!("/layer/{}/fx/{}/active", l + 1, k + 1), Feedback::OnOff { off: plum, on: layer[l as usize] }));
        }
    }
    // column 6, bottom 4 rows: output effects 1-4 (yellow: no layer color, the output is everyone's)
    for k in 0..4u8 {
        v.push(b(pad(5, 4 + k), Mode::Toggle, format!("/out/fx/{}/active", k + 1), Feedback::OnOff { off: plum, on: yellow }));
    }
    // columns 7-8, bottom 4 rows: free
    Profile { name: "Bhoreal".into(), ports: vec!["bhoreal".into(), "leonardo".into()], bindings: v, clear_notes: Some(64), clear_ccs: Some(0) }
}

/// Encoder number of the Midi Fighter Twister: row by row from the top left (checked on the
/// hardware 2026-11-21: top left = cc 0, top right = cc 3, bottom right = cc 15). Turning sends
/// that cc on channel 1 (absolute 0..127), pushing it on channel 2 (127 / 0); a cc sent to it on
/// channel 1 sets the led ring.
pub fn twister_cc(col: u8, row: u8) -> u8 {
    row * 4 + col
}

/// The Midi Fighter Twister default layout (features.md): column 4 = the mix of layer 1 / 2 / 3
/// and the master brightness (top-down); the led rings show the value. The other 12 are free.
pub fn twister_profile() -> Profile {
    let enc = |row: u8, addr: &str| Binding {
        channel: 0,
        msg: Msg::Cc(twister_cc(3, row)),
        mode: Mode::Absolute,
        addr: addr.into(),
        feedback: Feedback::Range,
        feedback_addr: None,
    };
    let bindings = vec![enc(0, "/layer/1/mix"), enc(1, "/layer/2/mix"), enc(2, "/layer/3/mix"), enc(3, "/master/brightness")];
    // all 4 banks (16 encoders each, cc 0..63): also rings of encoders without a binding go to 0
    Profile { name: "Midi Fighter Twister".into(), ports: vec!["twister".into()], bindings, clear_notes: Some(0), clear_ccs: Some(64) }
}

pub fn builtin_profiles() -> Vec<Profile> {
    vec![bhoreal_profile(), twister_profile()]
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
    /// profiles changed and not written yet (by name); written ~0.5 s after the last change
    dirty: std::collections::HashSet<String>,
    dirty_since: Option<Instant>,
    /// where profiles are kept (None: nowhere, e.g. the self-test)
    dir: Option<std::path::PathBuf>,
}

impl Midi {
    /// Profiles from ~/.config/videoping/midi (the self-test keeps them in memory only).
    pub fn new() -> Midi {
        let dir = if std::env::var_os("VIDEOPING_SELFTEST").is_some() { None } else { Some(profiles_dir()) };
        Midi::with_dir(dir)
    }

    pub fn with_dir(dir: Option<std::path::PathBuf>) -> Midi {
        Midi {
            profiles: load_profiles(dir.as_deref()),
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
            dirty_since: None,
            dir,
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

    /// The folder the profiles are saved in (for the hint in the tab).
    pub fn dir(&self) -> Option<&std::path::Path> {
        self.dir.as_deref()
    }

    fn changed(&mut self, pi: usize) {
        self.dirty.insert(self.profiles[pi].name.clone());
        self.dirty_since = Some(Instant::now());
        for d in &mut self.devices {
            if d.profile == Some(pi) {
                d.sent.clear();
            }
        }
    }

    /// Write changed profiles (called from poll, 0.5 s after the last change, and on quit).
    pub fn flush(&mut self) {
        let Some(dir) = self.dir.clone() else {
            self.dirty.clear();
            return;
        };
        for name in std::mem::take(&mut self.dirty) {
            let Some(p) = self.profiles.iter().find(|p| p.name == name) else { continue };
            let r = std::fs::create_dir_all(&dir)
                .map_err(|e| e.to_string())
                .and_then(|_| serde_json::to_string_pretty(p).map_err(|e| e.to_string()))
                .and_then(|txt| std::fs::write(dir.join(format!("{}.json", file_name(&p.name))), txt).map_err(|e| e.to_string()));
            if let Err(e) = r {
                self.error = Some(format!("midi profile {name}: {e}"));
            }
        }
        self.dirty_since = None;
    }

    /// Use profile `pi` (None: none) for this port. The device name moves to that profile, so it
    /// is picked automatically next time (one profile per device name).
    pub fn set_device_profile(&mut self, port: &str, pi: Option<usize>) {
        let base = port_base(port);
        let low = port.to_lowercase();
        for i in 0..self.profiles.len() {
            let before = self.profiles[i].ports.len();
            if Some(i) != pi {
                // everything in other profiles that would match this device goes
                self.profiles[i].ports.retain(|m| !low.contains(&m.to_lowercase()));
            } else if !self.profiles[i].matches(port) {
                self.profiles[i].ports.push(base.clone());
            }
            if self.profiles[i].ports.len() != before {
                self.changed(i);
            }
        }
        if let Some(d) = self.devices.iter_mut().find(|d| d.port == port) {
            // the old profile's leds off, the new one's state follows with the next feedback
            clear_device(d, d.profile.map(|old| &self.profiles[old]));
            d.profile = pi;
            d.sent.clear();
        }
    }

    /// Quit: every device's leds off.
    pub fn shutdown(&mut self) {
        for d in &mut self.devices {
            let p = d.profile.map(|pi| &self.profiles[pi]);
            clear_device(d, p);
        }
    }

    /// Edit a profile (bindings, ports); saved automatically, the leds are refreshed.
    pub fn edit_profile(&mut self, pi: usize, f: impl FnOnce(&mut Profile)) {
        let before = self.profiles[pi].clone();
        f(&mut self.profiles[pi]);
        if self.profiles[pi] != before {
            self.changed(pi);
        }
    }

    fn unique_name(&self, base: &str) -> String {
        let base = if base.trim().is_empty() { "profile" } else { base.trim() };
        if !self.profiles.iter().any(|p| p.name == base) {
            return base.to_string();
        }
        (2..).map(|n| format!("{base} {n}")).find(|n| !self.profiles.iter().any(|p| &p.name == n)).unwrap()
    }

    /// A new empty profile for this port (named after the device), used for it from now on.
    pub fn new_profile(&mut self, port: &str) -> usize {
        let name = self.unique_name(&port_base(port));
        self.profiles.push(Profile { name, ports: vec![], bindings: vec![], clear_notes: None, clear_ccs: None });
        let pi = self.profiles.len() - 1;
        self.changed(pi);
        self.set_device_profile(port, Some(pi));
        pi
    }

    /// A copy ("Bhoreal 2"), not matched to any device yet (choose it in the device dropdown).
    pub fn duplicate_profile(&mut self, pi: usize) -> usize {
        let mut p = self.profiles[pi].clone();
        p.name = self.unique_name(&p.name);
        p.ports.clear();
        self.profiles.push(p);
        let ni = self.profiles.len() - 1;
        self.changed(ni);
        ni
    }

    /// Rename (not a built-in: duplicate it instead). The old file is removed.
    pub fn rename_profile(&mut self, pi: usize, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("a profile needs a name".into());
        }
        if self.is_builtin(pi) {
            return Err("a built-in profile keeps its name: duplicate it to make your own".into());
        }
        if self.profiles.iter().enumerate().any(|(i, p)| i != pi && p.name == name) {
            return Err(format!("there is already a profile \"{name}\""));
        }
        let old = std::mem::replace(&mut self.profiles[pi].name, name.to_string());
        if let Some(dir) = &self.dir {
            let _ = std::fs::remove_file(dir.join(format!("{}.json", file_name(&old))));
        }
        self.dirty.remove(&old);
        self.changed(pi);
        Ok(())
    }

    /// Delete (not a built-in); devices that used it have no profile then.
    pub fn delete_profile(&mut self, pi: usize) -> Result<(), String> {
        if self.is_builtin(pi) {
            return Err("a built-in profile can not be deleted (\"built-in\" restores it)".into());
        }
        let p = self.profiles.remove(pi);
        if let Some(dir) = &self.dir {
            let _ = std::fs::remove_file(dir.join(format!("{}.json", file_name(&p.name))));
        }
        self.dirty.remove(&p.name);
        for d in &mut self.devices {
            d.profile = match d.profile {
                Some(i) if i == pi => None,
                Some(i) if i > pi => Some(i - 1),
                x => x,
            };
            d.sent.clear();
        }
        Ok(())
    }

    pub fn is_builtin(&self, pi: usize) -> bool {
        builtin_profiles().iter().any(|b| b.name == self.profiles[pi].name)
    }

    /// Back to the built-in version (removes the saved user file).
    pub fn revert_profile(&mut self, pi: usize) {
        let name = self.profiles[pi].name.clone();
        if let Some(b) = builtin_profiles().into_iter().find(|b| b.name == name) {
            if let Some(dir) = &self.dir {
                let _ = std::fs::remove_file(dir.join(format!("{}.json", file_name(&name))));
            }
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
            if let Some(mut dev) = self.connect(&name, pi) {
                match pi {
                    Some(pi) => eprintln!("midi: {name} -> profile {}", self.profiles[pi].name),
                    None => eprintln!("midi: {name} (no profile)"),
                }
                // all leds off first (every controller, also without profile); the real state
                // follows with the next feedback
                clear_device(&mut dev, pi.map(|pi| &self.profiles[pi]));
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
        // the output of the same device for the leds: the same name (without the alsa numbers),
        // else the same device name, else one the profile matches
        let base = port_base(port_name).to_lowercase();
        let key = port_key(port_name);
        let output = MidiOutput::new("videoping").ok().and_then(|out| {
            let ports = out.ports();
            let p = ports
                .iter()
                .find(|p| out.port_name(p).is_ok_and(|n| port_key(&n) == key))
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
        if self.dirty_since.is_some_and(|d| d.elapsed().as_secs_f32() > 0.5) {
            self.flush();
        }
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

/// Switch a controller's leds off: every bound control (note: velocity 0, cc: value 0) and notes
/// / ccs 0..n on channel 1 (all 128 each unless the profile limits it; no profile: all).
fn clear_device(d: &mut Device, p: Option<&Profile>) {
    let Some(out) = d.output.as_mut() else { return };
    let notes = p.and_then(|p| p.clear_notes).unwrap_or(128).min(128);
    let ccs = p.and_then(|p| p.clear_ccs).unwrap_or(128).min(128);
    let mut msgs: Vec<[u8; 3]> = (0..notes).map(|n| [0x90, n, 0]).collect();
    msgs.extend((0..ccs).map(|c| [0xB0, c, 0]));
    for b in p.map(|p| p.bindings.as_slice()).unwrap_or_default() {
        let m = match b.msg {
            Msg::Note(n) => [0x90 | b.channel, n, 0],
            Msg::Cc(c) => [0xB0 | b.channel, c, 0],
        };
        if !msgs.contains(&m) {
            msgs.push(m);
        }
    }
    for m in msgs {
        let _ = out.send(&m);
    }
    d.sent.clear();
}

/// The port name without the alsa client:port numbers at the end ("... 20:0").
fn port_key(port: &str) -> String {
    let t = port.trim();
    match t.rsplit_once(' ') {
        Some((head, last)) if last.split(':').all(|x| !x.is_empty() && x.chars().all(|c| c.is_ascii_digit())) => head.to_string(),
        _ => t.to_string(),
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

/// Built-in profiles, replaced by user profiles of the same name from `dir` (sorted by name).
fn load_profiles(dir: Option<&std::path::Path>) -> Vec<Profile> {
    let mut v = builtin_profiles();
    let Some(dir) = dir else { return v };
    let mut files: Vec<_> = std::fs::read_dir(dir).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "json")).collect();
    files.sort();
    for f in files {
        let Ok(txt) = std::fs::read_to_string(&f) else { continue };
        match serde_json::from_str::<Profile>(&txt) {
            Ok(p) => match v.iter().position(|b| b.name == p.name) {
                Some(i) => v[i] = p,
                None => v.push(p),
            },
            Err(err) => eprintln!("midi: {}: {err}", f.display()),
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
        assert_eq!(p.bindings.len(), 8 + 32 + 12 + 4);
        assert_eq!(p.clear_notes, Some(64), "all pads off on connect / quit, not beyond 63");
        let find = |n: u8| p.bindings.iter().find(|b| b.msg == Msg::Note(n)).unwrap().addr.clone();
        let at = |col, row| find(bhoreal_note(col, row));
        // as pressed on the hardware with the cable top right
        assert_eq!(bhoreal_note(0, 0), 63, "top left");
        assert_eq!(bhoreal_note(1, 0), 62, "right of it");
        assert_eq!(bhoreal_note(7, 7), 0, "bottom right");
        // column 1 top-down: select 1-3, on/off 1-3, blackout, record
        assert_eq!(at(0, 0), "/layer/1/select");
        assert_eq!(at(0, 5), "/layer/3/active");
        assert_eq!(at(0, 6), "/master/blackout");
        assert_eq!(at(0, 7), "/record");
        // columns 2-5: slots top-down per column, like the grid
        assert_eq!(at(1, 0), "/page/slot/1/load");
        assert_eq!(at(1, 1), "/page/slot/2/load");
        assert_eq!(at(2, 0), "/page/slot/9/load");
        assert_eq!(at(4, 7), "/page/slot/32/load");
        // columns 6-8, top 4 rows: effects 1-4 of layer 1-3
        assert_eq!(at(5, 0), "/layer/1/fx/1/active");
        assert_eq!(at(7, 3), "/layer/3/fx/4/active");
        // column 6, bottom 4 rows: output effects 1-4
        assert_eq!(at(5, 4), "/out/fx/1/active");
        assert_eq!(at(5, 7), "/out/fx/4/active");
        // every pad at most once; columns 7-8, bottom 4 rows are free
        let mut notes: Vec<u8> = p.bindings.iter().map(|b| match b.msg { Msg::Note(n) => n, Msg::Cc(c) => c }).collect();
        notes.sort();
        notes.dedup();
        assert_eq!(notes.len(), 56);
        for col in 6..8 {
            for row in 4..8 {
                assert!(!notes.contains(&bhoreal_note(col, row)), "free pad {col},{row}");
            }
        }
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
    fn twister_layout() {
        let p = twister_profile();
        let find = |cc: u8| p.bindings.iter().find(|b| b.msg == Msg::Cc(cc)).map(|b| (b.addr.clone(), b.mode, b.channel));
        assert_eq!(twister_cc(0, 0), 0, "top left");
        assert_eq!(twister_cc(3, 0), 3, "top right (its push came as cc 3 on channel 2)");
        assert_eq!(twister_cc(3, 3), 15, "bottom right");
        assert_eq!(find(3), Some(("/layer/1/mix".into(), Mode::Absolute, 0)));
        assert_eq!(find(7).unwrap().0, "/layer/2/mix");
        assert_eq!(find(11).unwrap().0, "/layer/3/mix");
        assert_eq!(find(15).unwrap().0, "/master/brightness");
        assert_eq!(p.clear_ccs, Some(64), "all rings of all 4 banks off on connect / quit");
        assert!(p.matches("Midi Fighter Twister:Midi Fighter Twister MIDI 1 16:0"));
        assert!(!bhoreal_profile().matches("Midi Fighter Twister:Midi Fighter Twister MIDI 1 16:0"));
    }

    #[test]
    fn port_names() {
        assert_eq!(port_key("Arduino Leonardo:Arduino Leonardo MIDI 1 20:0"), "Arduino Leonardo:Arduino Leonardo MIDI 1");
        assert_eq!(port_key("bhoreal-selftest:Bhoreal selftest 129:0"), port_key("bhoreal-selftest:Bhoreal selftest 128:0"));
        assert_eq!(port_base("Midi Fighter Twister:Midi Fighter Twister MIDI 1 16:0"), "Midi Fighter Twister");
    }

    #[test]
    fn profiles_saved_renamed_deleted() {
        let dir = std::env::temp_dir().join(format!("videoping-midi-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut m = Midi::with_dir(Some(dir.clone()));
        let b = m.profiles().iter().position(|p| p.name == "Bhoreal").unwrap();
        // a copy, renamed, edited: written automatically
        let c = m.duplicate_profile(b);
        assert_eq!(m.profiles()[c].name, "Bhoreal 2");
        assert!(m.profiles()[c].ports.is_empty(), "a copy matches no device until chosen");
        m.rename_profile(c, "club").unwrap();
        m.edit_profile(c, |p| p.bindings.truncate(3));
        assert!(m.rename_profile(b, "x").is_err(), "built-ins keep their name");
        assert!(m.rename_profile(c, "Bhoreal").is_err(), "no duplicate names");
        m.flush();
        assert!(dir.join("club.json").exists());
        assert!(!dir.join("Bhoreal_2.json").exists(), "the old name's file is gone");
        // loaded again from the folder
        let m2 = Midi::with_dir(Some(dir.clone()));
        let club = m2.profiles().iter().find(|p| p.name == "club").unwrap();
        assert_eq!(club.bindings.len(), 3);
        // choosing a profile for a device moves the device name to it
        let mut m3 = Midi::with_dir(Some(dir.clone()));
        let ci = m3.profiles().iter().position(|p| p.name == "club").unwrap();
        m3.set_device_profile("Arduino Leonardo:Arduino Leonardo MIDI 1 20:0", Some(ci));
        assert!(m3.profiles()[ci].matches("Arduino Leonardo:x"));
        let bi = m3.profiles().iter().position(|p| p.name == "Bhoreal").unwrap();
        assert!(!m3.profiles()[bi].matches("Arduino Leonardo:x"), "only one profile per device name");
        // delete
        assert!(m3.delete_profile(bi).is_err(), "built-ins can not be deleted");
        let ci = m3.profiles().iter().position(|p| p.name == "club").unwrap();
        m3.delete_profile(ci).unwrap();
        assert!(!dir.join("club.json").exists());
        let _ = std::fs::remove_dir_all(&dir);
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
