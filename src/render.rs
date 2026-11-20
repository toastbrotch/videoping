//! Render engine: instances (clip / generator + effect chain), mixer, output effects,
//! master, NDI readback.

use crate::gfx::{reset_state, Builtins, Gl, Program, Target, UploadTex};
use crate::isf::{self, InputKind, IsfInfo};
use crate::meter::{DecodeInfo, GpuTimer, Key};
use crate::record::Recorder;
use crate::ndi::{FrameMsg, NdiReceiver, NdiSender};
use crate::state::{EffectInst, Instance, Param, ParamValue, Rng, Session, SourceKind, NUM_LAYERS};
use crate::video::Player;
use glow::HasContext;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

pub const THUMB_W: i32 = 256;
pub const THUMB_H: i32 = 144;

pub struct Shader {
    pub info: Option<IsfInfo>,
    prog: Option<Program>,
    /// bool inputs had to be declared as float (see isf::build_fragment)
    bool_as_float: bool,
    pub error: Option<String>,
    mtime: Option<SystemTime>,
    checked: Instant,
}

fn mtime(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

impl Shader {
    fn load(gl: &Gl, path: &Path) -> Shader {
        let mt = mtime(path);
        let built = isf::parse_info(path).and_then(|(info, body)| {
            let custom_vs = isf::custom_vertex(path);
            let compile = |bool_as_float| {
                let vs = isf::build_vertex(&info, custom_vs.as_deref(), bool_as_float);
                let fs = isf::build_fragment(&info, &body, bool_as_float);
                Program::new(gl, &vs, &fs)
            };
            match compile(false) {
                Ok(p) => Ok((info, p, false)),
                Err(e) => {
                    let has_bool = info.inputs.iter().any(|i| matches!(i.kind, InputKind::Bool { .. } | InputKind::Event));
                    let retry = has_bool
                        .then(|| compile(true).ok())
                        .flatten();
                    match retry {
                        Some(p) => Ok((info, p, true)),
                        None => Err(format!("{}: {e}", info.name)),
                    }
                }
            }
        });
        match built {
            Ok((info, prog, bool_as_float)) => Shader {
                info: Some(info),
                prog: Some(prog),
                bool_as_float,
                error: None,
                mtime: mt,
                checked: Instant::now(),
            },
            Err(e) => {
                eprintln!("isf: {e}");
                Shader {
                    info: isf::parse_info(path).ok().map(|x| x.0),
                    prog: None,
                    bool_as_float: false,
                    error: Some(e),
                    mtime: mt,
                    checked: Instant::now(),
                }
            }
        }
    }
}

/// Render state of one ISF shader use (generator or effect): pass buffers, time.
#[derive(Default)]
pub struct PassState {
    targets: HashMap<String, [Target; 2]>,
    time: f32,
    frame: i32,
}

impl PassState {
    fn delete(&mut self, gl: &Gl) {
        for (_, [a, b]) in self.targets.drain() {
            a.delete(gl);
            b.delete(gl);
        }
    }
}

struct InstGpu {
    player: Option<Player>,
    ndi_recv: Option<NdiReceiver>,
    player_error: Option<String>,
    upload: UploadTex,
    src: Target,
    fx_raw: Target,
    ping: [Target; 2],
    thumb: Target,
    gen_state: PassState,
    out: Option<glow::Texture>,
    last_active: Instant,
    has_frame: bool,
}

impl InstGpu {
    fn new(gl: &Gl, w: i32, h: i32) -> InstGpu {
        InstGpu {
            player: None,
            ndi_recv: None,
            player_error: None,
            upload: UploadTex::new(gl),
            src: Target::new(gl, w, h),
            fx_raw: Target::new(gl, w, h),
            ping: [Target::new(gl, w, h), Target::new(gl, w, h)],
            thumb: Target::new(gl, THUMB_W, THUMB_H),
            gen_state: PassState::default(),
            out: None,
            last_active: Instant::now(),
            has_frame: false,
        }
    }
    fn delete(&mut self, gl: &Gl) {
        self.upload.delete(gl);
        self.src.delete(gl);
        self.fx_raw.delete(gl);
        for t in &self.ping {
            t.delete(gl);
        }
        self.thumb.delete(gl);
        self.gen_state.delete(gl);
    }
}

pub struct FrameCtx<'a> {
    pub dt: f32,
    pub bpm: f32,
    pub beat_prev: f64,
    pub beat_now: f64,
    pub rng: &'a mut Rng,
}

