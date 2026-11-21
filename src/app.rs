//! The "live" tab: 5 columns (finder | prepare grid + master + bpm + record + settings | prepare detail | mixer + output) + top bar.

use crate::isf::{self, FileKind, InputKind, IsfInfo};
use crate::meter::{self, Key};
use crate::render::{FrameCtx, Renderer, PREVIEW_ID};
use crate::state::*;
use crate::widgets::{self, layer_color, paint_tex, ACCENT};
use egui::{vec2, Color32, Rect, Sense, Ui};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Drag & drop payloads.
#[derive(Clone, Debug)]
enum Drag {
    File(PathBuf, FileKind),
    Slot(usize),
    Effect(Owner, usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Owner {
    Instance(u64),
    Output,
}

struct FinderEntry {
    path: PathBuf,
    name: String,
    kind: FileKind,
}

pub struct App {
    selftest: Option<SelfTest>,
    s: Session,
    r: Option<Renderer>,
    rng: Rng,
    last: Instant,
    beat: f64,
    taps: Vec<Instant>,
    finder: Vec<FinderEntry>,
    /// finder folders as last scanned, with the modification time of every scanned directory:
    /// checked every 2 s, a change (new / deleted / renamed file, other folders) rescans
    finder_watch: (Vec<PathBuf>, Vec<(PathBuf, Option<std::time::SystemTime>)>),
    finder_checked: Instant,
    finder_search: String,
    finder_sel: Option<(PathBuf, FileKind)>,
    finder_new_root: String,
    preview: Option<Instance>,
    preview_key: Option<(PathBuf, Option<u64>)>,
    tex_ids: HashMap<glow::Texture, egui::TextureId>,
    infos: HashMap<PathBuf, IsfInfo>,
    autosave_path: PathBuf,
    /// general config (not part of the session), saved when it changes
    cfg: Config,
    tab: Tab,
    /// midi mapping tab: selected device (port), address, search
    midi_sel_port: Option<String>,
    midi_sel_addr: Option<String>,
    midi_search: String,
    /// profile name being edited (profile index, text) and a delete waiting for "really?"
    midi_rename: Option<(usize, String)>,
    midi_confirm_delete: Option<usize>,
    /// MIDI engine (generic, profiles; None while it is being polled)
    midi: Option<crate::midi::Midi>,
    /// control registry: values at the last control_changes() call (change notifications)
    control_snap: std::collections::BTreeMap<String, crate::control::Value>,
    /// last render time and beat per instance (see inst_clock)
    inst_clock: HashMap<u64, (Instant, f64)>,
    /// prepare work runs every n-th video frame (2 = half rate, 4, 8; 0 = paused)
    prepare_div: u32,
    prepare_div_since: Instant,
    /// GPU ms of the main mix per frame (live instances, mixer, output effects, master, REC)
    main_ms: f32,
    /// renders of the selected non-live instance (self-test statistics)
    prepare_renders: u64,
    /// REC clicked: start (true) / stop (false), done in render() where the GL context is
    rec_request: Option<bool>,
    /// size of the current recording file (read once per second)
    rec_size: (Option<Instant>, u64),
    last_saved: String,
    last_autosave: Instant,
    last_gc: Instant,
    file_path: String,
    status: String,
    status_time: Instant,
    output_aspect: f32,
    /// available widths of column 3 / column 4 (last frame): both previews get the same size
    preview_avail: [f32; 2],
    /// "new" was clicked: confirmation dialog is open
    confirm_new: bool,
    /// next due time of a video frame (output frame rate)
    next_video: Option<Instant>,
    /// rendered video frames (self-test statistics)
    video_frames: u64,
    /// times of the video frames of the last second (fps counter in the top bar)
    frame_times: std::collections::VecDeque<Instant>,
    /// video frames missed since start (rendered too late by a whole interval or more)
    frames_dropped: u64,
    /// last time a frame was missed (the counter stays marked for a moment)
    last_drop: Option<Instant>,
    /// process CPU / RAM (resource meters)
    proc_stats: crate::meter::ProcStats,
    /// OpenGL renderer string of the gpu in use
    gpu_name: String,
    /// connected screens (drm connector names), refreshed every few seconds
    screens: Vec<String>,
    screens_read: Option<Instant>,
    /// (fullscreen, screen) last applied to the output window
    applied_output: Option<(bool, usize)>,
    /// NDI source discovery (started once libndi is loaded)
    ndi_finder: Option<crate::ndi::NdiFinder>,
    ndi_finder_try: Option<Instant>,
}

fn config_path() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("videoping").join("config.json")
}

fn load_config() -> Config {
    std::fs::read_to_string(config_path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn save_config(c: &Config) -> Result<(), String> {
    let p = config_path();
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let txt = serde_json::to_string_pretty(c).map_err(|e| e.to_string())?;
    std::fs::write(&p, txt).map_err(|e| format!("{}: {e}", p.display()))
}

fn autosave_path() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("videoping").join("autosave.json")
}

fn load_session(path: &Path) -> Result<Session, String> {
    let txt = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut s: Session = serde_json::from_str(&txt).map_err(|e| format!("{}: {e}", path.display()))?;
    s.bpm = s.bpm.round().clamp(20.0, 300.0); // bpm is an integer
    // shaders may have changed on disk: match params to the current inputs
    let fix = |path: &Path, params: &mut Vec<Param>| {
        if let Ok((info, _)) = isf::parse_info(path) {
            isf::reconcile_params(&info, params);
        }
    };
    for inst in s.slots.iter_mut().flatten() {
        if let SourceKind::Generator { path, params } = &mut inst.source {
            fix(path, params);
        }
        for fx in &mut inst.effects {
            fix(&fx.path, &mut fx.params);
        }
    }
    for fx in &mut s.output_effects {
        fix(&fx.path, &mut fx.params);
    }
    if s.slots.len() < SLOTS_PER_PAGE {
        s.slots.resize(SLOTS_PER_PAGE, None);
    }
    Ok(s)
}

fn save_session(s: &Session, path: &Path) -> Result<String, String> {
    let txt = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &txt).map_err(|e| format!("{}: {e}", path.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(txt)
}

impl crate::midi::Target for App {
    fn get(&self, addr: &str) -> Option<crate::control::Value> {
        self.control_get(addr)
    }
    fn set(&mut self, addr: &str, value: &crate::control::Value) -> Result<(), String> {
        self.control_set(addr, value)
    }
    fn range(&self, addr: &str) -> Option<(f32, f32)> {
        self.control_range(addr)
    }
}

/// Time step and beat of an instance since its own last render (instances in prepare are rendered
/// less often: their animations still run in real time). First render: the frame's dt.
fn inst_clock(clocks: &mut HashMap<u64, (Instant, f64)>, id: u64, now: Instant, dt: f32, beat_prev: f64, beat_now: f64) -> (f32, f64) {
    let r = match clocks.get(&id) {
        // longer gaps (instance was idle) are not caught up: at most half a second
        Some((t, b)) if now.duration_since(*t).as_secs_f32() < 1.0 => (now.duration_since(*t).as_secs_f32().min(0.5), *b),
        _ => (dt, beat_prev),
    };
    clocks.insert(id, (now, beat_now));
    r
}

fn mtime(p: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

/// Files of the finder folders, plus every scanned directory with its modification time.
fn scan_with_dirs(roots: &[PathBuf]) -> (Vec<FinderEntry>, Vec<(PathBuf, Option<std::time::SystemTime>)>) {
    fn walk(dir: &Path, depth: usize, out: &mut Vec<FinderEntry>, dirs: &mut Vec<(PathBuf, Option<std::time::SystemTime>)>) {
        if depth > 5 {
            return;
        }
        dirs.push((dir.to_path_buf(), mtime(dir)));
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            if p.is_dir() {
                walk(&p, depth + 1, out, dirs);
            } else if let Some(kind) = isf::classify(&p) {
                out.push(FinderEntry { name: isf::display_name(&p), path: p, kind });
            }
        }
    }
    let mut out = vec![];
    let mut dirs = vec![];
    for r in roots {
        walk(r, 0, &mut out, &mut dirs);
    }
    (out, dirs)
}


impl App {
    pub fn new(cc: &eframe::CreationContext) -> App {
        style(&cc.egui_ctx);
        let ap = autosave_path();
        let selftest = std::env::var("VIDEOPING_SELFTEST").ok().map(|dir| SelfTest {
            dir: PathBuf::from(dir),
            start: Instant::now(),
            shot: false,
            dumped: false,
            timeline: None,
            drag_step: 0,
            in_before: None,
            l2_mix: None,
            closing: false,
            empty_slot: None,
            clear_btn: None,
            brightness_slider: None,
            fx_handle: None,
            out_fx: None,
            at2s: None,
            ui_frames: 0,
            rec_step: 0,
            rec_file: None,
            rec_stopped_at: None,
            control_done: false,
            midi_sim: None,
            prep0: 0,
        });
        let (s, status) = if let Some(t) = &selftest {
            (selftest_session(t), "selftest".to_string())
        } else {
            match load_session(&ap) {
                Ok(s) => (s, format!("restored {}", ap.display())),
                Err(_) => (Session::default(), "new session".to_string()),
            }
        };
        let (finder, finder_dirs) = scan_with_dirs(&s.finder_roots);
        let finder_roots = s.finder_roots.clone();
        let default_file = dirs::home_dir().unwrap_or_default().join("videoping-session.json");
        App {
            selftest,
            s,
            r: None,
            rng: Rng::new(),
            last: Instant::now(),
            beat: 0.0,
            taps: vec![],
            finder_watch: (finder_roots, finder_dirs),
            finder_checked: Instant::now(),
            finder,
            finder_search: String::new(),
            finder_sel: None,
            finder_new_root: String::new(),
            preview: None,
            preview_key: None,
            tex_ids: HashMap::new(),
            infos: HashMap::new(),
            autosave_path: ap,
            // self-test: default config, never saved (VIDEOPING_SELFTEST_NOMETERS: meters off for
            // the overhead comparison)
            cfg: if std::env::var_os("VIDEOPING_SELFTEST").is_some() {
                Config {
                    meters: std::env::var_os("VIDEOPING_SELFTEST_NOMETERS").is_none(),
                    // the self-test folder only when the self-test records; otherwise the default
                    // (shown in the record box, nothing is written there)
                    record_dir: if std::env::var_os("VIDEOPING_SELFTEST_REC").is_some() {
                        std::env::var("VIDEOPING_SELFTEST").unwrap_or_default()
                    } else {
                        String::new()
                    },
                }
            } else {
                load_config()
            },
            last_saved: String::new(),
            last_autosave: Instant::now(),
            last_gc: Instant::now(),
            file_path: default_file.display().to_string(),
            status,
            status_time: Instant::now(),
            output_aspect: 16.0 / 9.0,
            preview_avail: [640.0; 2],
            ndi_finder: None,
            ndi_finder_try: None,
            confirm_new: false,
            next_video: None,
            video_frames: 0,
            frame_times: Default::default(),
            frames_dropped: 0,
            last_drop: None,
            tab: if std::env::var("VIDEOPING_SELFTEST_TAB").as_deref() == Ok("midi") { Tab::Midi } else { Tab::Live },
            midi_sel_port: None,
            midi_sel_addr: None,
            midi_search: String::new(),
            midi_rename: None,
            midi_confirm_delete: None,
            midi: Some(crate::midi::Midi::new()),
            control_snap: Default::default(),
            inst_clock: HashMap::new(),
            prepare_div: 2,
            prepare_div_since: Instant::now(),
            main_ms: 0.0,
            prepare_renders: 0,
            rec_request: None,
            rec_size: (None, 0),
            proc_stats: crate::meter::ProcStats::new(),
            gpu_name: String::new(),
            screens: vec![],
            screens_read: None,
            applied_output: None,
        }
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status = msg.into();
        self.status_time = Instant::now();
    }

    fn tex_id(&mut self, frame: &mut eframe::Frame, tex: Option<glow::Texture>) -> Option<egui::TextureId> {
        let tex = tex?;
        Some(*self.tex_ids.entry(tex).or_insert_with(|| frame.register_native_glow_texture(tex)))
    }

    fn new_instance_from_file(&mut self, path: &Path, kind: &FileKind) -> Option<Instance> {
        let source = match kind {
            FileKind::Clip => SourceKind::Clip { path: path.to_path_buf(), transport: ClipTransport::default() },
            FileKind::Generator => {
                let (info, _) = isf::parse_info(path).ok()?;
                SourceKind::Generator { path: path.to_path_buf(), params: isf::default_params(&info) }
            }
            FileKind::Ndi => SourceKind::Ndi { source: path.to_string_lossy().to_string() },
            FileKind::Effect => return None,
        };
        let name = match kind {
            FileKind::Ndi => path.to_string_lossy().to_string(),
            _ => isf::display_name(path),
        };
        Some(Instance {
            id: self.s.new_id(),
            name,
            source,
            effects: vec![],
            seed: self.rng.next_f32(),
        })
    }

    fn new_effect(&mut self, path: &Path) -> Option<EffectInst> {
        let (info, _) = isf::parse_info(path).ok()?;
        Some(EffectInst {
            uid: self.s.new_id(),
            path: path.to_path_buf(),
            active: true,
            mix: Mix::default(),
            params: isf::default_params(&info),
            seed: self.rng.next_f32(),
            open: false,
        })
    }

    fn add_effect(&mut self, owner: Owner, path: &Path) {
        let Some(fx) = self.new_effect(path) else { return };
        match owner {
            Owner::Instance(id) => {
                if let Some(inst) = self.s.find_mut(id) {
                    if inst.effects.len() < MAX_EFFECTS {
                        inst.effects.push(fx);
                    } else {
                        self.set_status(format!("max {MAX_EFFECTS} effects per clip"));
                    }
                }
            }
            Owner::Output => self.s.output_effects.push(fx),
        }
    }

    fn effects_mut(&mut self, owner: Owner) -> Option<&mut Vec<EffectInst>> {
        match owner {
            Owner::Instance(id) => self.s.find_mut(id).map(|i| &mut i.effects),
            Owner::Output => Some(&mut self.s.output_effects),
        }
    }

    /// double-click in finder / drop on empty slot
    fn add_file_to_slot(&mut self, path: &Path, kind: &FileKind, slot: Option<usize>) {
        if *kind == FileKind::Effect {
            if let Some(id) = self.s.selected_instance {
                self.add_effect(Owner::Instance(id), path);
            } else {
                self.set_status("select an instance in prepare first (effects go onto a clip)");
            }
            return;
        }
        if let Some(inst) = self.new_instance_from_file(path, kind) {
            let slot = match slot {
                Some(s) if s < self.s.slots.len() && self.s.slots[s].is_none() => s,
                _ => self.s.first_free_slot(),
            };
            let id = inst.id;
            self.s.slots[slot] = Some(inst);
            self.s.selected_instance = Some(id);
            self.s.page = slot / SLOTS_PER_PAGE;
        }
    }

    fn tap(&mut self) {
        let now = Instant::now();
        if self.taps.last().is_some_and(|t| now.duration_since(*t).as_secs_f32() > 2.0) {
            self.taps.clear();
        }
        self.taps.push(now);
        if self.taps.len() > 5 {
            self.taps.remove(0);
        }
        if self.taps.len() >= 2 {
            let span = now.duration_since(self.taps[0]).as_secs_f32();
            let bpm = 60.0 * (self.taps.len() - 1) as f32 / span;
            self.s.bpm = bpm.round().clamp(20.0, 300.0);
        }
        self.beat = self.beat.floor() + 1.0; // tap = downbeat
    }

    // ------------------------------------------------------------------ frame

    fn render(&mut self, gl: &std::sync::Arc<glow::Context>) {
        // MIDI first: a pad pressed before this frame is already in it
        if let Some(mut m) = self.midi.take() {
            m.poll(self);
            self.midi = Some(m);
        }
        let (w, h) = (self.s.output.width as i32, self.s.output.height as i32);
        if self.r.is_none() {
            use glow::HasContext;
            let (vendor, renderer) = unsafe { (gl.get_parameter_string(glow::VENDOR), gl.get_parameter_string(glow::RENDERER)) };
            eprintln!("gpu: {renderer} ({vendor})");
            self.set_status(format!("gpu: {renderer}"));
            self.gpu_name = renderer;
        }
        let fps = self.s.output.fps.clamp(1, 240);
        let r = self.r.get_or_insert_with(|| Renderer::new(gl, w, h));
        r.resize(gl, w, h);
        r.set_ndi(self.s.output.ndi_enabled, &self.s.output.ndi_name, &self.s.output.ndi_lib_dir, fps as i32);
        self.output_aspect = w as f32 / h as f32;

        // output frame rate: video (instances, mixer, output, NDI) is rendered at the chosen fps,
        // the ui itself repaints at the display rate
        let now = Instant::now();
        let interval = std::time::Duration::from_secs_f64(1.0 / fps as f64);
        let due = self.next_video.unwrap_or(now);
        // up to half a frame early is fine: ui repaints come at the display rate, and the
        // average stays exact because `due` advances by exactly one interval per frame
        if now + interval / 2 < due {
            return;
        }
        // missed frames: later than one whole interval
        let late = now.saturating_duration_since(due);
        if late >= interval {
            self.frames_dropped += (late.as_secs_f64() / interval.as_secs_f64()) as u64;
            self.last_drop = Some(now);
        }
        self.frame_times.push_back(now);
        while self.frame_times.front().is_some_and(|t| now.duration_since(*t).as_secs_f32() > 1.0) {
            self.frame_times.pop_front();
        }
        // keep the average rate exact; resync after a stall
        self.next_video = Some(if now.duration_since(due) > interval * 2 { now + interval } else { due + interval });
        let dt = now.duration_since(self.last).as_secs_f32().min(0.1);
        self.last = now;
        self.video_frames += 1;
        let beat_prev = self.beat;
        self.beat += dt as f64 * self.s.bpm as f64 / 60.0;

        // NDI discovery: start as soon as libndi can be loaded (retry every 5 s)
        if self.ndi_finder.is_none() && self.ndi_finder_try.is_none_or(|t| t.elapsed().as_secs() >= 5) {
            self.ndi_finder_try = Some(Instant::now());
            if let Ok(lib) = crate::ndi::lib(&self.s.output.ndi_lib_dir) {
                self.ndi_finder = Some(crate::ndi::NdiFinder::new(lib));
            }
        }

        // ---- 1. live instances (main mix), each with its own clock
        let now_beat = self.beat;
        let bpm = self.s.bpm;
        let live: Vec<u64> = self.s.layers.iter().filter_map(|l| l.instance).collect();
        let mut layer_tex = [None; NUM_LAYERS];
        for id in &live {
            let (idt, ibeat) = inst_clock(&mut self.inst_clock, *id, now, dt, beat_prev, now_beat);
            if let Some(inst) = self.s.slots.iter_mut().flatten().find(|i| i.id == *id) {
                let mut fc = FrameCtx { dt: idt, bpm, beat_prev: ibeat, beat_now: now_beat, rng: &mut self.rng };
                let tex = r.render_instance(gl, inst, &mut fc);
                for (li, l) in self.s.layers.iter().enumerate() {
                    if l.instance == Some(*id) {
                        layer_tex[li] = tex;
                    }
                }
            }
        }
        // recording: start / stop (asked by the REC button), errors end it
        match self.rec_request.take() {
            Some(true) if r.rec.is_none() => {
                let path = crate::record::new_file_name(&self.cfg.record_dir());
                match r.start_recording(gl, &path, fps) {
                    Ok(()) => {
                        self.rec_size = (None, 0);
                        self.status = format!("recording to {}", path.display());
                        self.status_time = Instant::now();
                    }
                    Err(e) => {
                        self.status = e;
                        self.status_time = Instant::now();
                    }
                }
            }
            Some(false) => {
                if let Some(rec) = r.rec.as_ref() {
                    self.status = format!("recorded {}", rec.path.display());
                    self.status_time = Instant::now();
                }
                r.stop_recording(gl);
            }
            _ => {}
        }
        if let Some(e) = r.rec.as_ref().and_then(|x| x.error.clone()) {
            self.status = e;
            self.status_time = Instant::now();
            r.stop_recording(gl);
        }
        // GPU timing always runs (also with the meters hidden): the prepare priority needs it;
        // its cost is not measurable (every 8th frame)
        r.meter.enabled = true;
        // ---- 2. main mix: mixer, output effects, master, NDI, REC
        let mut fc = FrameCtx { dt, bpm, beat_prev, beat_now: now_beat, rng: &mut self.rng };
        r.render_output(gl, &mut self.s, layer_tex, &mut fc);
        if self.cfg.meters {
            self.proc_stats.sample(gl);
        }

        // ---- 3. prepare work (selected non-live instance, finder preview, thumbnails): only after
        // the main mix, at a reduced rate that drops further when the main mix gets tight
        let main_ms = {
            let m = &r.meter;
            let ms = |k| m.ms(k).unwrap_or(0.0);
            let mut sum = ms(Key::Mixer) + ms(Key::Master) + ms(Key::Record);
            sum += self.s.output_effects.iter().filter(|e| e.active).map(|e| ms(Key::Fx(e.uid))).sum::<f32>();
            for inst in self.s.slots.iter().flatten().filter(|i| live.contains(&i.id)) {
                sum += ms(Key::Src(inst.id)) + inst.effects.iter().filter(|e| e.active).map(|e| ms(Key::Fx(e.uid))).sum::<f32>();
            }
            sum
        };
        let budget = 1000.0 / fps as f32;
        // a dropped frame counts only when our own GPU load is significant (on Wayland the
        // compositor alone can drop frames; pausing the preview would not help there)
        let drop_counts = self.last_drop.is_some_and(|t| t.elapsed().as_secs_f32() < 2.0) && r.meter.total_ms > budget * 0.5;
        let p = main_ms / budget;
        let mut want: u32 = if p < 0.6 { 2 } else if p < 0.8 { 4 } else if p < 0.95 { 8 } else { 0 };
        if drop_counts && want == 2 {
            want = 4;
        }
        // slower at once, faster again only after 2 s of calm (no flickering between rates)
        let slower = |a: u32, b: u32| if a == 0 { u32::MAX } else { a } > if b == 0 { u32::MAX } else { b };
        if slower(want, self.prepare_div) {
            self.prepare_div = want;
            self.prepare_div_since = Instant::now();
        } else if want != self.prepare_div && self.prepare_div_since.elapsed().as_secs_f32() > 2.0 {
            self.prepare_div = want;
            self.prepare_div_since = Instant::now();
        }
        self.main_ms = main_ms;
        // self-test comparison: the old behaviour (prepare every frame)
        if std::env::var_os("VIDEOPING_SELFTEST_NOPRIORITY").is_some() && self.selftest.is_some() {
            self.prepare_div = 1;
        }
        let div = self.prepare_div as u64;
        if div > 0 && self.video_frames.is_multiple_of(div) {
            let mut prep: Vec<u64> = vec![];
            if let Some(sel) = self.s.selected_instance.filter(|id| !live.contains(id)) {
                prep.push(sel);
            }
            // thumbnails of slots that never rendered: one at a time, only without pressure
            if div == 2 {
                if let Some(inst) = self.s.slots.iter().flatten().find(|i| {
                    !live.contains(&i.id) && !prep.contains(&i.id) && r.thumb(i.id).is_none() && r.player_error(i.id).is_none()
                }) {
                    prep.push(inst.id);
                }
            }
            for id in prep {
                if Some(id) == self.s.selected_instance {
                    self.prepare_renders += 1;
                }
                let (idt, ibeat) = inst_clock(&mut self.inst_clock, id, now, dt, beat_prev, now_beat);
                if let Some(inst) = self.s.slots.iter_mut().flatten().find(|i| i.id == id) {
                    let mut fc = FrameCtx { dt: idt, bpm, beat_prev: ibeat, beat_now: now_beat, rng: &mut self.rng };
                    r.render_instance(gl, inst, &mut fc);
                }
            }
            if let Some(pv) = self.preview.as_mut() {
                let (idt, ibeat) = inst_clock(&mut self.inst_clock, pv.id, now, dt, beat_prev, now_beat);
                let mut fc = FrameCtx { dt: idt, bpm, beat_prev: ibeat, beat_now: now_beat, rng: &mut self.rng };
                r.render_instance(gl, pv, &mut fc);
            }
            crate::gfx::reset_state(gl);
        }

        if self.last_gc.elapsed().as_secs() >= 2 {
            self.last_gc = Instant::now();
            r.gc(gl, &self.s);
            self.inst_clock.retain(|_, (t, _)| t.elapsed().as_secs() < 10);
        }

        // shader infos for the ui (labels, ranges)
        let mut paths: Vec<PathBuf> = vec![];
        for inst in self.s.slots.iter().flatten() {
            if let SourceKind::Generator { path, .. } = &inst.source {
                paths.push(path.clone());
            }
            paths.extend(inst.effects.iter().map(|e| e.path.clone()));
        }
        paths.extend(self.s.output_effects.iter().map(|e| e.path.clone()));
        if let Some(Instance { source: SourceKind::Generator { path, .. }, .. }) = &self.preview {
            paths.push(path.clone());
        }
        for p in paths {
            if let Some(info) = r.shader_info(gl, &p) {
                self.infos.insert(p, info.clone());
            }
        }
    }

    fn update_preview(&mut self, gl: &glow::Context) {
        let key = self.finder_sel.as_ref().map(|(p, k)| (p.clone(), if *k == FileKind::Effect { self.s.selected_instance } else { None }));
        if key == self.preview_key {
            return;
        }
        self.preview_key = key;
        if let Some(r) = self.r.as_mut() {
            r.drop_instance(gl, PREVIEW_ID);
        }
        self.preview = None;
        let Some((path, kind)) = self.finder_sel.clone() else { return };
        let inst = match kind {
            FileKind::Effect => {
                // preview an effect on the selected instance
                let base = self.s.selected_instance.and_then(|id| self.s.find(id)).cloned();
                let fx = self.new_effect(&path);
                match (base, fx) {
                    (Some(mut b), Some(fx)) => {
                        b.effects.push(fx);
                        Some(b)
                    }
                    _ => None,
                }
            }
            k => self.new_instance_from_file(&path, &k),
        };
        self.preview = inst.map(|mut i| {
            i.id = PREVIEW_ID;
            i
        });
    }

    fn autosave(&mut self) {
        if self.last_autosave.elapsed().as_secs() < 2 {
            return;
        }
        self.last_autosave = Instant::now();
        if let Ok(txt) = serde_json::to_string(&self.s) {
            if txt != self.last_saved {
                match save_session(&self.s, &self.autosave_path) {
                    Ok(_) => self.last_saved = txt,
                    Err(e) => self.set_status(format!("autosave failed: {e}")),
                }
            }
        }
    }

    // --------------------------------------------------------------------- ui

    /// Replace the whole session (load / new): drops all instance gpu state and the preview.
    fn replace_session(&mut self, s: Session, gl: &glow::Context) {
        self.s = s;
        if let Some(r) = self.r.as_mut() {
            r.drop_all_instances(gl);
        }
        self.preview = None;
        self.preview_key = None;
    }

    /// "new": clear all content, keep the setup (finder folders, output options).
    fn new_session(&mut self, gl: &glow::Context) {
        let mut s = Session::default();
        s.finder_roots = self.s.finder_roots.clone();
        s.output = self.s.output.clone();
        self.replace_session(s, gl);
        self.set_status("new session");
    }

    /// Confirmation dialog for "new".
    fn confirm_new_dialog(&mut self, ctx: &egui::Context, gl: &glow::Context) {
        if !self.confirm_new {
            return;
        }
        let mut clear = false;
        let mut cancel = false;
        let resp = egui::Modal::new(egui::Id::unique("confirm_new")).show(ctx, |ui| {
            ui.set_width(320.0);
            ui.label(egui::RichText::new("start a new session?").strong());
            ui.add_space(4.0);
            ui.label("clears the prepare grid, mixer, effects, output effects, bpm and master.");
            ui.label(egui::RichText::new("kept: finder folders and output options. unsaved changes are lost.").weak());
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let r = widgets::choice(ui, true, "clear everything");
                if let Some(t) = self.selftest.as_mut() {
                    t.clear_btn = Some(r.rect.center());
                }
                if r.clicked() {
                    clear = true;
                }
                if widgets::btn(ui, "cancel").clicked() {
                    cancel = true;
                }
            });
        });
        if clear {
            self.new_session(gl);
            self.confirm_new = false;
        } else if cancel || resp.should_close() {
            self.confirm_new = false;
        }
    }

    fn top_bar(&mut self, ui: &mut Ui, gl: &glow::Context) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("videoping").strong().color(ACCENT));
            ui.separator();
            if widgets::choice(ui, self.tab == Tab::Live, "live").clicked() {
                self.tab = Tab::Live;
            }
            ui.add_enabled(false, egui::Button::new(egui::RichText::new("mapping").size(widgets::BTN_TEXT)).min_size(widgets::BTN));
            ui.add_enabled(false, egui::Button::new(egui::RichText::new("shader editor").size(widgets::BTN_TEXT)).min_size(widgets::BTN));
            if widgets::choice(ui, self.tab == Tab::Midi, "midi mapping").clicked() {
                self.tab = Tab::Midi;
            }
            ui.add_enabled(false, egui::Button::new(egui::RichText::new("sequencer").size(widgets::BTN_TEXT)).min_size(widgets::BTN));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::btn(ui, "load").clicked() {
                    let p = PathBuf::from(self.file_path.trim());
                    match load_session(&p) {
                        Ok(s) => {
                            self.replace_session(s, gl);
                            self.set_status(format!("loaded {}", p.display()));
                        }
                        Err(e) => self.set_status(e),
                    }
                }
                if widgets::btn(ui, "save").clicked() {
                    let p = PathBuf::from(self.file_path.trim());
                    match save_session(&self.s, &p) {
                        Ok(_) => self.set_status(format!("saved {}", p.display())),
                        Err(e) => self.set_status(e),
                    }
                }
                if widgets::btn(ui, "new").on_hover_text("clear everything (asks first)").clicked() {
                    self.confirm_new = true;
                }
                ui.add(egui::TextEdit::singleline(&mut self.file_path).desired_width(260.0));
                ui.separator();
                self.fps_counter(ui);
                ui.separator();
                if self.cfg.meters {
                    self.usage_total(ui);
                    ui.separator();
                }
                if self.status_time.elapsed().as_secs() < 8 {
                    // only the space that is left (a long path must not run over the tabs)
                    ui.add(egui::Label::new(egui::RichText::new(&self.status).weak()).truncate()).on_hover_text(&self.status);
                }
            });
        });
    }

    /// Output frame rate: measured (last second) / target, plus dropped frames.
    /// Grey when on target; bright with ⚠ when below or a frame was missed in the last 2 s
    /// (no color: colors are reserved for the layers).
    fn fps_counter(&mut self, ui: &mut Ui) {
        let target = self.s.output.fps.clamp(1, 240);
        let measured = self.frame_times.len() as u32;
        let warming_up = self.frame_times.front().is_none_or(|t| t.elapsed().as_secs_f32() < 0.9);
        let recent_drop = self.last_drop.is_some_and(|t| t.elapsed().as_secs_f32() < 2.0);
        let low = !warming_up && (measured as f32) < target as f32 * 0.95;
        let bad = low || recent_drop;
        let txt = format!("{}{measured:>3} / {target} fps", if bad { "⚠ " } else { "" });
        let rich = egui::RichText::new(txt).monospace().size(widgets::BTN_TEXT);
        let rich = if bad { rich.strong().color(Color32::WHITE) } else { rich.color(Color32::from_gray(150)) };
        ui.label(rich).on_hover_text(format!(
            "output frame rate: measured over the last second / target\nframes dropped since start: {}\n\
             on Wayland the compositor can limit this below the target (see install.md)",
            self.frames_dropped
        ));
    }

    // ------------------------------------------------------------ control registry
    // MIDI, MCP (and later OSC) control videoping through these, see control.rs

    fn control_ext(&self) -> crate::control::Ext {
        crate::control::Ext { recording: self.r.as_ref().is_some_and(|r| r.rec.is_some()) }
    }

    /// Every address that exists right now.
    pub fn control_list(&self) -> Vec<crate::control::Desc> {
        crate::control::list(&self.s, &self.infos)
    }

    pub fn control_get(&self, addr: &str) -> Option<crate::control::Value> {
        crate::control::get(&self.s, &self.infos, self.control_ext(), addr)
    }

    /// Set an address; tap and record go through the app like the buttons.
    pub fn control_set(&mut self, addr: &str, value: &crate::control::Value) -> Result<(), String> {
        match crate::control::set(&mut self.s, &self.infos, addr, value)? {
            crate::control::Effect::None => {}
            crate::control::Effect::Tap => self.tap(),
            crate::control::Effect::Record(on) => {
                if on != self.control_ext().recording {
                    self.rec_request = Some(on);
                }
            }
        }
        Ok(())
    }

    /// Addresses whose value changed since the last call (also by mouse): LED feedback, MCP.
    pub fn control_changes(&mut self) -> Vec<(String, crate::control::Value)> {
        let now = crate::control::snapshot(&self.s, &self.infos, self.control_ext());
        let ch = crate::control::changes(&self.control_snap, &now);
        self.control_snap = now;
        ch
    }

    /// "/page/slot/9/..." -> the absolute "/slot/N/..." for the page shown (other addresses as they are)
    pub fn page_slot_resolve(&self, addr: &str) -> String {
        crate::control::page_alias(&self.s, addr).unwrap_or_else(|| addr.to_string())
    }

    /// min / max of a range address
    pub fn control_range(&self, addr: &str) -> Option<(f32, f32)> {
        crate::control::range(&self.s, &self.infos, addr)
    }

    /// Frame budget in ms at the output frame rate.
    fn budget_ms(&self) -> f32 {
        1000.0 / self.s.output.fps.clamp(1, 240) as f32
    }

    /// Total resource use next to the fps counter: GPU ms per frame, own VRAM, process CPU / RAM.
    fn usage_total(&self, ui: &mut Ui) {
        let Some(r) = self.r.as_ref() else { return };
        let budget = self.budget_ms();
        let gpu = r.meter.total_ms;
        let vram = r.vram_total();
        let p = &self.proc_stats;
        let txt = format!("GPU {:.1} ms · VRAM {} · CPU {:.0} %", gpu, meter::mb(vram), p.cpu_percent);
        let mut tip = format!(
            "GPU: {gpu:.1} ms per frame ({:.0} % of the {budget:.1} ms frame budget)\n\
             VRAM: {} allocated by videoping (frame buffers)\n\
             CPU: {:.0} % of one core ({} cores), mostly video decoding\n\
             RAM: {}",
            gpu / budget * 100.0,
            meter::mb(vram),
            p.cpu_percent,
            p.cores,
            meter::mb(p.rss_bytes),
        );
        if let Some((used, total)) = p.gpu_mem {
            tip.push_str(&format!("\nGPU memory (all programs): {} of {}", meter::mb(used), meter::mb(total)));
        }
        ui.label(egui::RichText::new(txt).monospace().size(widgets::BTN_TEXT).color(Color32::from_gray(150))).on_hover_text(tip);
    }

    /// Level 1..3 and tooltip of an instance (source + its active effects), None when it is not rendered.
    fn inst_usage(&self, inst: &Instance) -> Option<(u8, String)> {
        if !self.cfg.meters {
            return None;
        }
        let r = self.r.as_ref()?;
        let budget = self.budget_ms();
        let src_ms = r.meter.ms(Key::Src(inst.id))?;
        let fx: Vec<(f32, u64)> = inst
            .effects
            .iter()
            .filter(|e| e.active)
            .map(|e| (r.meter.ms(Key::Fx(e.uid)).unwrap_or(0.0), r.vram_fx(e.uid)))
            .collect();
        let ms = src_ms + fx.iter().map(|f| f.0).sum::<f32>();
        let vram = r.vram_inst(inst.id) + fx.iter().map(|f| f.1).sum::<u64>();
        let mut level = meter::gpu_level(ms, budget).max(meter::vram_level(vram));
        let mut tip = format!(
            "GPU {ms:.2} ms ({:.0} % of {budget:.1} ms), source {src_ms:.2} ms + {} effects\nVRAM {}",
            ms / budget * 100.0,
            fx.len(),
            meter::mb(vram)
        );
        if let Some(d) = r.decode_info(inst.id) {
            level = level.max(d.level());
            tip.push_str(&format!(
                "\ndecoding: {} ({}), {}x{} {:.0} fps",
                if d.decoder.is_empty() { "?" } else { &d.decoder },
                if d.hardware { "hardware" } else { "software, CPU" },
                d.w,
                d.h,
                d.fps
            ));
            if d.expected() > 0.0 {
                tip.push_str(&format!(", delivers {:.0} of {:.0} fps", d.delivered, d.expected()));
            }
            if d.behind() {
                tip.push_str("\n⚠ decoder does not keep up");
            }
        }
        Some((level, tip))
    }

    fn rescan_finder(&mut self) {
        let (f, dirs) = scan_with_dirs(&self.s.finder_roots);
        self.finder = f;
        self.finder_watch = (self.s.finder_roots.clone(), dirs);
        self.finder_checked = Instant::now();
    }

    /// Every 2 s: rescan when a finder folder changed on disk (new recording, copied files) or
    /// the folders themselves changed (loaded session). Only stats the known directories.
    fn watch_finder(&mut self) {
        if self.finder_checked.elapsed().as_secs() < 2 {
            return;
        }
        self.finder_checked = Instant::now();
        let changed = self.finder_watch.0 != self.s.finder_roots || self.finder_watch.1.iter().any(|(d, t)| mtime(d) != *t);
        if changed {
            self.rescan_finder();
        }
    }

    fn col_finder(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        self.watch_finder();
        // finder preview
        let w = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(vec2(w, w / self.output_aspect), Sense::hover());
        let tex = self.r.as_ref().and_then(|r| r.output_of(PREVIEW_ID));
        let id = self.tex_id(frame, tex);
        paint_tex(ui, rect, id, self.output_aspect);
        if self.preview.is_none() {
            let msg = match &self.finder_sel {
                Some((_, FileKind::Effect)) => "select an instance in prepare to preview this effect",
                _ => "click something in the finder",
            };
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, msg, egui::FontId::proportional(11.0), Color32::GRAY);
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.finder_search).hint_text("search").desired_width(field_width(ui, widgets::BTN.x)));
            if widgets::btn(ui, "↻").on_hover_text("rescan folders").clicked() {
                self.rescan_finder();
            }
        });
        egui::CollapsingHeader::new("folders").default_open(false).show(ui, |ui| {
            let mut remove = None;
            for (i, r) in self.s.finder_roots.iter().enumerate() {
                ui.horizontal(|ui| {
                    if widgets::btn(ui, "×").clicked() {
                        remove = Some(i);
                    }
                    ui.label(r.display().to_string());
                });
            }
            if let Some(i) = remove {
                self.s.finder_roots.remove(i);
                self.rescan_finder();
            }
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.finder_new_root).hint_text("/path/to/folder").desired_width(field_width(ui, widgets::BTN.x + 8.0)));
                if widgets::btn(ui, "add").clicked() {
                    let p = PathBuf::from(self.finder_new_root.trim());
                    if p.is_dir() {
                        self.s.finder_roots.push(p);
                        self.finder_new_root.clear();
                        self.rescan_finder();
                    } else {
                        self.set_status("not a folder");
                    }
                }
            });
        });
        ui.separator();
        let search = self.finder_search.to_lowercase();
        let mut action: Option<(PathBuf, FileKind, bool)> = None; // (path, kind, double-click)
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for (title, kind) in [("clips", FileKind::Clip), ("generators", FileKind::Generator), ("effects", FileKind::Effect)] {
                egui::CollapsingHeader::new(title).default_open(true).show(ui, |ui| {
                    for e in self.finder.iter().filter(|e| e.kind == kind) {
                        if !search.is_empty() && !e.name.to_lowercase().contains(&search) {
                            continue;
                        }
                        let selected = self.finder_sel.as_ref().is_some_and(|s| s.0 == e.path);
                        let r = ui
                            .add(egui::Button::selectable(selected, &e.name))
                            .interact(Sense::click_and_drag())
                            .on_hover_text(e.path.display().to_string());
                        r.dnd_set_drag_payload(Drag::File(e.path.clone(), e.kind.clone()));
                        if r.double_clicked() {
                            action = Some((e.path.clone(), e.kind.clone(), true));
                        } else if r.clicked() {
                            action = Some((e.path.clone(), e.kind.clone(), false));
                        }
                    }
                });
            }
            egui::CollapsingHeader::new("NDI sources").default_open(true).show(ui, |ui| match &self.ndi_finder {
                None => {
                    ui.label(egui::RichText::new("libndi not loaded: set \"NDI library folder\" in the output options").weak());
                }
                Some(f) => {
                    let sources = f.sources();
                    if sources.is_empty() {
                        ui.label(egui::RichText::new("searching…").weak());
                    }
                    for name in sources {
                        if !search.is_empty() && !name.to_lowercase().contains(&search) {
                            continue;
                        }
                        let path = PathBuf::from(&name);
                        let selected = self.finder_sel.as_ref().is_some_and(|s| s.0 == path);
                        let r = ui.add(egui::Button::selectable(selected, &name)).interact(Sense::click_and_drag());
                        r.dnd_set_drag_payload(Drag::File(path.clone(), FileKind::Ndi));
                        if r.double_clicked() {
                            action = Some((path, FileKind::Ndi, true));
                        } else if r.clicked() {
                            action = Some((path, FileKind::Ndi, false));
                        }
                    }
                }
            });
        });
        if let Some((p, k, dbl)) = action {
            self.finder_sel = Some((p.clone(), k.clone()));
            if dbl {
                self.add_file_to_slot(&p, &k, None);
            }
        }
    }

    /// column 2: prepare grid, below it master, bpm, record, output options and general config
    fn col_grid(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        egui::ScrollArea::vertical().id_salt("gridcol").auto_shrink([false, false]).show(ui, |ui| {
            self.prepare_grid(ui, frame);
            ui.add_space(6.0);
            ui.separator();
            self.section_master(ui);
            ui.add_space(6.0);
            ui.separator();
            self.section_bpm(ui);
            ui.add_space(6.0);
            ui.separator();
            self.section_record(ui);
            ui.add_space(6.0);
            ui.separator();
            self.section_output_options(ui);
            ui.add_space(6.0);
            ui.separator();
            self.section_general_config(ui);
        });
    }

    fn prepare_grid(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let pages = self.s.pages();
        ui.horizontal(|ui| {
            ui.label("prepare");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::btn(ui, "+").on_hover_text("add page").clicked() {
                    self.s.slots.extend(std::iter::repeat_n(None, SLOTS_PER_PAGE));
                    self.s.page = self.s.pages() - 1;
                }
                if widgets::btn(ui, "▶").clicked() && self.s.page + 1 < pages {
                    self.s.page += 1;
                }
                ui.label(format!("page {}/{}", self.s.page + 1, pages));
                if widgets::btn(ui, "◀").clicked() && self.s.page > 0 {
                    self.s.page -= 1;
                }
            });
        });
        let gap = 4.0;
        let cell_w = ((ui.available_width() - gap * (GRID_COLS as f32 - 1.0)) / GRID_COLS as f32).max(40.0);
        let thumb_h = cell_w * 9.0 / 16.0;
        let cell_h = thumb_h + 14.0;
        let origin = ui.cursor().min;
        let (_, _) = ui.allocate_exact_size(vec2(ui.available_width(), (cell_h + gap) * GRID_ROWS as f32), Sense::hover());
        let base = self.s.page * SLOTS_PER_PAGE;
        let mut ev_drop: Option<(usize, Drag)> = None;
        let mut ev_click = None;
        let mut ev_click_empty = false;
        let mut ev_dbl = None;
        let mut ev_delete = None;
        for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                // index runs top-down per column, like the bhoreal columns 2-5
                let slot = base + col * GRID_ROWS + row;
                let rect = Rect::from_min_size(origin + vec2(col as f32 * (cell_w + gap), row as f32 * (cell_h + gap)), vec2(cell_w, cell_h));
                let resp = ui.interact(rect, ui.scope_id().with(("slot", slot)), Sense::click_and_drag());
                let inst = self.s.slots.get(slot).and_then(|s| s.as_ref());
                let thumb_rect = Rect::from_min_size(rect.min, vec2(cell_w, thumb_h));
                ui.painter().rect_filled(rect, 3.0, ui.visuals().extreme_bg_color);
                if let Some(inst) = inst {
                    let id = inst.id;
                    let name = inst.name.clone();
                    let badge = source_badge(&inst.source);
                    let usage = self.inst_usage(inst);
                    let live = self.s.live_layer(id);
                    let selected = self.s.selected_instance == Some(id);
                    let tex = self.r.as_ref().and_then(|r| if live.is_some() || selected { r.output_of(id) } else { r.thumb(id) });
                    let tid = self.tex_id(frame, tex);
                    paint_tex(ui, thumb_rect, tid, self.output_aspect);
                    // type badge top left: ▶ clip, ∿ generator, NDI (neutral grey: colors belong to the layers)
                    {
                        let font = egui::FontId::proportional(11.0);
                        let galley = ui.painter().layout_no_wrap(badge.to_string(), font, Color32::from_gray(210));
                        let br = Rect::from_min_size(thumb_rect.left_top() + vec2(2.0, 2.0), galley.size() + vec2(6.0, 2.0));
                        ui.painter().rect_filled(br, 2.0, Color32::from_black_alpha(170));
                        ui.painter().galley(br.min + vec2(3.0, 1.0), galley, Color32::from_gray(210));
                    }
                    // resource meter bottom left of the thumbnail (only while rendered)
                    if let Some((level, tip)) = usage {
                        let mr = Rect::from_min_size(thumb_rect.left_bottom() + vec2(2.0, -widgets::METER_SIZE.y - 6.0), widgets::METER_SIZE + vec2(4.0, 4.0));
                        ui.painter().rect_filled(mr, 2.0, Color32::from_black_alpha(170));
                        widgets::paint_meter(ui.painter(), mr.shrink(2.0), level);
                        if ui.rect_contains_pointer(mr) {
                            egui::Tooltip::always_open(ui.ctx().clone(), ui.layer_id(), ui.scope_id().with(("meter", slot)), egui::PopupAnchor::Pointer)
                                .show(|ui| ui.label(tip));
                        }
                    }
                    if let Some(err) = self.r.as_ref().and_then(|r| r.player_error(id)) {
                        ui.painter().text(thumb_rect.center(), egui::Align2::CENTER_CENTER, "error", egui::FontId::proportional(11.0), Color32::RED);
                        resp.clone().on_hover_text(err.to_string());
                    }
                    ui.painter().text(
                        rect.left_bottom() + vec2(3.0, -2.0),
                        egui::Align2::LEFT_BOTTOM,
                        truncate(&name, (cell_w / 6.5) as usize),
                        egui::FontId::proportional(11.0),
                        ui.visuals().text_color(),
                    );
                    if let Some(l) = live {
                        widgets::outline(ui, rect, 3.0, layer_color(l));
                        ui.painter().text(rect.right_top() + vec2(-4.0, 2.0), egui::Align2::RIGHT_TOP, format!("L{}", l + 1), egui::FontId::monospace(11.0), layer_color(l));
                    } else if selected {
                        widgets::outline(ui, rect, 1.5, Color32::WHITE);
                    } else {
                        // loaded but not live / not selected: same thin border as an empty slot
                        widgets::outline(ui, rect, 1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
                    }
                    resp.dnd_set_drag_payload(Drag::Slot(slot));
                    if resp.double_clicked() {
                        ev_dbl = Some(id);
                    } else if resp.clicked() {
                        ev_click = Some(id);
                    }
                    // x to delete (on hover); not for instances that are live in the mixer
                    if live.is_none() && (resp.hovered() || ui.rect_contains_pointer(rect)) {
                        let xr = Rect::from_min_size(rect.right_top() + vec2(-16.0, rect.height() - 16.0), vec2(16.0, 16.0));
                        let xresp = ui.interact(xr, ui.scope_id().with(("slotx", slot)), Sense::click());
                        let c = if xresp.hovered() { Color32::RED } else { Color32::GRAY };
                        ui.painter().text(xr.center(), egui::Align2::CENTER_CENTER, "×", egui::FontId::proportional(14.0), c);
                        if xresp.clicked() {
                            ev_delete = Some(slot);
                        }
                    }
                } else {
                    widgets::outline(ui, rect, 1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
                    // click on an empty slot: nothing selected -> empty prepare detail
                    if resp.clicked() {
                        ev_click_empty = true;
                    }
                    if slot == 24 {
                        if let Some(t) = self.selftest.as_mut() {
                            t.empty_slot = Some(rect.center());
                        }
                    }
                }
                if let Some(p) = resp.dnd_hover_payload::<Drag>() {
                    // a new clip / generator / NDI source can't be dropped onto a live slot
                    let blocked = matches!(&*p, Drag::File(_, k) if *k != FileKind::Effect) && self.s.slot_is_live(slot);
                    if !blocked {
                        widgets::outline(ui, rect, 2.0, ACCENT);
                    }
                }
                if let Some(p) = resp.dnd_release_payload::<Drag>() {
                    ev_drop = Some((slot, (*p).clone()));
                }
            }
        }
        if let Some(id) = ev_click {
            self.s.selected_instance = Some(id);
        } else if ev_click_empty {
            self.s.selected_instance = None;
        }
        if let Some(id) = ev_dbl {
            self.s.selected_instance = Some(id);
            self.s.load_into_selected_layer(id);
        }
        if let Some(slot) = ev_delete {
            if !self.s.delete_slot(slot) {
                self.set_status("live in the mixer: unload it from the layer first");
            }
        }
        if let Some((slot, d)) = ev_drop {
            match d {
                Drag::Slot(from) => self.s.swap_slots(from, slot),
                Drag::File(path, FileKind::Effect) => {
                    if let Some(id) = self.s.slots.get(slot).and_then(|s| s.as_ref()).map(|i| i.id) {
                        self.add_effect(Owner::Instance(id), &path);
                        self.s.selected_instance = Some(id);
                    }
                }
                Drag::File(path, kind) => {
                    // empty slot: new instance; occupied but not live: replaced; live: not possible
                    if self.s.slot_is_live(slot) {
                        self.set_status("live in the mixer: can't replace it");
                    } else if slot < self.s.slots.len() {
                        self.s.delete_slot(slot);
                        self.add_file_to_slot(&path, &kind, Some(slot));
                    }
                }
                Drag::Effect(..) => {}
            }
        }
    }

    /// pinned top of column 3: prepare preview + name / live / duplicate.
    /// Returns the instance whose body should be shown below.
    fn prepare_detail_header(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) -> Option<u64> {
        let Some(id) = self.s.selected_instance.filter(|id| self.s.find(*id).is_some()) else {
            // same empty preview area as with an instance: the layout stays the same
            let w = self.preview_width(ui, 0);
            let (rect, _) = ui.allocate_exact_size(vec2(w, w / self.output_aspect), Sense::hover());
            paint_tex(ui, rect, None, self.output_aspect);
            ui.label(egui::RichText::new("select an instance in the grid").weak());
            return None;
        };
        // prepare preview (same size as the main output preview)
        let w = self.preview_width(ui, 0);
        let (rect, _) = ui.allocate_exact_size(vec2(w, w / self.output_aspect), Sense::hover());
        let tex = self.r.as_ref().and_then(|r| r.output_of(id));
        let tid = self.tex_id(frame, tex);
        paint_tex(ui, rect, tid, self.output_aspect);
        let live = self.s.live_layer(id);
        if let Some(l) = live {
            widgets::outline(ui, rect, 3.0, layer_color(l));
        }
        let mut dup = false;
        ui.horizontal(|ui| {
            let inst = self.s.find(id).unwrap();
            ui.label(egui::RichText::new(&inst.name).strong());
            if let Some(l) = live {
                ui.label(egui::RichText::new(format!("live in L{}", l + 1)).color(layer_color(l)));
            } else {
                // not live: rendered with lower priority than the main mix
                let fps = self.s.output.fps.clamp(1, 240);
                let (txt, tip) = match self.prepare_div {
                    0 => ("paused: main mix has priority".to_string(), "the main mix needs the whole frame budget"),
                    d => (format!("prepare {} fps", fps / d), "not live: rendered after the main mix at a lower rate"),
                };
                let budget = 1000.0 / fps as f32;
                ui.label(egui::RichText::new(txt).weak().size(widgets::BTN_TEXT))
                    .on_hover_text(format!("{tip}\nmain mix: GPU {:.1} ms of {budget:.1} ms per frame", self.main_ms));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::btn(ui, "duplicate").clicked() {
                    dup = true;
                }
            });
        });
        if dup {
            if let Some(slot) = self.s.slot_of(id) {
                if let Some(ns) = self.s.duplicate_slot(slot, &mut self.rng) {
                    self.s.selected_instance = self.s.slots[ns].as_ref().map(|i| i.id);
                    self.s.page = ns / SLOTS_PER_PAGE;
                }
            }
            return None;
        }
        Some(id)
    }

    /// scrollable part of column 3: clip interface / generator params, effect stack
    fn prepare_detail_body(&mut self, ui: &mut Ui, id: u64) {
        if self.s.find(id).is_none() {
            return;
        }
        if let Some(e) = self.r.as_ref().and_then(|r| r.player_error(id)) {
            ui.colored_label(Color32::RED, e.to_string());
        }
        let status = self.r.as_ref().and_then(|r| r.clip_status.get(&id).copied()).unwrap_or_default();
        let ndi_status = self.r.as_ref().and_then(|r| r.ndi_status.get(&id).copied());
        let mut scrub = None;
        let infos = &self.infos;
        let r_ref = self.r.as_ref();
        if let Some(inst) = self.s.find_mut(id) {
            match &mut inst.source {
                SourceKind::Clip { transport, .. } => {
                    let tl_rect = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), 22.0));
                    scrub = widgets::timeline(ui, transport, status.position, status.duration);
                    widgets::transport_row(ui, &mut transport.mode, &mut transport.loop_mode, &mut transport.speed, "x");
                    if let Some(t) = self.selftest.as_mut() {
                        t.timeline = Some(tl_rect);
                    }
                }
                SourceKind::Ndi { source } => {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("NDI").strong());
                        ui.label(source.as_str());
                    });
                    match ndi_status {
                        Some(Some((w, h, fps))) => {
                            ui.label(format!("{w}x{h} @ {fps:.2} fps"));
                        }
                        _ => {
                            ui.label("no signal");
                        }
                    }
                }
                SourceKind::Generator { path, params } => {
                    if let Some(err) = r_ref.and_then(|r| r.shader_error(path)) {
                        ui.colored_label(Color32::RED, err);
                    }
                    if let Some(info) = infos.get(path) {
                        params_ui(ui, info, params, ("gen", id));
                    }
                }
            }
        }
        if let Some(n) = scrub {
            if let Some(r) = self.r.as_mut() {
                r.pending_scrub.push((id, n));
            }
        }
        ui.add_space(4.0);
        self.effect_stack(ui, Owner::Instance(id));
    }

    /// Effect stack (prepare detail and output effects).
    fn effect_stack(&mut self, ui: &mut Ui, owner: Owner) {
        let mut ev_move: Option<(usize, usize)> = None;
        let mut ev_remove = None;
        let mut ev_add: Option<PathBuf> = None;
        // effect dragged in from the other stack (clip <-> output): (from owner, from index, insert at)
        let mut ev_transfer: Option<(Owner, usize, Option<usize>)> = None;
        let mut first_handle: Option<egui::Pos2> = None;
        let infos = self.infos.clone();
        let shader_errors: Vec<Option<String>> = {
            let paths: Vec<PathBuf> = self.effects_mut(owner).map(|v| v.iter().map(|e| e.path.clone()).collect()).unwrap_or_default();
            paths.iter().map(|p| self.r.as_ref().and_then(|r| r.shader_error(p)).map(|s| s.to_string())).collect()
        };
        // resource meter per effect: (level, tooltip), None when not rendered
        let fx_usage: Vec<Option<(u8, String)>> = {
            let budget = self.budget_ms();
            let on = self.cfg.meters;
            let uids: Vec<u64> = self.effects_mut(owner).map(|v| v.iter().map(|e| e.uid).collect()).unwrap_or_default();
            uids.iter()
                .map(|uid| {
                    let r = self.r.as_ref().filter(|_| on)?;
                    let ms = r.meter.ms(Key::Fx(*uid))?;
                    let vram = r.vram_fx(*uid);
                    let level = meter::gpu_level(ms, budget).max(meter::vram_level(vram));
                    let tip = format!("GPU {ms:.2} ms ({:.0} % of {budget:.1} ms)\nVRAM {} (own pass buffers)", ms / budget * 100.0, meter::mb(vram));
                    Some((level, tip))
                })
                .collect()
        };
        let count = self.effects_mut(owner).map(|v| v.len()).unwrap_or(0);
        let header = match owner {
            Owner::Instance(_) => format!("effects ({count}/{MAX_EFFECTS})"),
            Owner::Output => format!("output effects ({count})"),
        };
        let Some(effects) = self.effects_mut(owner) else { return };
        // the stack (= drop area) reaches down to the bottom of the visible column
        let frame = egui::Frame::group(ui.style());
        let margins = frame.total_margin().sum().y + 2.0;
        let fill_h = (ui.clip_rect().bottom() - ui.cursor().top() - margins).max(0.0);
        let frame_resp = frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(fill_h);
            ui.label(egui::RichText::new(header).weak());
            for (i, fx) in effects.iter_mut().enumerate() {
                let row = ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let handle = widgets::drag_handle(ui).on_hover_text("drag to reorder, or onto the other effect stack (clip <-> output)");
                    if i == 0 && matches!(owner, Owner::Instance(_)) {
                        first_handle = Some(handle.rect.center());
                    }
                    handle.dnd_set_drag_payload(Drag::Effect(owner, i));
                    widgets::check(ui, &mut fx.active, ACCENT).on_hover_text("effect on/off");
                    let arrow = if fx.open { "▾" } else { "▸" };
                    let name = isf::display_name(&fx.path);
                    if ui.add(egui::Button::new(format!("{arrow} {name}")).frame(false)).clicked() {
                        fx.open = !fx.open;
                    }
                    if let Some(Some((level, tip))) = fx_usage.get(i) {
                        widgets::meter(ui, *level, tip);
                    }
                    if let Some(Some(e)) = shader_errors.get(i) {
                        ui.colored_label(Color32::RED, "⚠").on_hover_text(e);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::btn(ui, "×").clicked() {
                            ev_remove = Some(i);
                        }
                    });
                });
                if let Some(p) = row.response.dnd_release_payload::<Drag>() {
                    match &*p {
                        Drag::Effect(o, from) if *o == owner => ev_move = Some((*from, i)),
                        Drag::Effect(o, from) => ev_transfer = Some((*o, *from, Some(i))),
                        Drag::File(path, FileKind::Effect) => ev_add = Some(path.clone()),
                        _ => {}
                    }
                }
                if row.response.dnd_hover_payload::<Drag>().is_some() {
                    widgets::outline(ui, row.response.rect, 1.5, ACCENT);
                }
                // mix-type, R G B and mix slider are always visible; parameters only when folded out
                ui.indent(("fx", fx.uid), |ui| {
                    widgets::mix_controls(ui, &mut fx.mix, ("fxmix", fx.uid));
                    if fx.open {
                        if let Some(info) = infos.get(&fx.path) {
                            params_ui(ui, info, &mut fx.params, ("fxp", fx.uid));
                        }
                    }
                });
            }
            if effects.is_empty() {
                ui.label(egui::RichText::new("drop effects here").weak());
            }
        });
        let resp = frame_resp.response;
        if let Some(t) = self.selftest.as_mut() {
            match owner {
                Owner::Instance(_) => t.fx_handle = first_handle.or(t.fx_handle),
                Owner::Output => t.out_fx = Some(resp.rect),
            }
        }
        let from_elsewhere = |p: &Drag| match p {
            Drag::File(_, FileKind::Effect) => true,
            Drag::Effect(o, _) => *o != owner,
            _ => false,
        };
        if resp.dnd_hover_payload::<Drag>().is_some_and(|p| from_elsewhere(&p)) {
            widgets::outline(ui, resp.rect, 2.0, ACCENT);
        }
        if let Some(p) = resp.dnd_release_payload::<Drag>() {
            match &*p {
                Drag::File(path, FileKind::Effect) => ev_add = Some(path.clone()),
                Drag::Effect(o, from) if *o != owner => ev_transfer = Some((*o, *from, None)),
                _ => {}
            }
        }
        if let Some((from_owner, from, at)) = ev_transfer {
            self.transfer_effect(from_owner, from, owner, at);
        }
        if let Some(effects) = self.effects_mut(owner) {
            if let Some(i) = ev_remove {
                effects.remove(i);
            } else if let Some((from, to)) = ev_move {
                if from < effects.len() && to < effects.len() && from != to {
                    let fx = effects.remove(from);
                    effects.insert(to, fx);
                }
            }
        }
        if let Some(p) = ev_add {
            self.add_effect(owner, &p);
        }
    }

    /// Move an effect with all its settings between a clip and the output effects.
    fn transfer_effect(&mut self, from: Owner, idx: usize, to: Owner, at: Option<usize>) {
        if from == to {
            return;
        }
        let full = match to {
            Owner::Instance(id) => self.s.find(id).is_some_and(|i| i.effects.len() >= MAX_EFFECTS),
            Owner::Output => false,
        };
        if full {
            self.set_status(format!("max {MAX_EFFECTS} effects per clip"));
            return;
        }
        let Some(fx) = self.effects_mut(from).and_then(|v| (idx < v.len()).then(|| v.remove(idx))) else { return };
        if let Some(target) = self.effects_mut(to) {
            let at = at.unwrap_or(target.len()).min(target.len());
            target.insert(at, fx);
        }
    }

    fn col_mixer(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        ui.label("mixer");
        let mut ev_move: Option<(usize, usize)> = None;
        for li in 0..NUM_LAYERS {
            let color = layer_color(li);
            let selected = self.s.selected_layer == li;
            // the layer's own click sense is registered *below* its buttons (UiBuilder::sense),
            // so the buttons keep their clicks and a click on the background selects the layer
            let resp = ui.scope_builder(egui::UiBuilder::new().id_salt(("layer", li)).sense(Sense::click()), |ui| {
            egui::Frame::group(ui.style())
                .stroke(egui::Stroke::new(if selected { 2.0 } else { 1.0 }, if selected { color } else { color.gamma_multiply(0.3) }))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        ui.label(egui::RichText::new(format!("L{}", li + 1)).strong().color(color));
                        let mut active = self.s.layers[li].active;
                        widgets::check(ui, &mut active, color).on_hover_text("layer on/off");
                        self.s.layers[li].active = active;
                        if widgets::btn(ui, "↑").clicked() && li > 0 {
                            ev_move = Some((li, li - 1));
                        }
                        if widgets::btn(ui, "↓").clicked() && li + 1 < NUM_LAYERS {
                            ev_move = Some((li, li + 1));
                        }
                        let name = self.s.layers[li].instance.and_then(|id| self.s.find(id)).map(|i| i.name.clone()).unwrap_or_else(|| "empty".into());
                        ui.label(truncate(&name, 22));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if self.s.layers[li].instance.is_some() && widgets::btn(ui, "×").on_hover_text("unload").clicked() {
                                self.s.layers[li].instance = None;
                            }
                        });
                    });
                    ui.horizontal(|ui| {
                        let pw = 96.0;
                        let (rect, _) = ui.allocate_exact_size(vec2(pw, pw / self.output_aspect), Sense::hover());
                        let tex = self.s.layers[li].instance.and_then(|id| self.r.as_ref().and_then(|r| r.output_of(id)));
                        let tid = self.tex_id(frame, tex);
                        paint_tex(ui, rect, tid, self.output_aspect);
                        ui.vertical(|ui| {
                            if li == 1 {
                                if let Some(t) = self.selftest.as_mut() {
                                    t.l2_mix = Some(ui.cursor().min);
                                }
                            }
                            widgets::mix_controls(ui, &mut self.s.layers[li].mix, ("layermix", li));
                        });
                    });
                    // clip basics (only for clips with a timeline)
                    if let Some(id) = self.s.layers[li].instance {
                        let status = self.r.as_ref().and_then(|r| r.clip_status.get(&id).copied()).unwrap_or_default();
                        let mut scrub = None;
                        if let Some(inst) = self.s.find_mut(id) {
                            if let SourceKind::Clip { transport, .. } = &mut inst.source {
                                scrub = widgets::timeline(ui, transport, status.position, status.duration);
                                widgets::transport_row(ui, &mut transport.mode, &mut transport.loop_mode, &mut transport.speed, "x");
                            }
                        }
                        if let (Some(n), Some(r)) = (scrub, self.r.as_mut()) {
                            r.pending_scrub.push((id, n));
                        }
                    }
                });
            });
            // click on a layer (background or any of its controls): select it + focus its instance
            let r = resp.response;
            let clicked_inside = ui.input(|i| i.pointer.primary_clicked()) && r.contains_pointer();
            if r.clicked() || clicked_inside {
                self.s.selected_layer = li;
                if let Some(id) = self.s.layers[li].instance {
                    self.s.selected_instance = Some(id);
                    if let Some(slot) = self.s.slot_of(id) {
                        self.s.page = slot / SLOTS_PER_PAGE;
                    }
                }
            }
            // drop a slot onto a layer = load it there
            if let Some(p) = r.dnd_release_payload::<Drag>() {
                if let Drag::Slot(slot) = &*p {
                    if let Some(id) = self.s.slots.get(*slot).and_then(|s| s.as_ref()).map(|i| i.id) {
                        self.s.selected_layer = li;
                        self.s.load_into_selected_layer(id);
                    }
                }
            }
        }
        if let Some((a, b)) = ev_move {
            self.s.layers.swap(a, b);
            if self.s.selected_layer == a {
                self.s.selected_layer = b;
            } else if self.s.selected_layer == b {
                self.s.selected_layer = a;
            }
        }

    }

    /// bpm tapper (below the prepare grid)
    fn section_bpm(&mut self, ui: &mut Ui) {
        ui.label("bpm");
        // one line: TAP, beat dots, bpm slider (takes the rest), ½x 2x
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new(egui::RichText::new("TAP").strong()).min_size(vec2(56.0, 28.0))).clicked() {
                self.tap();
            }
            let beat_in_bar = (self.beat.rem_euclid(4.0)).floor() as usize;
            let frac = self.beat.fract() as f32;
            for i in 0..4 {
                let (rect, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                let on = i == beat_in_bar;
                let c = if on { ACCENT.gamma_multiply(1.0 - frac * 0.7) } else { Color32::from_gray(50) };
                ui.painter().circle_filled(rect.center(), 5.0, c);
            }
            let gap = ui.spacing().item_spacing.x;
            ui.spacing_mut().slider_width = fill_slider_width(ui, 2.0 * (widgets::BTN.x + gap));
            ui.add(egui::Slider::new(&mut self.s.bpm, 20.0..=300.0).step_by(1.0).fixed_decimals(0)).on_hover_text("bpm");
            if widgets::btn(ui, "½x").clicked() {
                self.s.bpm = (self.s.bpm * 0.5).round().max(20.0);
            }
            if widgets::btn(ui, "2x").clicked() {
                self.s.bpm = (self.s.bpm * 2.0).round().min(300.0);
            }
        });
    }

    /// master: blackout, freeze, brightness (below the prepare grid)
    fn section_master(&mut self, ui: &mut Ui) {
        ui.label("master");
        // one line: BLACKOUT, FREEZE, brightness slider (takes the rest)
        ui.horizontal(|ui| {
            let m = &mut self.s.master;
            let bo = if m.blackout { Color32::from_rgb(200, 30, 30) } else { ui.visuals().widgets.inactive.weak_bg_fill };
            if ui.add(egui::Button::new(egui::RichText::new("BLACKOUT").strong()).fill(bo).min_size(vec2(84.0, 28.0))).clicked() {
                m.blackout = !m.blackout;
            }
            let fz = if m.freeze { Color32::from_rgb(40, 120, 220) } else { ui.visuals().widgets.inactive.weak_bg_fill };
            if ui.add(egui::Button::new(egui::RichText::new("FREEZE").strong()).fill(fz).min_size(vec2(70.0, 28.0))).clicked() {
                m.freeze = !m.freeze;
            }
            // during blackout the slider shows 0 %; pulling it up ends the blackout -> fade in from black
            let mut v = if m.blackout { 0.0 } else { m.brightness };
            ui.spacing_mut().slider_width = fill_slider_width(ui, 0.0);
            let r = widgets::percent_slider(ui, &mut v, "").on_hover_text("brightness");
            if let Some(t) = self.selftest.as_mut() {
                // the bar part only (without the value field)
                t.brightness_slider = Some(Rect::from_min_size(r.rect.min, vec2(ui.spacing().slider_width, r.rect.height())));
            }
            let m = &mut self.s.master;
            if r.changed() {
                if m.blackout {
                    if v > 0.0 {
                        m.blackout = false;
                        m.brightness = v;
                    }
                } else {
                    m.brightness = v;
                }
            }
        });
    }

    /// Record box (column 2, below bpm): REC with time, size and meter (the folder is in the REC hover).
    fn section_record(&mut self, ui: &mut Ui) {
        ui.label("record");
        ui.horizontal(|ui| self.rec_button(ui));
    }

    /// REC: records the output (after master) to an H.264 mp4 in the recording folder.
    fn rec_button(&mut self, ui: &mut Ui) {
        let rec = self.r.as_ref().and_then(|r| r.rec.as_ref());
        let recording = rec.is_some();
        let enc = crate::record::encoder();
        let available = matches!(enc, Some(Ok(_)));
        let fill = if recording { Color32::from_rgb(170, 25, 25) } else { ui.visuals().widgets.inactive.weak_bg_fill };
        let label = if recording { "● REC" } else { "REC" };
        let resp = ui.add_enabled(available || recording, egui::Button::new(egui::RichText::new(label).strong()).fill(fill).min_size(vec2(60.0, 28.0)));
        let resp = match enc {
            None => resp.on_disabled_hover_text("testing the hardware encoder ..."),
            Some(Err(e)) => resp.on_disabled_hover_text(e.as_str()),
            Some(Ok((name, desc))) => resp.on_hover_text(format!(
                "record the output (after master) to {}\nH.264 mp4, {desc} ({name}), plays again in videoping",
                self.cfg.record_dir().display()
            )),
        };
        if resp.clicked() {
            self.rec_request = Some(!recording);
        }
        if let Some(rec) = rec {
            if self.rec_size.0.is_none_or(|t| t.elapsed().as_secs() >= 1) {
                self.rec_size = (Some(Instant::now()), std::fs::metadata(&rec.path).map(|m| m.len()).unwrap_or(0));
            }
            let secs = rec.elapsed().as_secs();
            let txt = format!("{:02}:{:02} · {}", secs / 60, secs % 60, meter::mb(self.rec_size.1));
            let tip = format!("{}\n{} frames, {} dropped, encoder {}", rec.path.display(), rec.frames, rec.dropped, rec.encoder);
            ui.label(egui::RichText::new(txt).monospace().size(widgets::BTN_TEXT)).on_hover_text(tip);
            // resource meter: GPU conversion + readback, CPU copy to the encoder, encoder queue
            if self.cfg.meters {
                let budget = self.budget_ms();
                let gpu = self.r.as_ref().and_then(|r| r.meter.ms(Key::Record)).unwrap_or(0.0);
                let queue = rec.queue();
                let behind = queue >= crate::record::Recorder::QUEUE_MAX - 2;
                let level = if behind { 3 } else { meter::gpu_level(gpu, budget).max(meter::gpu_level(rec.cpu_ms, budget)) };
                let mut mtip = format!(
                    "GPU {gpu:.2} ms ({:.0} % of {budget:.1} ms): NV12 conversion + readback\n\
                     CPU {:.2} ms per frame: copy to the encoder (encoding itself runs on the {})\n\
                     encoder queue {queue} / {}",
                    gpu / budget * 100.0,
                    rec.cpu_ms,
                    if rec.encoder.starts_with("nv") { "NVIDIA GPU" } else { "GPU (VA-API)" },
                    crate::record::Recorder::QUEUE_MAX
                );
                if behind {
                    mtip.push_str("\n⚠ encoder does not keep up, frames get dropped");
                }
                widgets::meter(ui, level, &mtip);
            }
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
        }
    }

    fn col_output(&mut self, ui: &mut Ui) {
        self.effect_stack(ui, Owner::Output);
    }

    /// output options (column 2, below record)
    fn section_output_options(&mut self, ui: &mut Ui) {
        // connected screens (refreshed every 3 s: projectors get plugged in during setup)
        if self.screens_read.is_none_or(|t| t.elapsed().as_secs() >= 3) {
            self.screens_read = Some(Instant::now());
            self.screens = connected_screens();
        }
        let screens = self.screens.clone();
        let gpu = self.gpu_name.clone();
        egui::CollapsingHeader::new("output options").default_open(false).show(ui, |ui| {
            let o = &mut self.s.output;
            let res = [(1280, 720), (1920, 1080), (2560, 1440), (3840, 2160)];
            egui::ComboBox::from_label("resolution").selected_text(format!("{}x{}", o.width, o.height)).show_ui(ui, |ui| {
                for (w, h) in res {
                    if ui.selectable_label(o.width == w && o.height == h, format!("{w}x{h}")).clicked() {
                        o.width = w;
                        o.height = h;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("frame rate");
                ui.spacing_mut().item_spacing.x = 2.0;
                for f in [25, 30, 50, 60] {
                    if widgets::choice(ui, o.fps == f, &f.to_string()).on_hover_text("output frame rate (screen + NDI)").clicked() {
                        o.fps = f;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("screen");
                ui.spacing_mut().item_spacing.x = 2.0;
                let n = screens.len().max(o.screen + 1).max(1);
                for i in 0..n {
                    if widgets::choice(ui, o.screen == i, &(i + 1).to_string()).on_hover_text("fullscreen output goes to this screen").clicked() {
                        o.screen = i;
                    }
                }
            });
            if !screens.is_empty() {
                ui.label(egui::RichText::new(format!("connected: {}", screens.join(", "))).weak())
                    .on_hover_text("names from the system; their order can differ from the screen numbers");
            }
            ui.checkbox(&mut o.window_open, "output window");
            ui.add_enabled(o.window_open, egui::Checkbox::new(&mut o.fullscreen, "fullscreen on the chosen screen (F / double-click in output window)"));
            ui.checkbox(&mut o.ndi_enabled, "NDI output");
            ui.horizontal(|ui| {
                ui.label("ndi name");
                ui.add(egui::TextEdit::singleline(&mut o.ndi_name).desired_width(140.0));
            });
            ui.horizontal(|ui| {
                ui.label("NDI library folder");
                ui.add(
                    egui::TextEdit::singleline(&mut o.ndi_lib_dir)
                        .hint_text("empty = default places")
                        .desired_width(ui.available_width()),
                )
                .on_hover_text("folder that contains libndi.so.6, e.g. from the NDI SDK for Linux: lib/x86_64-linux-gnu");
            });
            if let Some(e) = self.r.as_ref().and_then(|r| r.ndi_error.as_ref()) {
                ui.colored_label(Color32::RED, e);
            } else if let Some(n) = self.r.as_ref().and_then(|r| r.ndi.as_ref()) {
                ui.label(egui::RichText::new("NDI sending").color(Color32::GREEN)).on_hover_text(&n.lib_path);
            }
            ui.separator();
            ui.label(egui::RichText::new(format!("gpu: {gpu}")).weak());

            if nvidia_available() {
                let o = &mut self.s.output;
                ui.checkbox(&mut o.prefer_nvidia, "render on NVIDIA GPU (applies on restart)")
                    .on_hover_text("restarts itself with PRIME render offload; one-off: start with --nvidia");
            }
        });
    }

    /// General config: settings of the program, not of the session (own file, untouched by
    /// save / load / new).
    fn section_general_config(&mut self, ui: &mut Ui) {
        egui::CollapsingHeader::new("general config").default_open(false).show(ui, |ui| {
            let before = self.cfg.clone();
            ui.checkbox(&mut self.cfg.meters, "resource meters")
                .on_hover_text("GPU time, VRAM and clip decoding per slot / effect, totals in the top bar");
            ui.label("recording folder");
            ui.horizontal(|ui| {
                let hint = self.cfg.record_dir().display().to_string();
                ui.add(egui::TextEdit::singleline(&mut self.cfg.record_dir).hint_text(hint).desired_width(ui.available_width()));
            });
            let dir = self.cfg.record_dir();
            if !self.s.finder_roots.contains(&dir) && widgets::btn(ui, "add to finder").on_hover_text("show the recordings in the finder").clicked() {
                let _ = std::fs::create_dir_all(&dir);
                self.s.finder_roots.push(dir);
                self.rescan_finder();
            }
            if self.cfg != before && self.selftest.is_none() {
                if let Err(e) = save_config(&self.cfg) {
                    self.set_status(e);
                }
            }
        });
    }

    /// Prepare preview (col 0) and main output preview (col 1) share one width:
    /// the narrower of the two columns, max 640.
    fn preview_width(&mut self, ui: &Ui, col: usize) -> f32 {
        self.preview_avail[col] = ui.available_width();
        self.preview_avail[0].min(self.preview_avail[1]).clamp(80.0, 640.0)
    }

    /// main output preview (pinned at the top of column 4)
    fn output_preview(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let w = self.preview_width(ui, 1);
        let (rect, _) = ui.allocate_exact_size(vec2(w, w / self.output_aspect), Sense::hover());
        let tex = self.r.as_ref().map(|r| r.out.tex);
        let tid = self.tex_id(frame, tex);
        paint_tex(ui, rect, tid, self.output_aspect);
        if self.s.master.blackout || self.s.master.freeze {
            let t = [(self.s.master.blackout, "BLACKOUT"), (self.s.master.freeze, "FREEZE")]
                .iter()
                .filter(|x| x.0)
                .map(|x| x.1)
                .collect::<Vec<_>>()
                .join(" + ");
            ui.painter().text(rect.left_top() + vec2(4.0, 4.0), egui::Align2::LEFT_TOP, t, egui::FontId::proportional(11.0), Color32::RED);
        }
    }

    fn output_window(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if !self.s.output.window_open {
            return;
        }
        let tex = self.r.as_ref().map(|r| r.out.tex);
        let tid = self.tex_id(frame, tex);
        let aspect = self.output_aspect;
        let vid = egui::ViewportId::from_hash_of("videoping-output");
        let fullscreen = self.s.output.fullscreen;
        let mut close = false;
        let mut toggle_fs = false;
        let screen = self.s.output.screen;
        let mut builder = egui::ViewportBuilder::default()
            .with_title("videoping output")
            .with_icon(crate::app_icon())
            .with_inner_size([960.0, 540.0]);
        builder = if fullscreen { builder.with_monitor(screen) } else { builder.with_fullscreen(false) };
        ctx.show_viewport_immediate(
            vid,
            builder,
            |ui, _class| {
                let rect = ui.max_rect();
                let full = ui.ctx().content_rect();
                let rect = if full.is_positive() { full } else { rect };
                paint_tex(ui, rect, tid, aspect);
                ui.input(|i| {
                    if i.viewport().close_requested() {
                        close = true;
                    }
                    if i.key_pressed(egui::Key::F) {
                        toggle_fs = true;
                    }
                    if i.key_pressed(egui::Key::Escape) && fullscreen {
                        toggle_fs = true;
                    }
                });
                if ui.input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary)) {
                    toggle_fs = true;
                }
            },
        );
        if close {
            self.s.output.window_open = false;
            self.s.output.fullscreen = false;
            self.applied_output = None;
            return;
        }
        if toggle_fs {
            self.s.output.fullscreen = !self.s.output.fullscreen;
        }
        // fullscreen on the chosen screen / back to a window
        let want = (self.s.output.fullscreen, self.s.output.screen);
        if self.applied_output != Some(want) {
            let cmd = if want.0 { egui::ViewportCommand::SetMonitor(want.1) } else { egui::ViewportCommand::Fullscreen(false) };
            ctx.send_viewport_cmd_to(vid, cmd);
            self.applied_output = Some(want);
        }
    }
}

/// Parameter ui: range slider + animation row per parameter.
fn params_ui(ui: &mut Ui, info: &IsfInfo, params: &mut [Param], salt: impl std::hash::Hash + Copy + std::fmt::Debug) {
    for inp in &info.inputs {
        let Some(p) = params.iter_mut().find(|p| p.name == inp.name) else { continue };
        let row = ui.push_id((salt, &inp.name), |ui| match (&inp.kind, &mut p.value) {
            (InputKind::Float { min, max, .. }, ParamValue::Anim(a)) => anim_ui(ui, &inp.label, &mut a[0], *min, *max, false),
            (InputKind::Long { min, max, labels, .. }, ParamValue::Anim(a)) => {
                if labels.is_empty() {
                    anim_ui(ui, &inp.label, &mut a[0], *min, *max, true)
                } else {
                    // same layout as the sliders: name column, then the control
                    ui.horizontal_top(|ui| {
                        param_label(ui, &inp.label);
                        let cur = a[0].value.round() as i32;
                        let txt = labels.iter().find(|l| l.0 == cur).map(|l| l.1.clone()).unwrap_or(cur.to_string());
                        egui::ComboBox::from_id_salt("long").selected_text(txt).show_ui(ui, |ui| {
                            for (v, l) in labels {
                                if ui.selectable_label(*v == cur, l).clicked() {
                                    a[0].value = *v as f32;
                                }
                            }
                        });
                    });
                }
            }
            (InputKind::Point2D { min, max, .. }, ParamValue::Anim(a)) if a.len() == 2 => {
                anim_ui(ui, &format!("{} x", inp.label), &mut a[0], min[0], max[0], false);
                anim_ui(ui, &format!("{} y", inp.label), &mut a[1], min[1], max[1], false);
            }
            (InputKind::Bool { .. }, ParamValue::Bool(b)) => {
                // same layout as the sliders: name column, then the control
                ui.horizontal_top(|ui| {
                    param_label(ui, &inp.label);
                    widgets::check(ui, b, ACCENT);
                });
            }
            (InputKind::Event, ParamValue::Bool(b)) => {
                if ui.button(&inp.label).clicked() {
                    *b = true;
                }
            }
            (InputKind::Color { .. }, ParamValue::Color(c)) => {
                // same layout as the sliders: name column, then the control
                ui.horizontal_top(|ui| {
                    param_label(ui, &inp.label);
                    ui.color_edit_button_rgba_unmultiplied(c);
                });
            }
            _ => {}
        });
        // right click on a parameter row (name or control): back to the default of the isf
        // header, as freshly loaded (value, range limits, animation off)
        if std::env::var_os("VIDEOPING_SELFTEST_RESET").is_some() {
            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::unique(("selftest_param_row", &inp.name)), row.response.rect));
        }
        // (rect_contains_pointer: not when a popup or dragged item lies on top of the row)
        // on press, not on click: a click is not counted when the mouse moves a little or the
        // button is held a bit longer
        let right_click = ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Secondary))
            && ui.rect_contains_pointer(row.response.rect);
        if right_click && let Some(d) = isf::default_params(info).into_iter().find(|d| d.name == inp.name) {
            p.value = d.value;
        }
    }
}

