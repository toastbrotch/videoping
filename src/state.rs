//! Session state: everything that gets saved (see features.md "save / load").

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const SLOTS_PER_PAGE: usize = 32; // 4 x 8 grid, mirrors bhoreal columns 2-5
pub const GRID_COLS: usize = 4;
pub const GRID_ROWS: usize = 8;
pub const NUM_LAYERS: usize = 3;
pub const MAX_EFFECTS: usize = 8;

/// Layer colors (ui live-marking and later bhoreal LEDs).
pub const LAYER_COLORS: [[u8; 3]; NUM_LAYERS] = [[230, 60, 60], [60, 200, 90], [70, 120, 255]];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MixType {
    #[default]
    Alpha,
    LumaKey,
    InvLumaKey,
    Add,
}

impl MixType {
    pub const ALL: [MixType; 4] = [MixType::Alpha, MixType::LumaKey, MixType::InvLumaKey, MixType::Add];
    pub fn label(self) -> &'static str {
        match self {
            MixType::Alpha => "alpha",
            MixType::LumaKey => "lumakey",
            MixType::InvLumaKey => "inv lumakey",
            MixType::Add => "add",
        }
    }
    pub fn shader_id(self) -> i32 {
        self as i32
    }
}

/// Mix-type, mix percentage and r/g/b channel mask: used by mixer layers and effects.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mix {
    pub mix_type: MixType,
    pub amount: f32,
    pub rgb: [bool; 3],
}

impl Default for Mix {
    fn default() -> Self {
        Mix { mix_type: MixType::Alpha, amount: 1.0, rgb: [true; 3] }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlayMode {
    #[default]
    Forward,
    Backward,
    Jumpcut,
    Pause,
}

impl PlayMode {
    pub const ALL: [PlayMode; 4] = [PlayMode::Forward, PlayMode::Backward, PlayMode::Jumpcut, PlayMode::Pause];
    pub fn icon(self) -> &'static str {
        match self {
            PlayMode::Forward => "▶",
            PlayMode::Backward => "◀",
            PlayMode::Jumpcut => "↯",
            PlayMode::Pause => "⏸",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            PlayMode::Forward => "forward",
            PlayMode::Backward => "backward",
            PlayMode::Jumpcut => "jumpcut",
            PlayMode::Pause => "pause",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LoopMode {
    #[default]
    Loop,
    Once,
    Bounce,
}

impl LoopMode {
    pub const ALL: [LoopMode; 3] = [LoopMode::Loop, LoopMode::Once, LoopMode::Bounce];
    pub fn label(self) -> &'static str {
        match self {
            LoopMode::Loop => "loop",
            LoopMode::Once => "once",
            LoopMode::Bounce => "bounce",
        }
    }
}

/// Speed: free (multiplier / cycles per second) or bpm (beats per limited part / cycle).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Speed {
    pub bpm_mode: bool,
    /// free mode: clip = playback rate (1.0 = normal), param = cycles per second
    pub free: f32,
    /// bpm mode: beats per limited part / per cycle; also the jumpcut interval N
    pub beats: f32,
}

impl Speed {
    pub fn clip_default() -> Self {
        Speed { bpm_mode: false, free: 1.0, beats: 4.0 }
    }
    pub fn param_default() -> Self {
        Speed { bpm_mode: false, free: 0.25, beats: 4.0 }
    }
    /// ½x button
    pub fn half(&mut self) {
        if self.bpm_mode {
            self.beats = (self.beats * 2.0).min(64.0);
        } else {
            self.free = (self.free * 0.5).max(0.01);
        }
    }
    /// 2x button
    pub fn double(&mut self) {
        if self.bpm_mode {
            self.beats = (self.beats * 0.5).max(0.25);
        } else {
            self.free = (self.free * 2.0).min(16.0);
        }
    }
    /// jumpcut interval in beats
    pub fn jump_beats(&self) -> f32 {
        self.beats.max(0.25)
    }
}

/// Animated range parameter (one float channel).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimParam {
    pub value: f32,
    pub lo: f32,
    pub hi: f32,
    pub mode: PlayMode,
    pub loop_mode: LoopMode,
    pub speed: Speed,
    /// runtime: phase 0..1 inside the range and direction (not essential to save, but harmless)
    #[serde(default)]
    pub phase: f32,
    #[serde(default = "one")]
    pub dir: f32,
}

fn one() -> f32 {
    1.0
}

impl AnimParam {
    pub fn new(value: f32, min: f32, max: f32) -> Self {
        AnimParam {
            value,
            lo: min,
            hi: max,
            mode: PlayMode::Pause,
            loop_mode: LoopMode::Loop,
            speed: Speed::param_default(),
            phase: 0.0,
            dir: 1.0,
        }
    }