impl FrameCtx<'_> {
    /// did a jumpcut boundary (every `n` beats) pass this frame?
    pub fn jump(&self, n: f32) -> bool {
        let n = n.max(0.0625) as f64;
        (self.beat_now / n).floor() != (self.beat_prev / n).floor()
    }
}

/// Clip info shown in the ui (timeline).
#[derive(Clone, Copy, Default)]
pub struct ClipStatus {
    pub position: f64,
    pub duration: Option<f64>,
}

pub struct Renderer {
    pub b: Builtins,
    w: i32,
    h: i32,
    shaders: HashMap<PathBuf, Shader>,
    insts: HashMap<u64, InstGpu>,
    fx_states: HashMap<u64, PassState>,
    black: Target,
    comp: [Target; 2],
    fx_raw: Target,
    frozen: Target,
    pub out: Target,
    ndi_flip: Target,
    pbos: [glow::Buffer; 2],
    pbo_pending: [bool; 2],
    pbo_idx: usize,
    pub ndi: Option<NdiSender>,
    pub ndi_error: Option<String>,
    /// folder with libndi (from the output options), also used for NDI input
    pub ndi_lib_dir: String,
    /// NDI input status per instance: (w, h, fps), None = no signal
    pub ndi_status: HashMap<u64, Option<(i32, i32, f32)>>,
    ndi_key: Option<String>,
    pub clip_status: HashMap<u64, ClipStatus>,
    pub pending_scrub: Vec<(u64, f32)>,
    /// GPU time per source / effect (resource meters)
    pub meter: GpuTimer,
    /// recording of the output (None = not recording)
    pub rec: Option<Recorder>,
    /// NV12 conversion target (R8, w x h*3/2) and its readback buffers
    rec_nv12: Option<Target>,
    rec_pbos: [glow::Buffer; 2],
    rec_pending: [bool; 2],
    rec_idx: usize,
}

impl Renderer {
    pub fn new(gl: &Gl, w: i32, h: i32) -> Renderer {
        let pbos = unsafe { [gl.create_buffer().expect("pbo"), gl.create_buffer().expect("pbo")] };
        let rec_pbos = unsafe { [gl.create_buffer().expect("pbo"), gl.create_buffer().expect("pbo")] };
        let r = Renderer {
            b: Builtins::new(gl),
            w,
            h,
            shaders: HashMap::new(),
            insts: HashMap::new(),
            fx_states: HashMap::new(),
            black: Target::new(gl, 4, 4),
            comp: [Target::new(gl, w, h), Target::new(gl, w, h)],
            fx_raw: Target::new(gl, w, h),
            frozen: Target::new(gl, w, h),
            out: Target::new(gl, w, h),
            ndi_flip: Target::new(gl, w, h),
            pbos,
            pbo_pending: [false; 2],
            pbo_idx: 0,
            ndi: None,
            ndi_error: None,
            ndi_lib_dir: String::new(),
            ndi_status: HashMap::new(),
            ndi_key: None,
            clip_status: HashMap::new(),
            pending_scrub: vec![],
            meter: GpuTimer::new(),
            rec: None,
            rec_nv12: None,
            rec_pbos,
            rec_pending: [false; 2],
            rec_idx: 0,
        };
        reset_state(gl);
        r
    }

    pub fn size(&self) -> (i32, i32) {
        (self.w, self.h)
    }

    /// Change output resolution: all full-size targets are recreated.
    pub fn resize(&mut self, gl: &Gl, w: i32, h: i32) {
        if w == self.w && h == self.h {
            return;
        }
        self.w = w;
        self.h = h;
        // a recording has one fixed size: it ends when the output resolution changes
        self.stop_recording(gl);
        for t in self.comp.iter().chain([&self.fx_raw, &self.frozen, &self.out, &self.ndi_flip]) {
            t.delete(gl);
        }
        self.comp = [Target::new(gl, w, h), Target::new(gl, w, h)];
        self.fx_raw = Target::new(gl, w, h);
        self.frozen = Target::new(gl, w, h);
        self.out = Target::new(gl, w, h);
        self.ndi_flip = Target::new(gl, w, h);
        for (_, mut g) in self.insts.drain() {
            g.delete(gl);
        }
        for (_, mut s) in self.fx_states.drain() {
            s.delete(gl);
        }
        self.pbo_pending = [false; 2];
    }

