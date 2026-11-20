//! Small OpenGL helpers on top of glow: render targets, programs, fullscreen quad
//! and the built-in shaders (blit, mix, master).

use glow::HasContext;
use std::collections::HashMap;

pub type Gl = glow::Context;

/// Texture + framebuffer.
pub struct Target {
    pub tex: glow::Texture,
    pub fbo: glow::Framebuffer,
    pub w: i32,
    pub h: i32,
}

impl Target {
    pub fn new(gl: &Gl, w: i32, h: i32) -> Target {
        Target::with_format(gl, w, h, glow::RGBA8, glow::RGBA)
    }

    /// One byte per pixel (recording: NV12 planes).
    pub fn new_r8(gl: &Gl, w: i32, h: i32) -> Target {
        Target::with_format(gl, w, h, glow::R8, glow::RED)
    }

    fn with_format(gl: &Gl, w: i32, h: i32, internal: u32, format: u32) -> Target {
        unsafe {
            let tex = gl.create_texture().expect("texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                internal as i32,
                w,
                h,
                0,
                format,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            let fbo = gl.create_framebuffer().expect("fbo");
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(tex), 0);
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            Target { tex, fbo, w, h }
        }
    }

    pub fn bind(&self, gl: &Gl) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
            gl.viewport(0, 0, self.w, self.h);
        }
    }

    pub fn clear(&self, gl: &Gl) {
        self.bind(gl);
        unsafe {
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    pub fn delete(&self, gl: &Gl) {
        unsafe {
            gl.delete_framebuffer(self.fbo);
            gl.delete_texture(self.tex);
        }
    }
}

/// Plain texture for CPU uploads (video frames).
pub struct UploadTex {
    pub tex: glow::Texture,
    pub w: i32,
    pub h: i32,
}

impl UploadTex {
    pub fn new(gl: &Gl) -> Self {
        unsafe {
            let tex = gl.create_texture().expect("texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            UploadTex { tex, w: 0, h: 0 }
        }
    }

    /// Upload tightly packed RGBA rows (stride in bytes).
    pub fn upload(&mut self, gl: &Gl, w: i32, h: i32, stride: i32, data: &[u8]) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.tex));
            gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, stride / 4);
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
            if w != self.w || h != self.h {
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA8 as i32,
                    w,
                    h,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(data)),
                );
                self.w = w;
                self.h = h;
            } else {
                gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    0,
                    0,
                    w,
                    h,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(data)),
                );
            }
            gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, 0);
        }
    }

    pub fn delete(&self, gl: &Gl) {
        unsafe { gl.delete_texture(self.tex) }
    }
}

pub struct Program {
    pub prog: glow::Program,
    uniforms: HashMap<String, Option<glow::UniformLocation>>,
}

impl Program {
    pub fn new(gl: &Gl, vs: &str, fs: &str) -> Result<Program, String> {
        unsafe {
            let prog = gl.create_program()?;
            let mut shaders = vec![];
            for (ty, src) in [(glow::VERTEX_SHADER, vs), (glow::FRAGMENT_SHADER, fs)] {
                let sh = gl.create_shader(ty)?;
                gl.shader_source(sh, src);
                gl.compile_shader(sh);
                if !gl.get_shader_compile_status(sh) {
                    let log = gl.get_shader_info_log(sh);
                    gl.delete_shader(sh);
                    for s in shaders {
                        gl.delete_shader(s);
                    }
                    gl.delete_program(prog);
                    let kind = if ty == glow::VERTEX_SHADER { "vertex" } else { "fragment" };
                    return Err(format!("{kind} shader: {log}"));
                }
                gl.attach_shader(prog, sh);
                shaders.push(sh);
            }
            gl.bind_attrib_location(prog, 0, "isf_position");
            gl.link_program(prog);
            for s in shaders {
                gl.detach_shader(prog, s);
                gl.delete_shader(s);
            }
            if !gl.get_program_link_status(prog) {
                let log = gl.get_program_info_log(prog);
                gl.delete_program(prog);
                return Err(format!("link: {log}"));
            }
            Ok(Program { prog, uniforms: HashMap::new() })
        }
    }

    pub fn loc(&mut self, gl: &Gl, name: &str) -> Option<glow::UniformLocation> {
        if let Some(l) = self.uniforms.get(name) {
            return l.clone();
        }
        let l = unsafe { gl.get_uniform_location(self.prog, name) };
        self.uniforms.insert(name.to_string(), l.clone());
        l
    }

    pub fn use_prog(&self, gl: &Gl) {
        unsafe { gl.use_program(Some(self.prog)) }
    }