/// Width of the parameter label column.
const PARAM_LABEL_W: f32 = 120.0;

fn anim_ui(ui: &mut Ui, label: &str, a: &mut AnimParam, min: f32, max: f32, integer: bool) {
    // the label column spans both rows (slider + play buttons): a name that wraps onto a second
    // line sits next to the play buttons instead of pushing them down
    ui.horizontal_top(|ui| {
        param_label(ui, label);
        ui.vertical(|ui| {
            widgets::range_slider(ui, a, min, max, integer);
            widgets::transport_row(ui, &mut a.mode, &mut a.loop_mode, &mut a.speed, "/s");
        });
    });
}

/// Parameter label: left aligned, normal text size, wraps onto a second line if too long.
fn param_label(ui: &mut Ui, label: &str) {
    ui.allocate_ui_with_layout(vec2(PARAM_LABEL_W, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_width(PARAM_LABEL_W);
        ui.add(egui::Label::new(egui::RichText::new(label).size(widgets::BTN_TEXT)).wrap()).on_hover_text(label);
    });
}

/// Type badge of a prepare slot.
fn source_badge(src: &SourceKind) -> &'static str {
    match src {
        SourceKind::Clip { .. } => "▶",
        SourceKind::Generator { .. } => "∿",
        SourceKind::Ndi { .. } => "NDI",
    }
}

