//! ISF (Interactive Shader Format) loading.
//!
//! The .fs files are used unchanged on disk. At load time the source is wrapped
//! for an OpenGL 3.3 core context: uniforms for all inputs plus the standard ISF
//! uniforms are declared, the IMG_* macros are defined and the GLSL 1.x keywords
//! (gl_FragColor, varying, texture2D) are mapped to their 3.30 equivalents.

use crate::state::{AnimParam, Param, ParamValue};
use serde_json::Value;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub enum InputKind {
    Float { min: f32, max: f32, default: f32 },
    Long { min: f32, max: f32, default: f32, labels: Vec<(i32, String)> },
    Bool { default: bool },
    Event,
    Color { default: [f32; 4] },
    Point2D { min: [f32; 2], max: [f32; 2], default: [f32; 2] },
    Image,
    Audio,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub name: String,
    pub label: String,
    pub kind: InputKind,
}

#[derive(Clone, Debug)]
pub struct Pass {
    pub target: Option<String>,
}

#[derive(Clone, Debug)]
pub struct IsfInfo {
    pub name: String,
    pub inputs: Vec<Input>,
    pub passes: Vec<Pass>,
    pub is_effect: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FileKind {
    Clip,
    Generator,
    Effect,
    /// NDI source in the finder (the "path" carries the source name)
    Ndi,
}

pub fn display_name(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
}

/// Split an ISF file into its JSON header and the GLSL body.
fn split(src: &str) -> Result<(Value, String), String> {
    let start = src.find("/*").ok_or("no ISF header (/* ... */)")?;
    let end = src[start..].find("*/").ok_or("ISF header not closed")? + start;
    let json = &src[start + 2..end];
    let v: Value = serde_json::from_str(json).map_err(|e| format!("ISF header json: {e}"))?;
    let body = format!("{}{}", &src[..start], &src[end + 2..]);
    Ok((v, body))
}

fn f(v: Option<&Value>, d: f32) -> f32 {
    v.and_then(|x| x.as_f64().or_else(|| x.as_bool().map(|b| if b { 1.0 } else { 0.0 })))
        .map(|x| x as f32)
        .unwrap_or(d)
}

fn arr<const N: usize>(v: Option<&Value>, d: [f32; N]) -> [f32; N] {
    let mut out = d;
    if let Some(a) = v.and_then(|x| x.as_array()) {
        for (i, x) in a.iter().take(N).enumerate() {
            if let Some(n) = x.as_f64() {
                out[i] = n as f32;
            }
        }
    }
    out
}

pub fn parse_info(path: &Path) -> Result<(IsfInfo, String), String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (h, body) = split(&src)?;
    let mut inputs = vec![];
    if let Some(list) = h.get("INPUTS").and_then(|v| v.as_array()) {
        for i in list {
            let name = i.get("NAME").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if name.is_empty() {
                continue;
            }
            let label = i.get("LABEL").and_then(|v| v.as_str()).unwrap_or(&name).to_string();
            let ty = i.get("TYPE").and_then(|v| v.as_str()).unwrap_or("float");
            let kind = match ty {
                "float" => {
                    let min = f(i.get("MIN"), 0.0);
                    let max = f(i.get("MAX"), 1.0);
                    InputKind::Float { min, max, default: f(i.get("DEFAULT"), min) }
                }
                "long" => {
                    let values: Vec<i32> = i
                        .get("VALUES")
                        .and_then(|v| v.as_array())
                        .map(|a| a.iter().filter_map(|x| x.as_f64()).map(|x| x as i32).collect())
                        .unwrap_or_default();
                    let labels: Vec<String> = i
                        .get("LABELS")
                        .and_then(|v| v.as_array())
                        .map(|a| a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect())
                        .unwrap_or_default();
                    let pairs: Vec<(i32, String)> = values
                        .iter()
                        .enumerate()
                        .map(|(k, v)| (*v, labels.get(k).cloned().unwrap_or_else(|| v.to_string())))
                        .collect();
                    let (min, max) = if pairs.is_empty() {
                        (f(i.get("MIN"), 0.0), f(i.get("MAX"), 1.0))
                    } else {
                        (
                            pairs.iter().map(|p| p.0).min().unwrap_or(0) as f32,
                            pairs.iter().map(|p| p.0).max().unwrap_or(0) as f32,
                        )
                    };
                    InputKind::Long { min, max, default: f(i.get("DEFAULT"), min), labels: pairs }
                }
                "bool" => InputKind::Bool { default: f(i.get("DEFAULT"), 0.0) > 0.5 },
                "event" => InputKind::Event,
                "color" => InputKind::Color { default: arr(i.get("DEFAULT"), [1.0, 1.0, 1.0, 1.0]) },
                "point2D" => {
                    let min = arr(i.get("MIN"), [0.0, 0.0]);
                    let max = arr(i.get("MAX"), [1.0, 1.0]);
                    InputKind::Point2D { min, max, default: arr(i.get("DEFAULT"), [0.5, 0.5]) }
                }
                "image" => InputKind::Image,
                "audio" | "audioFFT" => InputKind::Audio,
                _ => continue,
            };
            inputs.push(Input { name, label, kind });
        }
    }
    let mut passes = vec![];
    if let Some(list) = h.get("PASSES").and_then(|v| v.as_array()) {
        for p in list {
            // all pass targets are kept between frames (covers PERSISTENT buffers)
            passes.push(Pass { target: p.get("TARGET").and_then(|v| v.as_str()).map(|s| s.to_string()) });
        }
    }
    if passes.is_empty() {
        passes.push(Pass { target: None });
    }
    let is_effect = inputs.iter().any(|i| i.name == "inputImage" && i.kind == InputKind::Image);
    let info = IsfInfo {
        name: display_name(path),
        inputs,
        passes,
        is_effect,
    };
    Ok((info, body))
}