    pub fn f1(&mut self, gl: &Gl, name: &str, v: f32) {
        let l = self.loc(gl, name);
        unsafe { gl.uniform_1_f32(l.as_ref(), v) }
    }
    pub fn i1(&mut self, gl: &Gl, name: &str, v: i32) {
        let l = self.loc(gl, name);
        unsafe { gl.uniform_1_i32(l.as_ref(), v) }
    }
    pub fn f2(&mut self, gl: &Gl, name: &str, a: f32, b: f32) {
        let l = self.loc(gl, name);
        unsafe { gl.uniform_2_f32(l.as_ref(), a, b) }
    }
    pub fn f3(&mut self, gl: &Gl, name: &str, v: [f32; 3]) {
        let l = self.loc(gl, name);
        unsafe { gl.uniform_3_f32(l.as_ref(), v[0], v[1], v[2]) }
    }
    pub fn f4(&mut self, gl: &Gl, name: &str, v: [f32; 4]) {
        let l = self.loc(gl, name);
        unsafe { gl.uniform_4_f32(l.as_ref(), v[0], v[1], v[2], v[3]) }
    }
    /// bind texture to unit and set sampler uniform
    pub fn tex(&mut self, gl: &Gl, name: &str, unit: u32, tex: Option<glow::Texture>) {
        let l = self.loc(gl, name);
        unsafe {
            gl.active_texture(glow::TEXTURE0 + unit);
            gl.bind_texture(glow::TEXTURE_2D, tex);
            gl.uniform_1_i32(l.as_ref(), unit as i32);
        }
    }

    pub fn delete(&self, gl: &Gl) {
        unsafe { gl.delete_program(self.prog) }
    }
}

/// Fullscreen quad (triangle strip, attribute 0 = position in clip space).
pub struct Quad {
    vao: glow::VertexArray,
    _vbo: glow::Buffer,
}

impl Quad {
    pub fn new(gl: &Gl) -> Quad {
        unsafe {
            let vao = gl.create_vertex_array().expect("vao");
            let vbo = gl.create_buffer().expect("vbo");
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let verts: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
            let bytes: &[u8] = std::slice::from_raw_parts(verts.as_ptr() as *const u8, verts.len() * 4);
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            Quad { vao, _vbo: vbo }
        }
    }

    pub fn draw(&self, gl: &Gl) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            gl.bind_vertex_array(None);
        }
    }
}

const VS: &str = "#version 330
in vec2 isf_position;
out vec2 uv;
void main() { gl_Position = vec4(isf_position, 0.0, 1.0); uv = isf_position * 0.5 + 0.5; }
";

/// Copies a texture, optionally flipping vertically and letterboxing (fit) by aspect.
const BLIT_FS: &str = "#version 330
in vec2 uv;
out vec4 o;
uniform sampler2D src;
uniform int flip;
uniform vec2 scale; // fit: >1 = letterbox
void main() {
    vec2 p = (uv - 0.5) * scale + 0.5;
    if (flip == 1) p.y = 1.0 - p.y;
    if (p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0) { o = vec4(0.0, 0.0, 0.0, 1.0); return; }
    o = vec4(texture(src, p).rgb, 1.0);
}
";

/// Mixes `top` over `bottom`: mix-type, amount, rgb channel mask.
const MIX_FS: &str = "#version 330
in vec2 uv;
out vec4 o;
uniform sampler2D bottom;
uniform sampler2D top;
uniform int mixType;   // 0 alpha, 1 lumakey, 2 inv lumakey, 3 add
uniform float amount;
uniform vec3 rgb;
void main() {
    vec4 b = texture(bottom, uv);
    vec4 t = texture(top, uv);
    float l = dot(t.rgb, vec3(0.299, 0.587, 0.114));
    vec3 r;
    if (mixType == 0)      r = mix(b.rgb, t.rgb, t.a * amount);
    else if (mixType == 1) r = mix(b.rgb, t.rgb, smoothstep(0.05, 0.35, l) * amount);
    else if (mixType == 2) r = mix(b.rgb, t.rgb, (1.0 - smoothstep(0.05, 0.35, l)) * amount);
    else                   r = b.rgb + t.rgb * amount;
    r = mix(b.rgb, r, rgb);
    o = vec4(clamp(r, 0.0, 1.0), 1.0);
}
";

/// Master: brightness, blackout.
const MASTER_FS: &str = "#version 330
in vec2 uv;
out vec4 o;
uniform sampler2D src;
uniform float brightness;
void main() { o = vec4(texture(src, uv).rgb * brightness, 1.0); }
";