/// Connected screens (DRM connectors with status "connected"), e.g. ["eDP-1", "HDMI-A-1"].
fn connected_screens() -> Vec<String> {
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir("/sys/class/drm") {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            // card1-HDMI-A-1 -> HDMI-A-1 (skip card1, renderD128, ...)
            let Some((card, conn)) = name.split_once('-') else { continue };
            if !card.starts_with("card") {
                continue;
            }
            if std::fs::read_to_string(e.path().join("status")).is_ok_and(|s| s.trim() == "connected") {
                out.push(conn.to_string());
            }
        }
    }
    out.sort();
    out
}

/// Is there an NVIDIA GL/EGL driver that PRIME render offload can use?
pub fn nvidia_available() -> bool {
    std::path::Path::new(crate::NVIDIA_EGL_VENDOR).exists()
}

/// Width for a text field followed by a button of width `button_w` on the same line, so the line
/// never gets wider than the column (a too-wide line makes a resizable panel grow by itself).
/// Slider bar width so that bar + value field + `after` (widgets right of it) fill the rest of the line.
fn fill_slider_width(ui: &Ui, after: f32) -> f32 {
    let value_field = ui.spacing().interact_size.x + 8.0;
    (ui.available_width() - value_field - ui.spacing().item_spacing.x - after - 2.0).max(40.0)
}