    /// Drop the GPU state of all instances and effects (new / load session). Output buffers, NDI
    /// and a running recording stay.
    pub fn drop_all_instances(&mut self, gl: &Gl) {
        for (_, mut g) in self.insts.drain() {
            g.delete(gl);
        }
        for (_, mut st) in self.fx_states.drain() {
            st.delete(gl);
        }
        self.clip_status.clear();
        self.ndi_status.clear();
    }

    pub fn shader_info(&mut self, gl: &Gl, path: &Path) -> Option<&IsfInfo> {
        self.shader(gl, path).info.as_ref()
    }

    pub fn shader_error(&self, path: &Path) -> Option<&str> {
        self.shaders.get(path).and_then(|s| s.error.as_deref())
    }

    /// Get (compile on first use, hot-reload when the file changes).
    fn shader(&mut self, gl: &Gl, path: &Path) -> &mut Shader {
        let reload = match self.shaders.get_mut(path) {
            None => true,
            Some(s) => {
                if s.checked.elapsed().as_millis() > 1000 {
                    s.checked = Instant::now();
                    mtime(path) != s.mtime
                } else {
                    false
                }
            }
        };
        if reload {
            if let Some(old) = self.shaders.remove(path) {
                if let Some(p) = old.prog {
                    p.delete(gl);
                }
            }
            self.shaders.insert(path.to_path_buf(), Shader::load(gl, path));
        }
        self.shaders.get_mut(path).unwrap()
    }

    pub fn player_error(&self, id: u64) -> Option<&str> {
        self.insts.get(&id).and_then(|g| g.player_error.as_deref())
    }

    pub fn thumb(&self, id: u64) -> Option<glow::Texture> {
        self.insts.get(&id).filter(|g| g.has_frame).map(|g| g.thumb.tex)
    }

    /// Own VRAM of an instance (source, effect ping-pong, thumbnail, generator passes).
    pub fn vram_inst(&self, id: u64) -> u64 {
        let Some(g) = self.insts.get(&id) else { return 0 };
        let t = |t: &Target| t.w as u64 * t.h as u64 * 4;
        t(&g.src) + t(&g.fx_raw) + t(&g.ping[0]) + t(&g.ping[1]) + t(&g.thumb)
            + g.upload.w.max(0) as u64 * g.upload.h.max(0) as u64 * 4
            + pass_bytes(&g.gen_state)
    }

    /// Own VRAM of an effect (its persistent / pass buffers).
    pub fn vram_fx(&self, uid: u64) -> u64 {
        self.fx_states.get(&uid).map(pass_bytes).unwrap_or(0)
    }

    /// All VRAM videoping allocated for frames (instances, effects, mixer / output buffers).
    pub fn vram_total(&self) -> u64 {
        let t = |t: &Target| t.w as u64 * t.h as u64 * 4;
        let own: u64 = self.comp.iter().chain([&self.fx_raw, &self.frozen, &self.out, &self.ndi_flip]).map(t).sum();
        own + self.insts.keys().map(|id| self.vram_inst(*id)).sum::<u64>() + self.fx_states.values().map(pass_bytes).sum::<u64>()
    }

    /// Decoding info of a clip instance (None: not a clip or not playing).
    pub fn decode_info(&self, id: u64) -> Option<DecodeInfo> {
        self.insts.get(&id).and_then(|g| g.player.as_ref()).map(|p| p.decode_info())
    }

    pub fn output_of(&self, id: u64) -> Option<glow::Texture> {
        self.insts.get(&id).and_then(|g| g.out)
    }