/// RGBA -> NV12 (BT.709, limited range) for recording, into an R8 target of w x h*3/2:
/// rows 0..h = Y, rows h.. = interleaved U V of 2x2 blocks. Row 0 is the top of the image
/// (glReadPixels reads it first), the source texture has row 0 at the bottom.
const NV12_FS: &str = "#version 330
out vec4 o;
uniform sampler2D src;
uniform ivec2 size;
const vec3 K = vec3(0.2126, 0.7152, 0.0722);
vec3 px(ivec2 p) { return texelFetch(src, ivec2(p.x, size.y - 1 - p.y), 0).rgb; }
void main() {
    ivec2 f = ivec2(gl_FragCoord.xy);
    if (f.y < size.y) {
        o = vec4(vec3((16.0 + 219.0 * dot(px(f), K)) / 255.0), 1.0);
        return;
    }
    ivec2 p = ivec2((f.x / 2) * 2, (f.y - size.y) * 2);
    vec3 c = (px(p) + px(p + ivec2(1, 0)) + px(p + ivec2(0, 1)) + px(p + ivec2(1, 1))) * 0.25;
    float y = dot(c, K);
    float v = (f.x % 2 == 0) ? (c.b - y) / 1.8556 : (c.r - y) / 1.5748;
    o = vec4(vec3((128.0 + 224.0 * v) / 255.0), 1.0); // alpha 1: blending may still be on
}
";

pub struct Builtins {
    pub quad: Quad,
    pub blit: Program,
    pub mix: Program,
    pub master: Program,
    pub nv12: Program,
}

impl Builtins {
    pub fn new(gl: &Gl) -> Builtins {
        Builtins {
            quad: Quad::new(gl),
            blit: Program::new(gl, VS, BLIT_FS).expect("blit shader"),
            mix: Program::new(gl, VS, MIX_FS).expect("mix shader"),
            master: Program::new(gl, VS, MASTER_FS).expect("master shader"),
            nv12: Program::new(gl, VS, NV12_FS).expect("nv12 shader"),
        }
    }

    /// src -> dst, optional vertical flip, keeps aspect (letterbox) of src_w/src_h in dst.
    pub fn blit(&mut self, gl: &Gl, src: glow::Texture, src_w: i32, src_h: i32, dst: &Target, flip: bool) {
        dst.bind(gl);
        let src_aspect = src_w.max(1) as f32 / src_h.max(1) as f32;
        let dst_aspect = dst.w as f32 / dst.h as f32;
        let scale = if src_aspect > dst_aspect {
            [1.0, src_aspect / dst_aspect]
        } else {
            [dst_aspect / src_aspect, 1.0]
        };
        self.blit.use_prog(gl);
        self.blit.tex(gl, "src", 0, Some(src));
        self.blit.i1(gl, "flip", flip as i32);
        self.blit.f2(gl, "scale", scale[0], scale[1]);
        self.quad.draw(gl);
    }

    pub fn mix(&mut self, gl: &Gl, bottom: glow::Texture, top: glow::Texture, mix: &crate::state::Mix, dst: &Target) {
        dst.bind(gl);
        self.mix.use_prog(gl);
        self.mix.tex(gl, "bottom", 0, Some(bottom));
        self.mix.tex(gl, "top", 1, Some(top));
        self.mix.i1(gl, "mixType", mix.mix_type.shader_id());
        self.mix.f1(gl, "amount", mix.amount);
        self.mix.f3(gl, "rgb", mix.rgb.map(|b| if b { 1.0 } else { 0.0 }));
        self.quad.draw(gl);
    }

    /// `src` (w x h RGBA) -> `dst` (R8, w x h*3/2) as NV12.
    pub fn nv12(&mut self, gl: &Gl, src: glow::Texture, w: i32, h: i32, dst: &Target) {
        dst.bind(gl);
        self.nv12.use_prog(gl);
        self.nv12.tex(gl, "src", 0, Some(src));
        if let Some(l) = self.nv12.loc(gl, "size") {
            unsafe { gl.uniform_2_i32(Some(&l), w, h) };
        }
        self.quad.draw(gl);
    }

    pub fn master(&mut self, gl: &Gl, src: glow::Texture, brightness: f32, dst: &Target) {
        dst.bind(gl);
        self.master.use_prog(gl);
        self.master.tex(gl, "src", 0, Some(src));
        self.master.f1(gl, "brightness", brightness);
        self.quad.draw(gl);
    }
}

/// Reset GL state so egui's painter starts clean.
pub fn reset_state(gl: &Gl) {
    unsafe {
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        gl.use_program(None);
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, None);
        gl.disable(glow::BLEND);
        gl.disable(glow::SCISSOR_TEST);
    }
}