    /// Advance the animation. `jump` is true on the frame a jumpcut should happen.
    pub fn tick(&mut self, dt: f32, bpm: f32, jump: bool, rng: &mut Rng) {
        let rate = if self.speed.bpm_mode {
            let secs = self.speed.beats.max(0.0625) * 60.0 / bpm.max(1.0);
            1.0 / secs
        } else {
            self.speed.free
        };
        match self.mode {
            PlayMode::Pause => return,
            PlayMode::Jumpcut => {
                if jump {
                    self.phase = rng.next_f32();
                } else {
                    return;
                }
            }
            PlayMode::Forward | PlayMode::Backward => {
                let base = if self.mode == PlayMode::Forward { 1.0 } else { -1.0 };
                let dir = if self.loop_mode == LoopMode::Bounce { self.dir } else { 1.0 };
                let mut p = self.phase + base * dir * rate * dt;
                match self.loop_mode {
                    LoopMode::Loop => p = p.rem_euclid(1.0),
                    LoopMode::Once => p = p.clamp(0.0, 1.0),
                    LoopMode::Bounce => {
                        if p > 1.0 {
                            p = 2.0 - p;
                            self.dir = -self.dir;
                        } else if p < 0.0 {
                            p = -p;
                            self.dir = -self.dir;
                        }
                        p = p.clamp(0.0, 1.0);
                    }
                }
                self.phase = p;
            }
        }
        self.value = self.lo + (self.hi - self.lo) * self.phase;
    }
}

/// Parameter value of an ISF input.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ParamValue {
    /// float, long (rounded), point2D (2 channels)
    Anim(Vec<AnimParam>),
    Bool(bool),
    Color([f32; 4]),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub value: ParamValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectInst {
    pub uid: u64,
    pub path: PathBuf,
    pub active: bool,
    pub mix: Mix,
    pub params: Vec<Param>,
    #[serde(default)]
    pub seed: f32,
    /// ui: parameters folded out
    #[serde(default)]
    pub open: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipTransport {
    pub mode: PlayMode,
    pub loop_mode: LoopMode,
    pub speed: Speed,
    /// limiters, normalized 0..1
    pub in_pt: f32,
    pub out_pt: f32,
}

impl Default for ClipTransport {
    fn default() -> Self {
        ClipTransport {
            mode: PlayMode::Forward,
            loop_mode: LoopMode::Loop,
            speed: Speed::clip_default(),
            in_pt: 0.0,
            out_pt: 1.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SourceKind {
    /// mp4 clip with timeline
    Clip { path: PathBuf, transport: ClipTransport },
    /// isf generator
    Generator { path: PathBuf, params: Vec<Param> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instance {
    pub id: u64,
    pub name: String,
    pub source: SourceKind,
    pub effects: Vec<EffectInst>,
    /// uInstanceSeed (random per instance, new on duplicate, saved)
    pub seed: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    pub instance: Option<u64>,
    pub active: bool,
    pub mix: Mix,
}

impl Default for Layer {
    fn default() -> Self {
        Layer { instance: None, active: true, mix: Mix::default() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Master {
    pub brightness: f32,
    /// not saved as "on": always start with blackout/freeze off
    #[serde(skip)]
    pub blackout: bool,
    #[serde(skip)]
    pub freeze: bool,
}

impl Default for Master {
    fn default() -> Self {
        Master { brightness: 1.0, blackout: false, freeze: false }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputOptions {
    pub width: u32,
    pub height: u32,
    pub window_open: bool,
    pub fullscreen: bool,
    pub ndi_enabled: bool,
    pub ndi_name: String,
    /// folder with libndi.so.6 (empty = search the default places)
    #[serde(default)]
    pub ndi_lib_dir: String,
}

impl Default for OutputOptions {
    fn default() -> Self {
        OutputOptions {
            width: 1920,
            height: 1080,
            window_open: false,
            fullscreen: false,
            ndi_enabled: false,
            ndi_name: "videoping".into(),
            ndi_lib_dir: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub slots: Vec<Option<Instance>>,
    pub page: usize,
    pub selected_instance: Option<u64>,
    pub layers: [Layer; NUM_LAYERS],
    pub selected_layer: usize,
    pub bpm: f32,
    pub output_effects: Vec<EffectInst>,
    pub master: Master,
    pub output: OutputOptions,
    pub finder_roots: Vec<PathBuf>,
    pub next_id: u64,
}

impl Default for Session {
    fn default() -> Self {
        let mut roots = vec![];
        if let Ok(cwd) = std::env::current_dir() {
            roots.push(cwd);
        }
        if let Some(v) = dirs::video_dir() {
            roots.push(v);
        }
        Session {
            slots: vec![None; SLOTS_PER_PAGE],
            page: 0,
            selected_instance: None,
            layers: Default::default(),
            selected_layer: 0,
            bpm: 120.0,
            output_effects: vec![],
            master: Master::default(),
            output: OutputOptions::default(),
            finder_roots: roots,
            next_id: 1,
        }
    }
}

impl Session {
    pub fn new_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn pages(&self) -> usize {
        self.slots.len().div_ceil(SLOTS_PER_PAGE).max(1)
    }

    pub fn find(&self, id: u64) -> Option<&Instance> {
        self.slots.iter().flatten().find(|i| i.id == id)
    }

    pub fn find_mut(&mut self, id: u64) -> Option<&mut Instance> {
        self.slots.iter_mut().flatten().find(|i| i.id == id)
    }

    pub fn slot_of(&self, id: u64) -> Option<usize> {
        self.slots.iter().position(|s| s.as_ref().is_some_and(|i| i.id == id))
    }

    /// mixer-layer the instance is live in
    pub fn live_layer(&self, id: u64) -> Option<usize> {
        self.layers.iter().position(|l| l.instance == Some(id))
    }

    /// first free slot (adds a page if all are full)
    pub fn first_free_slot(&mut self) -> usize {
        if let Some(i) = self.slots.iter().position(|s| s.is_none()) {
            return i;
        }
        let i = self.slots.len();
        self.slots.extend(std::iter::repeat_n(None, SLOTS_PER_PAGE));
        i
    }

    /// load instance into the selected mixer-layer. Already live in another layer -> nothing happens.
    pub fn load_into_selected_layer(&mut self, id: u64) {
        if let Some(l) = self.live_layer(id) {
            if l != self.selected_layer {
                return;
            }
        }
        self.layers[self.selected_layer].instance = Some(id);
        self.selected_instance = Some(id);
    }

    /// x on a slot: removes the instance, also from the mixer
    pub fn delete_slot(&mut self, slot: usize) {
        if let Some(inst) = self.slots[slot].take() {
            for l in &mut self.layers {
                if l.instance == Some(inst.id) {
                    l.instance = None;
                }
            }
            if self.selected_instance == Some(inst.id) {
                self.selected_instance = None;
            }
        }
    }

    pub fn duplicate_slot(&mut self, slot: usize, rng: &mut Rng) -> Option<usize> {
        let mut copy = self.slots.get(slot)?.clone()?;
        copy.id = self.new_id();
        copy.seed = rng.next_f32();
        for fx in &mut copy.effects {
            fx.uid = self.new_id();
        }
        let free = self.first_free_slot();
        self.slots[free] = Some(copy);
        Some(free)
    }

    pub fn swap_slots(&mut self, a: usize, b: usize) {
        if a < self.slots.len() && b < self.slots.len() {
            self.slots.swap(a, b);
        }
    }
}

/// Small xorshift rng (no external dependency, deterministic enough for vj randomness).
pub struct Rng(u64);

impl Rng {
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x1234_5678)
            | 1;
        Rng(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inst(s: &mut Session, slot: usize) -> u64 {
        let id = s.new_id();
        s.slots[slot] = Some(Instance {
            id,
            name: format!("i{id}"),
            source: SourceKind::Clip { path: "x.mp4".into(), transport: ClipTransport::default() },
            effects: vec![],
            seed: 0.5,
        });
        id
    }

    #[test]
    fn live_instance_not_loaded_into_second_layer() {
        let mut s = Session::default();
        let a = inst(&mut s, 0);
        s.selected_layer = 0;
        s.load_into_selected_layer(a);
        s.selected_layer = 1;
        s.load_into_selected_layer(a);
        assert_eq!(s.layers[0].instance, Some(a));
        assert_eq!(s.layers[1].instance, None);
    }

    #[test]
    fn delete_live_instance_empties_layer() {
        let mut s = Session::default();
        let a = inst(&mut s, 3);
        s.load_into_selected_layer(a);
        s.delete_slot(3);
        assert!(s.slots[3].is_none());
        assert_eq!(s.layers[0].instance, None);
    }

    #[test]
    fn duplicate_gets_new_id_and_seed() {
        let mut s = Session::default();
        let a = inst(&mut s, 0);
        let mut rng = Rng::new();
        let ns = s.duplicate_slot(0, &mut rng).unwrap();
        let b = s.slots[ns].as_ref().unwrap();
        assert_ne!(b.id, a);
        assert_ne!(b.seed, 0.5);
    }

    #[test]
    fn full_page_adds_page() {
        let mut s = Session::default();
        for i in 0..SLOTS_PER_PAGE {
            inst(&mut s, i);
        }
        let free = s.first_free_slot();
        assert_eq!(free, SLOTS_PER_PAGE);
        assert_eq!(s.pages(), 2);
    }

    #[test]
    fn speed_half_double() {
        let mut sp = Speed::clip_default();
        sp.double();
        assert_eq!(sp.free, 2.0);
        sp.bpm_mode = true;
        sp.double(); // 2x in bpm mode = half the beats
        assert_eq!(sp.beats, 2.0);
        sp.half();
        assert_eq!(sp.beats, 4.0);
    }

    #[test]
    fn anim_bpm_cycle_and_bounce() {
        let mut rng = Rng::new();
        let mut a = AnimParam::new(0.0, 0.0, 1.0);
        a.mode = PlayMode::Forward;
        a.speed.bpm_mode = true;
        a.speed.beats = 4.0; // 120 bpm -> one cycle = 2 s
        a.tick(1.0, 120.0, false, &mut rng);
        assert!((a.value - 0.5).abs() < 1e-4);
        a.loop_mode = LoopMode::Bounce;
        a.tick(1.5, 120.0, false, &mut rng); // 0.5 + 0.75 = 1.25 -> bounces to 0.75
        assert!((a.value - 0.75).abs() < 1e-4);
        assert_eq!(a.dir, -1.0);
    }

    #[test]
    fn anim_jumpcut_only_on_jump() {
        let mut rng = Rng::new();
        let mut a = AnimParam::new(0.3, 0.0, 1.0);
        a.mode = PlayMode::Jumpcut;
        a.tick(0.5, 120.0, false, &mut rng);
        assert_eq!(a.value, 0.3);
        a.tick(0.5, 120.0, true, &mut rng);
        assert!(a.value >= 0.0 && a.value <= 1.0);
    }

    #[test]
    fn blackout_freeze_not_saved_on() {
        let mut s = Session::default();
        s.master.blackout = true;
        s.master.freeze = true;
        let back: Session = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert!(!back.master.blackout && !back.master.freeze);
    }
}