    /// Run all passes of an ISF shader into `dst`.
    #[allow(clippy::too_many_arguments)]
    fn run_isf(
        &mut self,
        gl: &Gl,
        path: &Path,
        params: &mut [Param],
        state_key: StateKey,
        input: Option<glow::Texture>,
        dst: &Target,
        seed: f32,
        dt: f32,
    ) -> bool {
        let black = self.black.tex;
        let (w, h) = (dst.w, dst.h);
        let shader = self.shader(gl, path);
        let (Some(info), Some(_)) = (shader.info.clone(), shader.prog.as_ref()) else {
            dst.clear(gl);
            return false;
        };
        // take the pass state out to avoid double borrows
        let mut state = match state_key {
            StateKey::Gen(id) => self.insts.get_mut(&id).map(|g| std::mem::take(&mut g.gen_state)).unwrap_or_default(),
            StateKey::Fx(uid) => self.fx_states.remove(&uid).unwrap_or_default(),
        };
        state.time += dt;
        let npasses = info.passes.len();
        for (pi, pass) in info.passes.iter().enumerate() {
            if let Some(t) = &pass.target {
                if !state.targets.contains_key(t) {
                    state.targets.insert(t.clone(), [Target::new(gl, w, h), Target::new(gl, w, h)]);
                }
            }
            let shader = self.shaders.get_mut(path).unwrap();
            let bool_as_float = shader.bool_as_float;
            let prog = shader.prog.as_mut().unwrap();
            match &pass.target {
                Some(t) => state.targets[t][1].bind(gl),
                None => dst.bind(gl),
            }
            prog.use_prog(gl);
            prog.f1(gl, "TIME", state.time);
            prog.f1(gl, "TIMEDELTA", dt);
            prog.f2(gl, "RENDERSIZE", w as f32, h as f32);
            prog.i1(gl, "PASSINDEX", pi as i32);
            prog.i1(gl, "FRAMEINDEX", state.frame);
            prog.f1(gl, "uInstanceSeed", seed);
            prog.f4(gl, "DATE", date_uniform());
            let mut unit = 0u32;
            for inp in &info.inputs {
                let p = params.iter().find(|p| p.name == inp.name);
                match (&inp.kind, p.map(|p| &p.value)) {
                    (InputKind::Float { .. }, Some(ParamValue::Anim(a))) => prog.f1(gl, &inp.name, a[0].value),
                    (InputKind::Long { .. }, Some(ParamValue::Anim(a))) => {
                        prog.i1(gl, &inp.name, a[0].value.round() as i32)
                    }
                    (InputKind::Bool { .. } | InputKind::Event, Some(ParamValue::Bool(v))) => {
                        if bool_as_float {
                            prog.f1(gl, &inp.name, *v as i32 as f32)
                        } else {
                            prog.i1(gl, &inp.name, *v as i32)
                        }
                    }
                    (InputKind::Color { .. }, Some(ParamValue::Color(c))) => prog.f4(gl, &inp.name, *c),
                    (InputKind::Point2D { .. }, Some(ParamValue::Anim(a))) if a.len() == 2 => {
                        prog.f2(gl, &inp.name, a[0].value, a[1].value)
                    }
                    (InputKind::Image, _) => {
                        let tex = if inp.name == "inputImage" { input.unwrap_or(black) } else { black };
                        prog.tex(gl, &inp.name, unit, Some(tex));
                        unit += 1;
                    }
                    (InputKind::Audio, _) => {
                        prog.tex(gl, &inp.name, unit, Some(black));
                        unit += 1;
                    }
                    _ => {}
                }
            }
            for (name, pair) in &state.targets {
                prog.tex(gl, name, unit, Some(pair[0].tex));
                unit += 1;
            }
            // pass target bound again (tex() calls don't change the framebuffer)
            match &pass.target {
                Some(t) => state.targets[t][1].bind(gl),
                None => dst.bind(gl),
            }
            self.b.quad.draw(gl);
            if let Some(t) = &pass.target {
                let pair = state.targets.get_mut(t).unwrap();
                pair.swap(0, 1);
                if pi + 1 == npasses {
                    let tex = pair[0].tex;
                    self.b.blit(gl, tex, w, h, dst, false);
                }
            }
        }
        // events are one-frame pulses
        for inp in &info.inputs {
            if inp.kind == InputKind::Event {
                if let Some(p) = params.iter_mut().find(|p| p.name == inp.name) {
                    p.value = ParamValue::Bool(false);
                }
            }
        }
        state.frame += 1;
        match state_key {
            StateKey::Gen(id) => {
                if let Some(g) = self.insts.get_mut(&id) {
                    g.gen_state = state;
                }
            }
            StateKey::Fx(uid) => {
                self.fx_states.insert(uid, state);
            }
        }
        true
    }