fn field_width(ui: &Ui, button_w: f32) -> f32 {
    (ui.available_width() - button_w - ui.spacing().item_spacing.x - ui.spacing().button_padding.x * 2.0 - 2.0).max(40.0)
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n.saturating_sub(1)).collect::<String>())
    }
}

/// egui's built-in fonts lack symbols like ▶ ◀ ↑ ↓ ≡: add system fonts as fallback.
fn load_symbol_fonts(ctx: &egui::Context) {
    let candidates = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf",
        "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
    ];
    let mut fonts = egui::FontDefinitions::default();
    let mut added = false;
    for (i, path) in candidates.iter().enumerate() {
        if let Ok(bytes) = std::fs::read(path) {
            let name = format!("symbols{i}");
            fonts.font_data.insert(name.clone(), std::sync::Arc::new(egui::FontData::from_owned(bytes)));
            for fam in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts.families.entry(fam).or_default().push(name.clone());
            }
            added = true;
        }
    }
    if added {
        ctx.set_fonts(fonts);
    } else {
        eprintln!("no DejaVu / Noto Symbols font found: some ui symbols may show as boxes");
    }
}

fn style(ctx: &egui::Context) {
    load_symbol_fonts(ctx);
    ctx.set_visuals(egui::Visuals::dark());
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = vec2(6.0, 4.0);
        s.spacing.button_padding = vec2(5.0, 2.0);
        s.visuals.selection.bg_fill = ACCENT;
        s.visuals.panel_fill = Color32::from_gray(18);
        s.visuals.extreme_bg_color = Color32::from_gray(8);
        // less rounded corners on all buttons / fields
        let w = &mut s.visuals.widgets;
        for st in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
            st.corner_radius = egui::CornerRadius::same(widgets::BTN_RADIUS);
        }
    });
}