pub fn classify(path: &Path) -> Option<FileKind> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    match ext.as_str() {
        "mp4" | "mov" | "m4v" | "mkv" | "webm" | "avi" => Some(FileKind::Clip),
        "fs" => {
            let (info, _) = parse_info(path).ok()?;
            Some(if info.is_effect { FileKind::Effect } else { FileKind::Generator })
        }
        _ => None,
    }
}

/// Default parameter values for a shader.
pub fn default_params(info: &IsfInfo) -> Vec<Param> {
    info.inputs
        .iter()
        .filter_map(|i| {
            let value = match &i.kind {
                InputKind::Float { min, max, default } => ParamValue::Anim(vec![AnimParam::new(*default, *min, *max)]),
                InputKind::Long { min, max, default, .. } => {
                    ParamValue::Anim(vec![AnimParam::new(*default, *min, *max)])
                }
                InputKind::Bool { default } => ParamValue::Bool(*default),
                InputKind::Event => ParamValue::Bool(false),
                InputKind::Color { default } => ParamValue::Color(*default),
                InputKind::Point2D { min, max, default } => ParamValue::Anim(vec![
                    AnimParam::new(default[0], min[0], max[0]),
                    AnimParam::new(default[1], min[1], max[1]),
                ]),
                InputKind::Image | InputKind::Audio => return None,
            };
            Some(Param { name: i.name.clone(), value })
        })
        .collect()
}

/// Make sure saved params match the shader's inputs (shader may have changed on disk).
pub fn reconcile_params(info: &IsfInfo, params: &mut Vec<Param>) {
    let defaults = default_params(info);
    let mut out = Vec::with_capacity(defaults.len());
    for d in defaults {
        match params.iter().find(|p| p.name == d.name) {
            Some(p) if std::mem::discriminant(&p.value) == std::mem::discriminant(&d.value) => out.push(p.clone()),
            _ => out.push(d),
        }
    }
    *params = out;
}