    /// Effect chain: input -> (effect -> mix with input) * n. Returns the result texture.
    fn run_effects(
        &mut self,
        gl: &Gl,
        effects: &mut [EffectInst],
        input: glow::Texture,
        raw: Option<u64>,
        pings: PingSel,
        fc: &mut FrameCtx,
    ) -> glow::Texture {
        let mut cur = input;
        let mut pi = 0;
        for fx in effects.iter_mut() {
            tick_params(&mut fx.params, fc);
            if !fx.active {
                continue;
            }
            let idx = pi;
            let (raw_t, ping_t) = self.take_targets(raw, pings, idx);
            self.meter.begin(gl, Key::Fx(fx.uid));
            let ok = self.run_isf(gl, &fx.path, &mut fx.params, StateKey::Fx(fx.uid), Some(cur), &raw_t, fx.seed, fc.dt);
            if ok {
                self.b.mix(gl, cur, raw_t.tex, &fx.mix, &ping_t);
                cur = ping_t.tex;
                pi ^= 1;
            }
            self.meter.end(gl);
            self.put_targets(raw, pings, raw_t, ping_t, idx);
        }
        cur
    }

    // Targets are moved out temporarily so that `run_isf(&mut self)` can be called.
    fn take_targets(&mut self, raw: Option<u64>, pings: PingSel, pi: usize) -> (Target, Target) {
        match (raw, pings) {
            (Some(id), PingSel::Inst) => {
                let g = self.insts.get_mut(&id).unwrap();
                let r = std::mem::replace(&mut g.fx_raw, ph(&self.black));
                let p = std::mem::replace(&mut g.ping[pi], ph(&self.black));
                (r, p)
            }
            _ => {
                let r = std::mem::replace(&mut self.fx_raw, ph(&self.black));
                let p = std::mem::replace(&mut self.comp[pi], ph(&self.black));
                (r, p)
            }
        }
    }

    fn put_targets(&mut self, raw: Option<u64>, pings: PingSel, r: Target, p: Target, pi: usize) {
        match (raw, pings) {
            (Some(id), PingSel::Inst) => {
                let g = self.insts.get_mut(&id).unwrap();
                g.fx_raw = r;
                g.ping[pi] = p;
            }
            _ => {
                self.fx_raw = r;
                self.comp[pi] = p;
            }
        }
    }

    /// Render one instance (source + effects). Returns its output texture.
    pub fn render_instance(&mut self, gl: &Gl, inst: &mut Instance, fc: &mut FrameCtx) -> Option<glow::Texture> {
        let (w, h) = (self.w, self.h);
        let id = inst.id;
        self.insts.entry(id).or_insert_with(|| InstGpu::new(gl, w, h));
        self.meter.begin(gl, Key::Src(id));
        let g = self.insts.get_mut(&id).unwrap();
        g.last_active = Instant::now();

        match &mut inst.source {
            SourceKind::Clip { path, transport } => {
                if g.player.is_none() && g.player_error.is_none() {
                    match Player::new(path) {
                        Ok(p) => g.player = Some(p),
                        Err(e) => {
                            eprintln!("video: {e}");
                            g.player_error = Some(e);
                        }
                    }
                }
                if let Some(p) = g.player.as_mut() {
                    if let Some(pos) = self.pending_scrub.iter().rev().find(|s| s.0 == id).map(|s| s.1) {
                        p.scrub(pos);
                    }
                    p.update(transport, fc.bpm, fc.jump(transport.speed.jump_beats()), fc.rng);
                    if let Some(e) = p.error.take() {
                        g.player_error = Some(e);
                    }
                    self.clip_status.insert(id, ClipStatus { position: p.position, duration: p.duration });
                    let up = &mut g.upload;
                    let got = p.take_frame(|f| up.upload(gl, f.w, f.h, f.stride, &f.data));
                    if got.is_some() {
                        g.has_frame = true;
                    }
                    if g.upload.w > 0 {
                        let (tex, uw, uh) = (g.upload.tex, g.upload.w, g.upload.h);
                        let src = std::mem::replace(&mut g.src, ph(&self.black));
                        self.b.blit(gl, tex, uw, uh, &src, true);
                        self.insts.get_mut(&id).unwrap().src = src;
                    }
                }
            }
            SourceKind::Ndi { source } => {
                // retry once libndi got loaded (e.g. after setting the NDI library folder)
                if g.ndi_recv.is_none() && g.player_error.is_some() && crate::ndi::loaded().is_some() {
                    g.player_error = None;
                }
                if g.ndi_recv.is_none() && g.player_error.is_none() {
                    match crate::ndi::lib(&self.ndi_lib_dir).and_then(|l| NdiReceiver::new(l, source)) {
                        Ok(r) => g.ndi_recv = Some(r),
                        Err(e) => g.player_error = Some(e),
                    }
                }
                if let Some(rx) = g.ndi_recv.as_mut() {
                    let up = &mut g.upload;
                    if rx.take_frame(|f| up.upload(gl, f.w, f.h, f.stride, &f.data)).is_some() {
                        g.has_frame = true;
                    }
                    self.ndi_status.insert(id, rx.status());
                    if g.upload.w > 0 {
                        let (tex, uw, uh) = (g.upload.tex, g.upload.w, g.upload.h);
                        let src = std::mem::replace(&mut g.src, ph(&self.black));
                        self.b.blit(gl, tex, uw, uh, &src, true);
                        self.insts.get_mut(&id).unwrap().src = src;
                    }
                }
            }
            SourceKind::Generator { path, params } => {
                tick_params(params, fc);
                let src = std::mem::replace(&mut g.src, ph(&self.black));
                let path = path.clone();
                self.run_isf(gl, &path, params, StateKey::Gen(id), None, &src, inst.seed, fc.dt);
                let g = self.insts.get_mut(&id).unwrap();
                g.src = src;
                g.has_frame = true;
            }
        }
        self.pending_scrub.retain(|s| s.0 != id);
        self.meter.end(gl);

        let src_tex = self.insts[&id].src.tex;
        let out = self.run_effects(gl, &mut inst.effects, src_tex, Some(id), PingSel::Inst, fc);
        // thumbnail = last rendered frame (static when the instance is not active)
        let g = self.insts.get_mut(&id).unwrap();
        g.out = Some(out);
        if g.has_frame {
            let thumb = std::mem::replace(&mut g.thumb, ph(&self.black));
            self.meter.begin(gl, Key::Src(id));
            self.b.blit(gl, out, w, h, &thumb, false);
            self.meter.end(gl);
            self.insts.get_mut(&id).unwrap().thumb = thumb;
        }
        Some(out)
    }