impl App {
    /// The tab "live": finder | prepare grid | prepare detail | mixer + output.
    fn live_tab(&mut self, ui: &mut Ui, frame: &mut eframe::Frame, ctx: &egui::Context) {
        egui::Panel::left("col1").resizable(true).default_size(260.0).size_range(160.0..=600.0).show(ui, |ui| {
            self.col_finder(ui, frame);
        });
        egui::Panel::left("grid_col").resizable(true).default_size(420.0).size_range(240.0..=900.0).show(ui, |ui| {
            self.col_grid(ui, frame);
        });
        // prepare detail: ~30% of the window by default (10% narrower than the old layout)
        let detail_w = (ctx.content_rect().width() * 0.30).clamp(320.0, 900.0);
        egui::Panel::left("detail_col").resizable(true).default_size(detail_w).size_range(260.0..=1000.0).show(ui, |ui| {
            // always claim the full width: an empty detail must not shrink the column
            ui.set_min_width(ui.available_width());
            // preview stays on top, the rest scrolls below it
            if let Some(id) = self.prepare_detail_header(ui, frame) {
                ui.add_space(4.0);
                egui::ScrollArea::vertical().id_salt("detail").auto_shrink([false, false]).show(ui, |ui| {
                    self.prepare_detail_body(ui, id);
                });
            }
        });
        // mixer + output merged in the last column (takes the remaining width)
        egui::CentralPanel::default().show(ui, |ui| {
            // main output preview stays on top, the rest scrolls below it
            self.output_preview(ui, frame);
            ui.separator();
            egui::ScrollArea::vertical().id_salt("mixout").auto_shrink([false, false]).show(ui, |ui| {
                self.col_mixer(ui, frame);
                ui.add_space(6.0);
                self.col_output(ui);
            });
        });
    }
}