fn replace_word(src: &str, word: &str, with: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let is_ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    while i < src.len() {
        if src[i..].starts_with(word)
            && (i == 0 || !is_ident(bytes[i - 1]))
            && (i + word.len() >= src.len() || !is_ident(bytes[i + word.len()]))
        {
            out.push_str(with);
            i += word.len();
        } else {
            let ch = src[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// Remove a leading `#version` line from user code (we set our own).
fn strip_version(src: &str) -> String {
    src.lines()
        .map(|l| if l.trim_start().starts_with("#version") { "" } else { l })
        .collect::<Vec<_>>()
        .join("\n")
}

fn uniform_decls(info: &IsfInfo) -> String {
    let mut s = String::new();
    s.push_str(
        "uniform float TIME;\nuniform float TIMEDELTA;\nuniform vec2 RENDERSIZE;\nuniform int PASSINDEX;\n\
         uniform int FRAMEINDEX;\nuniform vec4 DATE;\nuniform float uInstanceSeed;\n",
    );
    for i in &info.inputs {
        let ty = match i.kind {
            InputKind::Float { .. } => "float",
            InputKind::Long { .. } => "int",
            InputKind::Bool { .. } | InputKind::Event => "bool",
            InputKind::Color { .. } => "vec4",
            InputKind::Point2D { .. } => "vec2",
            InputKind::Image | InputKind::Audio => "sampler2D",
        };
        s.push_str(&format!("uniform {ty} {};\n", i.name));
    }
    for p in &info.passes {
        if let Some(t) = &p.target {
            if !info.inputs.iter().any(|i| &i.name == t) {
                s.push_str(&format!("uniform sampler2D {t};\n"));
            }
        }
    }
    s
}

const MACROS: &str = "\
#define texture2D texture\n\
#define texture2DRect texture\n\
#define IMG_NORM_PIXEL(i, c) texture(i, c)\n\
#define IMG_PIXEL(i, c) texture(i, (c) / vec2(textureSize(i, 0)))\n\
#define IMG_THIS_PIXEL(i) texture(i, isf_FragNormCoord)\n\
#define IMG_THIS_NORM_PIXEL(i) texture(i, isf_FragNormCoord)\n\
#define IMG_SIZE(i) vec2(textureSize(i, 0))\n\
#define IMG_NORM_PIXEL_LOD(i, c, l) textureLod(i, c, l)\n\
#define IMG_PIXEL_LOD(i, c, l) textureLod(i, (c) / vec2(textureSize(i, 0)), l)\n";

pub fn build_fragment(info: &IsfInfo, body: &str) -> String {
    let mut code = strip_version(body);
    code = replace_word(&code, "gl_FragColor", "isf_FragColor");
    code = code.replace("gl_FragData[0]", "isf_FragColor");
    code = replace_word(&code, "varying", "in");
    format!(
        "#version 330\nout vec4 isf_FragColor;\nin vec2 isf_FragNormCoord;\n{}{}#line 1\n{}",
        uniform_decls(info),
        MACROS,
        code
    )
}

const DEFAULT_VS_MAIN: &str = "void main() { isf_vertShaderInit(); }\n";

pub fn build_vertex(info: &IsfInfo, custom: Option<&str>) -> String {
    let user = match custom {
        Some(src) => {
            let mut c = strip_version(src);
            c = replace_word(&c, "varying", "out");
            c = replace_word(&c, "attribute", "in");
            c
        }
        None => DEFAULT_VS_MAIN.to_string(),
    };
    format!(
        "#version 330\nin vec2 isf_position;\nout vec2 isf_FragNormCoord;\n{}\
         void isf_vertShaderInit() {{ gl_Position = vec4(isf_position, 0.0, 1.0); isf_FragNormCoord = isf_position * 0.5 + 0.5; }}\n\
         #line 1\n{}",
        uniform_decls(info),
        user
    )
}

/// Load the optional vertex shader that sits next to the .fs file.
pub fn custom_vertex(path: &Path) -> Option<String> {
    let vs = path.with_extension("vs");
    std::fs::read_to_string(vs).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_project_shaders() {
        let (fx, body) = parse_info(Path::new("isf-effect/gameboy-camera.fs")).unwrap();
        assert!(fx.is_effect);
        assert_eq!(fx.inputs.iter().filter(|i| i.kind != InputKind::Image).count(), 10);
        let frag = build_fragment(&fx, &body);
        assert!(frag.contains("isf_FragColor = vec4(col, 1.0)"));
        assert!(!frag.contains("gl_FragColor"));

        let (gen_info, body) = parse_info(Path::new("isf-generator/0-id-display.fs")).unwrap();
        assert!(!gen_info.is_effect);
        let frag = build_fragment(&gen_info, &body);
        assert!(frag.contains("uniform float uInstanceSeed;"));
        assert_eq!(classify(Path::new("isf-generator/0-id-display.fs")), Some(FileKind::Generator));
    }

    #[test]
    fn replace_word_respects_identifiers() {
        assert_eq!(replace_word("varying vec2 myvarying;", "varying", "in"), "in vec2 myvarying;");
    }
}