    /// Mixer + output effects + master. Returns the output texture.
    pub fn render_output(
        &mut self,
        gl: &Gl,
        s: &mut Session,
        layer_tex: [Option<glow::Texture>; NUM_LAYERS],
        fc: &mut FrameCtx,
    ) -> glow::Texture {
        // mixer: bottom layer (3) first, top layer (1) last
        self.meter.begin(gl, Key::Mixer);
        self.comp[0].clear(gl);
        let mut cur = 0usize;
        for li in (0..NUM_LAYERS).rev() {
            let layer = &s.layers[li];
            if !layer.active {
                continue;
            }
            if let Some(tex) = layer_tex[li] {
                let (a, b) = if cur == 0 { (0, 1) } else { (1, 0) };
                let bottom = self.comp[a].tex;
                let dst = std::mem::replace(&mut self.comp[b], ph(&self.black));
                self.b.mix(gl, bottom, tex, &layer.mix, &dst);
                self.comp[b] = dst;
                cur = b;
            }
        }
        // copy mix result into fx_raw-independent buffer: use frozen as staging only if not frozen
        let mix_tex = self.comp[cur].tex;
        // output effects use comp ping-pong; first move the mix result to a stable texture
        let stage = std::mem::replace(&mut self.ndi_flip, ph(&self.black));
        let (w, h) = (self.w, self.h);
        self.b.blit(gl, mix_tex, w, h, &stage, false);
        let stage_tex = stage.tex;
        self.meter.end(gl);
        let result = self.run_effects(gl, &mut s.output_effects, stage_tex, None, PingSel::Comp, fc);
        self.meter.begin(gl, Key::Master);
        if !s.master.freeze {
            self.b.blit(gl, result, w, h, &self.frozen, false);
        }
        self.ndi_flip = stage;
        let brightness = if s.master.blackout { 0.0 } else { s.master.brightness };
        let frozen = self.frozen.tex;
        let out = std::mem::replace(&mut self.out, ph(&self.black));
        self.b.master(gl, frozen, brightness, &out);
        self.out = out;

        if self.ndi.is_some() {
            self.ndi_readback(gl);
        }
        self.meter.end(gl);
        if self.rec.is_some() {
            self.meter.begin(gl, Key::Record);
            self.rec_readback(gl);
            self.meter.end(gl);
        }
        self.meter.end_frame(gl);
        reset_state(gl);
        self.out.tex
    }