#[path = "midi_tab.rs"]
mod midi_tab;

/// Top-level tabs (mapping, shader editor and sequencer later).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tab {
    Live,
    Midi,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let Some(gl) = frame.gl().cloned() else { return };
        self.update_preview(&gl);
        self.render(&gl);

        egui::Panel::top("top").show(ui, |ui| self.top_bar(ui, &gl));
        // the output keeps rendering in every tab (render() above)
        match self.tab {
            Tab::Live => self.live_tab(ui, frame, &ctx),
            Tab::Midi => self.midi_tab(ui),
        }

        self.confirm_new_dialog(&ctx, &gl);
        self.output_window(&ctx, frame);
        if self.selftest.is_some() {
            self.selftest_step(&ctx, &gl);
        } else {
            self.autosave();
        }
        ctx.request_repaint();
    }

    /// the self-test must not touch the stored ui layout
    /// self-test: simulated mouse drag of the in-limiter "[" of the prepare-detail timeline
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        // the MIDI self-test plays without the simulated mouse (its clicks select layers too)
        if std::env::var_os("VIDEOPING_SELFTEST_MIDI").is_some() {
            return;
        }
        let in_pt = self.s.selected_instance.and_then(|id| self.s.find(id)).and_then(|i| match &i.source {
            SourceKind::Clip { transport, .. } => Some(transport.in_pt),
            _ => None,
        });
        let pending = self.r.as_ref().map(|r| r.pending_scrub.len()).unwrap_or(0);
        if std::env::var("VIDEOPING_SELFTEST_FX").is_ok() {
            self.selftest_fx_transfer(raw);
            return;
        }
        if std::env::var("VIDEOPING_SELFTEST_RESET").is_ok() {
            self.selftest_param_reset(ctx, raw);
            return;
        }
        let _ = ctx;
        let l2 = (self.s.layers[1].mix.mix_type, self.s.selected_layer);
        let selected = self.s.selected_instance;
        let Some(t) = self.selftest.as_mut() else { return };
        // after the screenshot: click an empty slot -> nothing selected
        // after "new": blackout on, click the brightness slider at ~40 % -> blackout off, fade in
        if t.drag_step >= 27 {
            if t.drag_step > 31 {
                return;
            }
            if let Some(r) = t.brightness_slider {
                let p = egui::pos2(r.left() + r.width() * 0.4, r.center().y);
                let btn = |pressed| egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
                match t.drag_step {
                    27 => self.s.master.blackout = true,
                    28 => raw.events.push(egui::Event::PointerMoved(p)),
                    29 => raw.events.push(btn(true)),
                    30 => raw.events.push(btn(false)),
                    _ => eprintln!(
                        "selftest fade in: blackout = {} (expected false), brightness = {:.0} % (expected ~40)",
                        self.s.master.blackout,
                        self.s.master.brightness * 100.0
                    ),
                }
            }
            if let Some(t) = self.selftest.as_mut() {
                t.drag_step += 1;
            }
            return;
        }
        if t.drag_step >= 21 {
            if t.start.elapsed().as_secs_f32() < 5.6 || t.drag_step > 26 {
                return;
            }
            let btn = |p: egui::Pos2, pressed| egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
            match (t.drag_step, t.clear_btn) {
                (21, _) => self.confirm_new = true,
                (22, _) => {
                    let dir = t.dir.clone();
                    ctx.request_screenshot(move |img| {
                        let bytes: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
                        let _ = std::fs::write(dir.join(format!("ui-new_{}x{}.rgba", img.size[0], img.size[1])), bytes);
                    });
                }
                (23, Some(p)) => raw.events.push(egui::Event::PointerMoved(p)),
                (24, Some(p)) => raw.events.push(btn(p, true)),
                (25, Some(p)) => raw.events.push(btn(p, false)),
                (26, _) => eprintln!(
                    "selftest new: instances = {}, layers loaded = {}, dialog open = {}, finder folders kept = {}, fps kept = {}",
                    self.s.slots.iter().flatten().count(),
                    self.s.layers.iter().filter(|l| l.instance.is_some()).count(),
                    self.confirm_new,
                    !self.s.finder_roots.is_empty(),
                    self.s.output.fps
                ),
                _ => {}
            }
            if let Some(t) = self.selftest.as_mut() {
                t.drag_step += 1;
            }
            return;
        }
        if t.drag_step >= 17 {
            if t.start.elapsed().as_secs_f32() < 5.0 || t.drag_step > 20 {
                return;
            }
            if let Some(p) = t.empty_slot {
                let btn = |pressed| egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
                match t.drag_step {
                    17 => raw.events.push(egui::Event::PointerMoved(p)),
                    18 => raw.events.push(btn(true)),
                    19 => raw.events.push(btn(false)),
                    _ => {
                        eprintln!("selftest empty slot click: selected instance = {selected:?} (expected None)");
                        // second screenshot: layout with an empty prepare detail
                        let dir = t.dir.clone();
                        ctx.request_screenshot(move |img| {
                            let bytes: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
                            let _ = std::fs::write(dir.join(format!("ui-empty_{}x{}.rgba", img.size[0], img.size[1])), bytes);
                        });
                    }
                }
            }
            t.drag_step += 1;
            return;
        }
        let Some(tl) = t.timeline else { return };
        if t.start.elapsed().as_secs_f32() < 2.0 {
            return;
        }
        // after the drag: click "Lk" (2nd mix button) of mixer layer L2
        if t.drag_step >= 13 {
            if let Some(m) = t.l2_mix {
                let p = m + vec2(30.0 + 2.0 + 15.0, 9.0);
                let btn = |pressed| egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                };
                match t.drag_step {
                    13 => raw.events.push(egui::Event::PointerMoved(p)),
                    14 => raw.events.push(btn(true)),
                    15 => raw.events.push(btn(false)),
                    _ => eprintln!("selftest click: L2 mix-type = {:?} (expected LumaKey), selected layer = L{} (expected L2)", l2.0, l2.1 + 1),
                }
            }
            t.drag_step += 1;
            return;
        }
        let in_pt = in_pt.unwrap_or(0.0);
        let track = tl.shrink2(vec2(2.0, 0.0));
        // press 6 px inside the "[" handle, then move 80 px right in 10 steps, release
        let x0 = track.left() + in_pt * track.width() + 6.0;
        let y = tl.center().y;
        let ev = &mut raw.events;
        let pos = |x: f32| egui::pos2(x, y);
        match t.drag_step {
            0 => {
                t.in_before = Some(in_pt);
                ev.push(egui::Event::PointerMoved(pos(x0)));
            }
            1 => ev.push(egui::Event::PointerButton {
                pos: pos(x0),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            }),
            2..=11 => {
                let start = t.in_before.unwrap_or(0.0);
                let x = track.left() + start * track.width() + 6.0 + 8.0 * (t.drag_step - 1) as f32;
                ev.push(egui::Event::PointerMoved(pos(x)));
            }
            _ => {
                let start = t.in_before.unwrap_or(0.0);
                let x = track.left() + start * track.width() + 6.0 + 80.0;
                ev.push(egui::Event::PointerButton {
                    pos: pos(x),
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                });
                eprintln!(
                    "selftest drag: in_pt {:.3} -> {:.3} (expected +{:.3}), scrubs queued: {pending}",
                    start,
                    in_pt,
                    80.0 / track.width()
                );
            }
        }
        t.drag_step += 1;
    }

    fn persist_egui_memory(&self) -> bool {
        self.selftest.is_none()
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if self.selftest.is_none() {
            let _ = save_session(&self.s, &self.autosave_path);
        }
        // midi profile changes of the last half second
        if let Some(m) = self.midi.as_mut() {
            m.flush();
        }
        if let (Some(r), Some(gl)) = (self.r.as_mut(), gl) {
            r.ndi = None;
            // a running recording is finished before quitting (otherwise the end is cut off)
            if let Some(h) = r.stop_recording(gl) {
                let _ = h.join();
            }
        }
    }
}

/// Self-test (VIDEOPING_SELFTEST=<dir>): builds a session, saves a ui screenshot and an
/// output frame as raw RGBA into <dir>, then quits. Never touches the autosave.
/// Virtual "Bhoreal" for the MIDI self-test: an output videoping reads the pads from, an input
/// that receives the LED feedback.
struct MidiSim {
    out: midir::MidiOutputConnection,
    _leds_in: midir::MidiInputConnection<()>,
    leds: std::sync::Arc<std::sync::Mutex<HashMap<u8, u8>>>,
    step: u32,
}

struct SelfTest {
    dir: PathBuf,
    start: Instant,
    shot: bool,
    dumped: bool,
    /// rect of the prepare-detail timeline (for the simulated limiter drag)
    timeline: Option<Rect>,
    drag_step: u32,
    in_before: Option<f32>,
    /// top-left of L2's mix buttons (for the simulated click on "Lk")
    l2_mix: Option<egui::Pos2>,
    closing: bool,
    /// center of an empty prepare slot (simulated click -> deselect)
    empty_slot: Option<egui::Pos2>,
    /// center of "clear everything" in the "new" dialog
    clear_btn: Option<egui::Pos2>,
    /// rect of the brightness slider (blackout -> fade in test)
    brightness_slider: Option<Rect>,
    /// recording test: 0 = not started, 1 = recording, 2 = stopped, 3 = finder checked
    rec_step: u8,
    rec_file: Option<PathBuf>,
    rec_stopped_at: Option<f32>,
    control_done: bool,
    midi_sim: Option<MidiSim>,
    prep0: u64,
    /// "≡" handle of the first clip effect / output effect stack (effect transfer test)
    fx_handle: Option<egui::Pos2>,
    out_fx: Option<Rect>,
    /// (video frames, ui frames) at t = 2 s, ui frame counter
    at2s: Option<(u64, u64)>,
    ui_frames: u64,
}