    /// Async readback through two PBOs: read this frame, send the previous one.
    fn ndi_readback(&mut self, gl: &Gl) {
        let (w, h) = (self.w, self.h);
        let size = (w * h * 4) as usize;
        let out_tex = self.out.tex;
        let flip = std::mem::replace(&mut self.ndi_flip, ph(&self.black));
        self.b.blit(gl, out_tex, w, h, &flip, true); // NDI wants top-down rows
        unsafe {
            let i = self.pbo_idx;
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(flip.fbo));
            gl.bind_buffer(glow::PIXEL_PACK_BUFFER, Some(self.pbos[i]));
            gl.buffer_data_size(glow::PIXEL_PACK_BUFFER, size as i32, glow::STREAM_READ);
            gl.read_pixels(0, 0, w, h, glow::RGBA, glow::UNSIGNED_BYTE, glow::PixelPackData::BufferOffset(0));
            self.pbo_pending[i] = true;
            let j = i ^ 1;
            if self.pbo_pending[j] {
                gl.bind_buffer(glow::PIXEL_PACK_BUFFER, Some(self.pbos[j]));
                let ptr = gl.map_buffer_range(glow::PIXEL_PACK_BUFFER, 0, size as i32, glow::MAP_READ_BIT);
                if !ptr.is_null() {
                    if let Some(ndi) = &self.ndi {
                        let mut buf = ndi.buffer(size);
                        std::ptr::copy_nonoverlapping(ptr, buf.as_mut_ptr(), size);
                        ndi.send(FrameMsg { w, h, data: buf });
                    }
                    gl.unmap_buffer(glow::PIXEL_PACK_BUFFER);
                }
                self.pbo_pending[j] = false;
            }
            gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
            self.pbo_idx = j;
        }
        self.ndi_flip = flip;
    }

    /// Start recording the output to `path` (H.264 mp4).
    pub fn start_recording(&mut self, gl: &Gl, path: &Path, fps: u32) -> Result<(), String> {
        self.stop_recording(gl);
        let (w, h) = (self.w & !1, self.h & !1); // NV12 needs even sizes
        let rec = Recorder::start(path, w, h, fps)?;
        self.rec_nv12 = Some(Target::new_r8(gl, w, h * 3 / 2));
        self.rec_pending = [false; 2];
        self.rec = Some(rec);
        Ok(())
    }

    /// Stop recording (the file is finished in the background; the handle ends when it is done).
    pub fn stop_recording(&mut self, gl: &Gl) -> Option<std::thread::JoinHandle<()>> {
        let h = self.rec.take().map(|r| r.stop());
        if let Some(t) = self.rec_nv12.take() {
            t.delete(gl);
        }
        self.rec_pending = [false; 2];
        h
    }

    /// NV12 conversion on the GPU, async readback through two PBOs: read this frame, push the
    /// previous one to the encoder (the CPU never waits for the GPU).
    fn rec_readback(&mut self, gl: &Gl) {
        let (Some(rec), Some(nv)) = (self.rec.as_mut(), self.rec_nv12.as_ref()) else { return };
        rec.poll();
        let (w, h) = (rec.w, rec.h);
        let size = (w * h * 3 / 2) as usize;
        self.b.nv12(gl, self.out.tex, w, h, nv);
        unsafe {
            let i = self.rec_idx;
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(nv.fbo));
            gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1);
            gl.bind_buffer(glow::PIXEL_PACK_BUFFER, Some(self.rec_pbos[i]));
            gl.buffer_data_size(glow::PIXEL_PACK_BUFFER, size as i32, glow::STREAM_READ);
            gl.read_pixels(0, 0, w, h * 3 / 2, glow::RED, glow::UNSIGNED_BYTE, glow::PixelPackData::BufferOffset(0));
            gl.pixel_store_i32(glow::PACK_ALIGNMENT, 4);
            self.rec_pending[i] = true;
            let j = i ^ 1;
            if self.rec_pending[j] {
                gl.bind_buffer(glow::PIXEL_PACK_BUFFER, Some(self.rec_pbos[j]));
                let ptr = gl.map_buffer_range(glow::PIXEL_PACK_BUFFER, 0, size as i32, glow::MAP_READ_BIT);
                if !ptr.is_null() {
                    rec.push(std::slice::from_raw_parts(ptr, size));
                    gl.unmap_buffer(glow::PIXEL_PACK_BUFFER);
                }
                self.rec_pending[j] = false;
            }
            gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
            self.rec_idx = j;
        }
    }

    pub fn set_ndi(&mut self, enabled: bool, name: &str, lib_dir: &str, fps: i32) {
        if self.ndi_lib_dir != lib_dir {
            self.ndi_lib_dir = lib_dir.to_string();
        }
        let want = enabled.then(|| format!("{name}\u{0}{lib_dir}\u{0}{fps}"));
        if want == self.ndi_key {
            return;
        }
        self.ndi = None;
        self.ndi_error = None;
        self.pbo_pending = [false; 2];
        self.ndi_key = want.clone();
        if want.is_some() {
            // tried once per name / folder (not every frame)
            match NdiSender::new(name, fps, lib_dir) {
                Ok(s) => {
                    eprintln!("ndi: sending as \"{name}\" ({})", s.lib_path);
                    self.ndi = Some(s);
                }
                Err(e) => {
                    eprintln!("ndi: {e}");
                    self.ndi_error = Some(e);
                }
            }
        }
    }

    /// Free GPU state of deleted instances; stop players of long inactive instances.
    pub fn gc(&mut self, gl: &Gl, s: &Session) {
        let alive: std::collections::HashSet<u64> = s.slots.iter().flatten().map(|i| i.id).collect();
        let dead: Vec<u64> = self.insts.keys().filter(|k| !alive.contains(k) && **k != PREVIEW_ID).copied().collect();
        for id in dead {
            if let Some(mut g) = self.insts.remove(&id) {
                g.delete(gl);
            }
            self.clip_status.remove(&id);
            self.ndi_status.remove(&id);
        }
        for g in self.insts.values_mut() {
            if g.last_active.elapsed().as_secs() > 10 {
                g.player = None;
                g.ndi_recv = None;
            }
        }
        let fx_alive: std::collections::HashSet<u64> = s
            .slots
            .iter()
            .flatten()
            .flat_map(|i| i.effects.iter().map(|e| e.uid))
            .chain(s.output_effects.iter().map(|e| e.uid))
            .collect();
        let dead_fx: Vec<u64> = self.fx_states.keys().filter(|k| !fx_alive.contains(k)).copied().collect();
        for uid in dead_fx {
            if let Some(mut st) = self.fx_states.remove(&uid) {
                st.delete(gl);
            }
        }
    }

    /// Synchronous readback of the output (self-test only), rows bottom-up.
    pub fn read_output(&self, gl: &Gl) -> Vec<u8> {
        let mut buf = vec![0u8; (self.w * self.h * 4) as usize];
        unsafe {
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.out.fbo));
            gl.read_pixels(0, 0, self.w, self.h, glow::RGBA, glow::UNSIGNED_BYTE, glow::PixelPackData::Slice(Some(&mut buf)));
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
        }
        buf
    }

    /// Drop the finder preview's GPU state.
    pub fn drop_instance(&mut self, gl: &Gl, id: u64) {
        if let Some(mut g) = self.insts.remove(&id) {
            g.delete(gl);
        }
    }
}

fn pass_bytes(s: &PassState) -> u64 {
    s.targets.values().flat_map(|p| p.iter()).map(|t| t.w as u64 * t.h as u64 * 4).sum()
}

/// Placeholder while a target is moved out (never drawn to, never deleted).
fn ph(t: &Target) -> Target {
    Target { tex: t.tex, fbo: t.fbo, w: 0, h: 0 }
}

/// Id of the transient finder preview instance.
pub const PREVIEW_ID: u64 = u64::MAX;

#[derive(Clone, Copy)]
enum StateKey {
    Gen(u64),
    Fx(u64),
}

#[derive(Clone, Copy)]
enum PingSel {
    Inst,
    Comp,
}

pub fn tick_params(params: &mut [Param], fc: &mut FrameCtx) {
    for p in params {
        if let ParamValue::Anim(chs) = &mut p.value {
            for a in chs {
                let jump = a.mode == crate::state::PlayMode::Jumpcut && fc.jump(a.speed.jump_beats());
                a.tick(fc.dt, fc.bpm, jump, fc.rng);
            }
        }
    }
}

fn date_uniform() -> [f32; 4] {
    let secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0);
    let days = (secs / 86400.0).floor();
    let sod = secs - days * 86400.0;
    // approximate year/month/day (good enough for shaders that animate on DATE)
    let year = 1970.0 + (days / 365.2425).floor();
    [year as f32, 1.0, 1.0, sod as f32]
}