fn selftest_session(t: &SelfTest) -> Session {
    let mut s = Session::default();
    let mut rng = Rng::new();
    let root = std::env::current_dir().unwrap_or_default();
    let gen_path = root.join("isf-generator/0-id-display.fs");
    let fx_path = std::env::var("VIDEOPING_SELFTEST_EFFECT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| root.join("isf-effect/gameboy-camera.fs"));
    let clip = std::env::var("VIDEOPING_SELFTEST_CLIP").map(PathBuf::from).unwrap_or_else(|_| t.dir.join("testclip.mp4"));
    let gen_info = isf::parse_info(&gen_path).expect("generator").0;
    let fx_info = isf::parse_info(&fx_path).expect("effect").0;
    let gen_id = s.new_id();
    s.slots[0] = Some(Instance {
        id: gen_id,
        name: "0-id-display".into(),
        source: SourceKind::Generator { path: gen_path, params: isf::default_params(&gen_info) },
        effects: vec![],
        seed: rng.next_f32(),
    });
    let clip_id = s.new_id();
    let fx_uid = s.new_id();
    let fx2_uid = s.new_id();
    s.slots[8] = Some(Instance {
        id: clip_id,
        name: "testclip".into(),
        source: SourceKind::Clip { path: clip, transport: ClipTransport { in_pt: 0.2, out_pt: 0.85, ..Default::default() } },
        effects: vec![EffectInst {
            uid: fx_uid,
            path: fx_path,
            active: true,
            mix: Mix::default(),
            params: isf::default_params(&fx_info),
            seed: rng.next_f32(),
            open: true,
        }, EffectInst {
            // second effect, folded in (shows mix controls only)
            uid: fx2_uid,
            path: root.join("isf-effect/gameboy-camera.fs"),
            active: false,
            mix: Mix { mix_type: MixType::Add, amount: 0.5, rgb: [true, false, true] },
            params: isf::default_params(&fx_info),
            seed: rng.next_f32(),
            open: false,
        }],
        seed: rng.next_f32(),
    });
    s.layers[0].instance = Some(gen_id);
    s.layers[0].mix.mix_type = match std::env::var("VIDEOPING_SELFTEST_L1MIX").as_deref() {
        Ok("ilk") => MixType::InvLumaKey,
        Ok("a") => MixType::Alpha,
        Ok("ad") => MixType::Add,
        _ => MixType::LumaKey,
    };
    s.layers[1].instance = Some(clip_id);
    // VIDEOPING_SELFTEST_L1=clip: clip in layer 1 (over the generator), VIDEOPING_SELFTEST_L1AMOUNT:
    // its mix in % (luma fade check: the clip has several brightness levels)
    if std::env::var("VIDEOPING_SELFTEST_L1").as_deref() == Ok("clip") {
        s.layers[0].instance = Some(clip_id);
        s.layers[1].instance = Some(gen_id);
    }
    if let Some(a) = std::env::var("VIDEOPING_SELFTEST_L1AMOUNT").ok().and_then(|v| v.parse::<f32>().ok()) {
        s.layers[0].mix.amount = a / 100.0;
    }
    s.selected_instance = Some(clip_id);
    // finder: only the repo shaders and the self-test folder (no private files in screenshots)
    s.finder_roots = vec![root.join("isf-generator"), root.join("isf-effect"), t.dir.clone()];
    // VIDEOPING_SELFTEST_PREPARE=<generator .fs>: a (heavy) generator in slot 3, selected but not
    // live -> prepare priority test
    if let Ok(p) = std::env::var("VIDEOPING_SELFTEST_PREPARE") {
        let path = PathBuf::from(p);
        if let Ok((info, _)) = isf::parse_info(&path) {
            let id = s.new_id();
            s.slots[2] = Some(Instance {
                id,
                name: isf::display_name(&path),
                source: SourceKind::Generator { params: isf::default_params(&info), path },
                effects: vec![],
                seed: rng.next_f32(),
            });
            s.selected_instance = Some(id);
        }
    }
    // MIDI test: a copy of the generator that is not live (to load with a pad)
    if std::env::var_os("VIDEOPING_SELFTEST_MIDI").is_some() {
        s.duplicate_slot(0, &mut rng);
    }
    s.output.ndi_enabled = std::env::var("VIDEOPING_SELFTEST_NDI").is_ok();
    if let Some(fps) = std::env::var("VIDEOPING_SELFTEST_FPS").ok().and_then(|f| f.parse().ok()) {
        s.output.fps = fps;
    }
    s
}

impl App {
    /// VIDEOPING_SELFTEST_FX: drag the first clip effect by its handle onto the output effects.
    fn selftest_fx_transfer(&mut self, raw: &mut egui::RawInput) {
        let counts = |s: &Session| {
            let clip = s.selected_instance.and_then(|id| s.find(id)).map(|i| i.effects.len()).unwrap_or(0);
            (clip, s.output_effects.len())
        };
        let (clip_n, out_n) = counts(&self.s);
        let Some(t) = self.selftest.as_mut() else { return };
        let (Some(h), Some(out)) = (t.fx_handle, t.out_fx) else { return };
        if t.start.elapsed().as_secs_f32() < 2.0 || t.drag_step > 16 {
            return;
        }
        let target = out.center();
        let pos = |k: f32| h + (target - h) * k;
        let btn = |p: egui::Pos2, pressed| egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
        match t.drag_step {
            0 => {
                eprintln!("selftest fx transfer: before clip effects = {clip_n}, output effects = {out_n}");
                raw.events.push(egui::Event::PointerMoved(h));
            }
            1 => raw.events.push(btn(h, true)),
            2..=13 => raw.events.push(egui::Event::PointerMoved(pos((t.drag_step - 1) as f32 / 12.0))),
            14 => raw.events.push(btn(target, false)),
            16 => eprintln!("selftest fx transfer: after  clip effects = {clip_n}, output effects = {out_n} (expected one moved)"),
            _ => {}
        }
        t.drag_step += 1;
    }

    /// VIDEOPING_SELFTEST_RESET: change the first parameter of the first clip effect, then
    /// right click its row -> back to the default.
    fn selftest_param_reset(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let Some(t) = self.selftest.as_mut() else { return };
        if t.start.elapsed().as_secs_f32() < 2.0 || t.drag_step > 17 {
            return;
        }
        let Some(fx) = self.s.selected_instance.and_then(|id| self.s.find_mut(id)).and_then(|i| i.effects.first_mut()) else { return };
        let Some(p) = fx.params.iter_mut().find(|p| matches!(p.value, ParamValue::Anim(_))) else { return };
        let ParamValue::Anim(a) = &mut p.value else { return };
        let rect: Option<Rect> = ctx.data(|d| d.get_temp(egui::Id::unique(("selftest_param_row", &p.name))));
        let btn = |pos, pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Secondary, pressed, modifiers: Default::default() };
        let slider_pos = |r: Rect| egui::pos2(r.left() + PARAM_LABEL_W + (r.width() - PARAM_LABEL_W) * 0.3, r.top() + widgets::BAR_H / 2.0);
        match (t.drag_step, rect) {
            (0, _) => {
                a[0].lo = (a[0].lo + a[0].hi) / 2.0;
                a[0].value = a[0].hi;
                eprintln!("selftest param reset: {} set to value {} range {}..{}", p.name, a[0].value, a[0].lo, a[0].hi);
            }
            // on the slider bar itself (top of the row, right of the label column); a real
            // press: held for some frames, the mouse moves a little
            (2, Some(r)) => raw.events.push(egui::Event::PointerMoved(slider_pos(r))),
            (3, Some(r)) => raw.events.push(btn(slider_pos(r), true)),
            (4..=14, Some(r)) => raw.events.push(egui::Event::PointerMoved(slider_pos(r) + egui::vec2((t.drag_step % 3) as f32, 0.0))),
            (15, Some(r)) => raw.events.push(btn(slider_pos(r), false)),
            (3..=16, _) => eprintln!("selftest param reset: step {} value {}", t.drag_step, a[0].value),
            (17, _) => eprintln!(
                "selftest param reset: {} after right click = value {} range {}..{} (expected the default)",
                p.name, a[0].value, a[0].lo, a[0].hi
            ),
            _ => {}
        }
        t.drag_step += 1;
    }

    fn selftest_midi(&mut self) {
        use midir::os::unix::{VirtualInput, VirtualOutput};
        let Some(t) = self.selftest.as_mut() else { return };
        if t.midi_sim.is_none() {
            if t.start.elapsed().as_secs_f32() < 0.5 {
                return;
            }
            let leds: std::sync::Arc<std::sync::Mutex<HashMap<u8, u8>>> = Default::default();
            let l2 = leds.clone();
            let out = midir::MidiOutput::new("bhoreal-selftest-pads").and_then(|o| o.create_virtual("Bhoreal selftest").map_err(|e| e.kind()).map_err(|_| midir::InitError));
            let inp = midir::MidiInput::new("bhoreal-selftest-leds").ok().and_then(|i| {
                i.create_virtual(
                    "Bhoreal selftest",
                    move |_, b, _| {
                        if b.len() == 3 && b[0] & 0xF0 == 0x90 {
                            l2.lock().unwrap().insert(b[1], b[2]);
                        }
                    },
                    (),
                )
                .ok()
            });
            match (out, inp) {
                (Ok(out), Some(inp)) => t.midi_sim = Some(MidiSim { out, _leds_in: inp, leds, step: 0 }),
                _ => {
                    eprintln!("selftest midi: could not create virtual ports (ALSA sequencer?)");
                    std::process::exit(1);
                }
            }
            return;
        }
        let connected = self.midi.as_ref().is_some_and(|m| m.devices().iter().any(|d| d.0.contains("selftest")));
        // the copy of the generator: the generator in a slot other than slot 1
        let dup = self.s.slots.iter().enumerate().skip(1).find(|(_, x)| x.as_ref().is_some_and(|i| matches!(i.source, SourceKind::Generator { .. }))).map(|(i, _)| i);
        let Some(t) = self.selftest.as_mut() else { return };
        let Some(sim) = t.midi_sim.as_mut() else { return };
        if sim.step == 0 && !connected {
            return; // wait for the port scan (every 2 s)
        }
        let dup_note = 8 + dup.unwrap_or(0) as u8;
        // pad presses (press / release on consecutive frames, like a finger)
        let seq: [(u32, u8, u8); 8] = [(1, 2, 64), (2, 2, 0), (3, dup_note, 64), (4, dup_note, 0), (5, 6, 64), (6, 6, 0), (7, 48, 64), (8, 48, 0)];
        if let Some(&(_, note, vel)) = seq.iter().find(|x| x.0 == sim.step) {
            let _ = sim.out.send(&[0x90, note, vel]);
        }
        if sim.step == 1 {
            eprintln!("selftest midi: connected: {:?}", self.midi.as_ref().map(|m| m.devices()));
        }
        if sim.step == 40 {
            let leds = sim.leds.lock().unwrap().clone();
            let led = |n: u8| leds.get(&n).copied();
            eprintln!(
                "selftest midi: selected layer = L{} (expected L3), L3 = slot {:?} (expected the copy in slot {}), blackout = {} (expected true), clip fx 1 on = {} (expected false)",
                self.s.selected_layer + 1,
                self.s.layers[2].instance.and_then(|id| self.s.slot_of(id)).map(|x| x + 1),
                dup.map(|x| x + 1).unwrap_or(0),
                self.s.master.blackout,
                self.s.slots[8].as_ref().is_some_and(|i| i.effects.first().is_some_and(|e| e.active)),
            );
            eprintln!(
                "selftest midi: leds: L1 select {:?} (0), L3 select {:?} (orange 10), blackout {:?} (red 1), freeze {:?} (0), slot 1 {:?} (blue 72), copy {:?} (orange 10), clip slot 9 {:?} (green 44), empty slot 32 {:?} (0), L2 fx1 {:?} (plum 108), L2 fx3 {:?} (0), L1 fx1 {:?} (0); {} leds received",
                led(0), led(2), led(6), led(7), led(8), led(dup_note), led(16), led(39), led(48), led(50), led(40), leds.len()
            );
        }
        // learn: cc 20 of the virtual device -> /master/freeze, then use it
        let port = self.midi.as_ref().and_then(|m| m.devices().into_iter().find(|d| d.0.contains("selftest")).map(|d| d.0));
        match sim.step {
            42 => {
                if let (Some(m), Some(port)) = (self.midi.as_mut(), port.clone()) {
                    m.learn = Some(crate::midi::Learn { port: port.clone(), addr: "/master/freeze".into(), kind: crate::control::Kind::Toggle });
                    self.midi_sel_port = Some(port);
                    self.midi_sel_addr = Some("/master/freeze".into());
                }
            }
            45 => {
                let _ = sim.out.send(&[0xB0, 20, 127]);
            }
            46 | 56 => {
                let _ = sim.out.send(&[0xB0, 20, 0]);
            }
            55 => {
                let _ = sim.out.send(&[0xB0, 20, 127]);
            }
            62 => {
                let m = self.midi.as_ref();
                let bound = m.and_then(|m| m.profiles().iter().find(|p| p.name == "Bhoreal").map(|p| p.bindings.iter().any(|b| b.msg == crate::midi::Msg::Cc(20) && b.addr == "/master/freeze")));
                eprintln!(
                    "selftest midi learn: {:?}; binding in the Bhoreal profile = {:?} (expected true); freeze after using it = {} (expected true)",
                    m.and_then(|m| m.learned.clone()),
                    bound,
                    self.s.master.freeze
                );
            }
            _ => {}
        }
        sim.step += 1;
    }

    fn selftest_step(&mut self, ctx: &egui::Context, gl: &glow::Context) {
        // VIDEOPING_SELFTEST_SHADERS=<dir>: compile every .fs in that folder, report, quit
        if let (Ok(dir), Some(r)) = (std::env::var("VIDEOPING_SELFTEST_SHADERS"), self.r.as_mut()) {
            let mut files: Vec<_> = std::fs::read_dir(&dir).into_iter().flatten().flatten().map(|e| e.path()).collect();
            files.sort();
            for f in files.iter().filter(|f| f.extension().is_some_and(|e| e == "fs")) {
                r.shader_info(gl, f);
                match r.shader_error(f) {
                    Some(e) => eprintln!("selftest shader FAIL {}: {e}", f.display()),
                    None => eprintln!("selftest shader ok   {}", f.display()),
                }
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let Some(t) = self.selftest.as_mut() else { return };
        let secs = t.start.elapsed().as_secs_f32();
        // NDI input test: receive our own NDI output as soon as it is discovered
        if std::env::var("VIDEOPING_SELFTEST_NDI").is_ok() && !self.s.slots.iter().flatten().any(|i| matches!(i.source, SourceKind::Ndi { .. })) {
            let own = self.ndi_finder.as_ref().and_then(|f| f.sources().into_iter().find(|s| s.ends_with("(videoping)")));
            if let Some(src) = own {
                eprintln!("selftest: NDI source {src:?} found after {secs:.1}s");
                if let Some(inst) = self.new_instance_from_file(Path::new(&src), &FileKind::Ndi) {
                    self.s.selected_instance = Some(inst.id);
                    self.s.slots[16] = Some(inst);
                }
            }
        }
        // VIDEOPING_SELFTEST_REC: record from 1.5 s to 4 s (file in the self-test folder)
        if std::env::var_os("VIDEOPING_SELFTEST_REC").is_some() {
            if let Some(t) = self.selftest.as_mut() {
                let secs = t.start.elapsed().as_secs_f32();
                // VIDEOPING_SELFTEST_REC_OFF: same timing without recording (overhead comparison)
                if t.rec_step == 0 && secs > 1.5 && crate::record::encoder().is_some() && std::env::var_os("VIDEOPING_SELFTEST_REC_OFF").is_none() {
                    t.rec_step = 1;
                    self.rec_request = Some(true);
                    // the recording folder is in the finder: the new file must show up there
                    let dir = self.cfg.record_dir();
                    if !self.s.finder_roots.contains(&dir) {
                        self.s.finder_roots.push(dir);
                    }
                } else if t.rec_step == 1 && secs > std::env::var("VIDEOPING_SELFTEST_REC").ok().and_then(|v| v.parse::<f32>().ok()).filter(|v| *v > 1.5).unwrap_or(4.0) {
                    t.rec_step = 2;
                    if let Some(rec) = self.r.as_ref().and_then(|r| r.rec.as_ref()) {
                        eprintln!("selftest rec: {} frames, {} dropped -> {}", rec.frames, rec.dropped, rec.path.display());
                        let gpu = self.r.as_ref().and_then(|r| r.meter.ms(Key::Record));
                        eprintln!("selftest rec meter: gpu {gpu:?} ms, cpu {:.2} ms, encoder queue {}", rec.cpu_ms, rec.queue());
                        t.rec_file = Some(rec.path.clone());
                    } else {
                        eprintln!("selftest rec: not recording ({})", self.status);
                    }
                    self.rec_request = Some(false);
                } else if t.rec_step == 2 && t.rec_stopped_at.is_none() {
                    t.rec_stopped_at = Some(secs);
                } else if t.rec_step == 2 && t.rec_stopped_at.is_some_and(|s0| secs > s0 + 2.5) {
                    t.rec_step = 3;
                    let listed = t.rec_file.as_ref().is_some_and(|f| self.finder.iter().any(|e| &e.path == f));
                    eprintln!("selftest rec finder: recording listed in the finder = {listed} (expected true)");
                }
            }
        }
        // VIDEOPING_SELFTEST_MIDI: a virtual Bhoreal presses pads, the LED feedback is checked
        if std::env::var_os("VIDEOPING_SELFTEST_MIDI").is_some() {
            self.selftest_midi();
        }
        // VIDEOPING_SELFTEST_CONTROL: drive the session through the control registry at 2 s
        if std::env::var_os("VIDEOPING_SELFTEST_CONTROL").is_some() && self.selftest.as_ref().is_some_and(|t| !t.control_done && t.start.elapsed().as_secs_f32() > 2.0) {
            use crate::control::Value;
            let _ = self.control_changes(); // baseline
            let n = self.control_list().len();
            let bpm0 = self.s.bpm;
            let steps: [(&str, Value); 6] = [
                ("/layer/1/mixtype", Value::Text("iLk".into())),
                ("/layer/1/mix", Value::Num(0.6)),
                ("/master/brightness", Value::Num(0.5)),
                ("/bpm/double", Value::Bool(true)),
                ("/layer/2/fx/1/active", Value::Bool(false)),
                ("/layer/9/mix", Value::Num(0.5)),
            ];
            for (a, v) in &steps {
                eprintln!("selftest control: set {a} = {v:?} -> {:?}", self.control_set(a, v));
            }
            let ch = self.control_changes();
            eprintln!(
                "selftest control: {n} addresses; layer 1 = {:?} {:.2}, brightness {:.2}, bpm {} -> {}, clip fx 1 on = {:?}",
                self.s.layers[0].mix.mix_type,
                self.s.layers[0].mix.amount,
                self.s.master.brightness,
                bpm0,
                self.s.bpm,
                self.control_get("/layer/2/fx/1/active")
            );
            eprintln!("selftest control: changes reported: {:?}", ch.iter().map(|c| c.0.as_str()).collect::<Vec<_>>());
            if let Some(t) = self.selftest.as_mut() {
                t.control_done = true;
            }
        }
        // clean status line for the screenshot (no temporary paths)
        if self.selftest.as_ref().is_some_and(|t| !t.shot && t.start.elapsed().as_secs_f32() > 4.0) {
            self.status.clear();
        }
        // resource meters of the session at screenshot time
        if self.selftest.as_ref().is_some_and(|t| !t.shot && t.start.elapsed().as_secs_f32() > 4.5) {
            eprintln!(
                "selftest prepare priority: main mix GPU {:.2} ms of {:.1} ms, prepare every {} frames (0 = paused)",
                self.main_ms,
                1000.0 / self.s.output.fps.max(1) as f32,
                self.prepare_div
            );
            for inst in self.s.slots.iter().flatten() {
                if let Some((level, tip)) = self.inst_usage(inst) {
                    eprintln!("selftest meter {}: level {level}, {}", inst.name, tip.replace('\n', " | "));
                }
            }
        }
        let Some(t) = self.selftest.as_mut() else { return };
        if secs > 4.0 && !t.dumped {
            t.dumped = true;
            if let Some(r) = self.r.as_ref() {
                let (w, h) = r.size();
                let px = r.read_output(gl);
                let _ = std::fs::write(t.dir.join(format!("out_{w}x{h}.rgba")), px);
            }
        }
        if secs > 4.5 && !t.shot {
            t.shot = true;
            let dir = t.dir.clone();
            ctx.request_screenshot(move |img| {
                let bytes: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
                let _ = std::fs::write(dir.join(format!("ui_{}x{}.rgba", img.size[0], img.size[1])), bytes);
            });
        }
        t.ui_frames += 1;
        if secs >= 2.0 && t.at2s.is_none() {
            t.at2s = Some((self.video_frames, t.ui_frames));
            t.prep0 = self.prepare_renders;
        }
        // a longer test recording (VIDEOPING_SELFTEST_REC=<seconds>) keeps the self-test open
        let close_at = std::env::var("VIDEOPING_SELFTEST_REC").ok().and_then(|v| v.parse::<f32>().ok()).map(|v| v + 4.0).unwrap_or(0.0).max(6.5);
        if secs > close_at && !t.closing {
            t.closing = true;
            let (v0, u0) = t.at2s.unwrap_or((0, 0));
            let span = secs - 2.0;
            eprintln!(
                "selftest fps (2 s .. {:.1} s): video {:.1} fps (output set to {}), ui {:.1} fps",
                secs,
                (self.video_frames - v0) as f32 / span,
                self.s.output.fps,
                (t.ui_frames - u0) as f32 / span
            );
            eprintln!("selftest prepare: selected non-live instance rendered {:.1} times per second", (self.prepare_renders - t.prep0) as f32 / span);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
