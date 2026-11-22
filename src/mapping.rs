//! Projection mapping (features.md "mapping"): slices (an input rectangle of the composition on
//! a corner-pinned quad of the output) and masks (polygons that black out the output), in named
//! presets stored next to the config (a mapping belongs to a venue, not to a set).
//!
//! Coordinates are normalized 0..1 with the origin top left, independent of the resolution.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

pub const MAX_MASK_POINTS: usize = 64;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slice {
    pub name: String,
    /// the part of the composition: x, y, w, h
    pub input: [f32; 4],
    /// where it goes on the output: corners top left, top right, bottom right, bottom left
    pub output: [[f32; 2]; 4],
}

impl Slice {
    pub fn full(name: &str) -> Slice {
        Slice { name: name.into(), input: [0.0, 0.0, 1.0, 1.0], output: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mask {
    pub name: String,
    /// polygon on the output
    pub points: Vec<[f32; 2]>,
    /// soft edge in pixels of the output
    #[serde(default)]
    pub feather: f32,
    /// keep only the inside (instead of blacking it out)
    #[serde(default)]
    pub invert: bool,
}

impl Mask {
    pub fn new(name: &str) -> Mask {
        Mask { name: name.into(), points: vec![[0.4, 0.4], [0.6, 0.4], [0.6, 0.6], [0.4, 0.6]], feather: 0.0, invert: false }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mapping {
    pub name: String,
    #[serde(default)]
    pub slices: Vec<Slice>,
    #[serde(default)]
    pub masks: Vec<Mask>,
}

impl Mapping {
    pub fn new(name: &str) -> Mapping {
        Mapping { name: name.into(), slices: vec![Slice::full("slice 1")], masks: vec![] }
    }
}

/// Row-major 3x3 matrix that maps the unit square (u, v) onto the quad (TL, TR, BR, BL):
/// (x, y) = (M * (u, v, 1)).xy / z.
pub fn square_to_quad(q: &[[f32; 2]; 4]) -> [f32; 9] {
    let [x0, y0] = q[0].map(|v| v as f64);
    let [x1, y1] = q[1].map(|v| v as f64);
    let [x2, y2] = q[2].map(|v| v as f64);
    let [x3, y3] = q[3].map(|v| v as f64);
    let sx = x0 - x1 + x2 - x3;
    let sy = y0 - y1 + y2 - y3;
    let (g, h) = if sx.abs() < 1e-12 && sy.abs() < 1e-12 {
        (0.0, 0.0)
    } else {
        let (dx1, dx2, dy1, dy2) = (x1 - x2, x3 - x2, y1 - y2, y3 - y2);
        let den = dx1 * dy2 - dx2 * dy1;
        if den.abs() < 1e-12 {
            (0.0, 0.0)
        } else {
            ((sx * dy2 - dx2 * sy) / den, (dx1 * sy - sx * dy1) / den)
        }
    };
    let m = [x1 - x0 + g * x1, x3 - x0 + h * x3, x0, y1 - y0 + g * y1, y3 - y0 + h * y3, y0, g, h, 1.0];
    m.map(|v| v as f32)
}

/// Inverse of a row-major 3x3 matrix (None if singular).
pub fn invert3(m: &[f32; 9]) -> Option<[f32; 9]> {
    let m: Vec<f64> = m.iter().map(|v| *v as f64).collect();
    let c = [
        m[4] * m[8] - m[5] * m[7],
        m[2] * m[7] - m[1] * m[8],
        m[1] * m[5] - m[2] * m[4],
        m[5] * m[6] - m[3] * m[8],
        m[0] * m[8] - m[2] * m[6],
        m[2] * m[3] - m[0] * m[5],
        m[3] * m[7] - m[4] * m[6],
        m[1] * m[6] - m[0] * m[7],
        m[0] * m[4] - m[1] * m[3],
    ];
    let det = m[0] * c[0] + m[1] * c[3] + m[2] * c[6];
    if det.abs() < 1e-12 {
        return None;
    }
    Some(c.map(|v| (v / det) as f32))
}

#[cfg(test)]
pub fn apply(m: &[f32; 9], p: [f32; 2]) -> [f32; 2] {
    let z = m[6] * p[0] + m[7] * p[1] + m[8];
    [(m[0] * p[0] + m[1] * p[1] + m[2]) / z, (m[3] * p[0] + m[4] * p[1] + m[5]) / z]
}

/// The mapping presets on disk, saved by themselves after a change.
pub struct Presets {
    dir: PathBuf,
    pub list: Vec<Mapping>,
    changed_at: Option<Instant>,
    /// file names of the presets as loaded (a rename / delete removes the old file)
    on_disk: Vec<String>,
}

fn file_name(name: &str) -> String {
    let safe: String = name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' }).collect();
    format!("{}.json", safe.trim())
}

impl Presets {
    pub fn dir() -> PathBuf {
        // the self-test keeps its own presets (never the user's)
        if let Ok(d) = std::env::var("VIDEOPING_SELFTEST") {
            return PathBuf::from(d).join("mapping");
        }
        dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("videoping").join("mapping")
    }

    pub fn load() -> Presets {
        let dir = Self::dir();
        let mut list: Vec<Mapping> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .filter_map(|e| std::fs::read_to_string(e.path()).ok().and_then(|t| serde_json::from_str(&t).ok()))
            .collect();
        list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        let on_disk = list.iter().map(|m| file_name(&m.name)).collect();
        Presets { dir, list, changed_at: None, on_disk }
    }

    pub fn get(&self, name: &str) -> Option<&Mapping> {
        self.list.iter().find(|m| m.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Mapping> {
        self.changed_at = Some(Instant::now());
        self.list.iter_mut().find(|m| m.name == name)
    }

    pub fn touch(&mut self) {
        self.changed_at = Some(Instant::now());
    }

    /// A free name like "mapping 2".
    pub fn free_name(&self, base: &str) -> String {
        (1..).map(|n| if n == 1 && !base.ends_with(' ') { base.to_string() } else { format!("{base} {n}") }).find(|n| self.get(n).is_none()).unwrap()
    }

    pub fn add(&mut self, m: Mapping) -> String {
        let name = m.name.clone();
        self.list.push(m);
        self.list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        self.touch();
        name
    }

    pub fn rename(&mut self, old: &str, new: &str) -> Result<(), String> {
        let new = new.trim();
        if new.is_empty() {
            return Err("empty name".into());
        }
        if new != old && self.get(new).is_some() {
            return Err(format!("\"{new}\" exists already"));
        }
        if let Some(m) = self.list.iter_mut().find(|m| m.name == old) {
            m.name = new.to_string();
        }
        self.touch();
        Ok(())
    }

    pub fn delete(&mut self, name: &str) {
        self.list.retain(|m| m.name != name);
        self.touch();
    }

    /// Save 0.5 s after the last change (and on quit: flush).
    pub fn poll(&mut self) {
        if self.changed_at.is_some_and(|t| t.elapsed().as_millis() >= 500) {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        if self.changed_at.take().is_none() {
            return;
        }
        let _ = std::fs::create_dir_all(&self.dir);
        let now: Vec<String> = self.list.iter().map(|m| file_name(&m.name)).collect();
        for old in &self.on_disk {
            if !now.contains(old) {
                let _ = std::fs::remove_file(self.dir.join(old));
            }
        }
        for (m, f) in self.list.iter().zip(&now) {
            if let Ok(t) = serde_json::to_string_pretty(m) {
                let _ = std::fs::write(self.dir.join(f), t);
            }
        }
        self.on_disk = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_pin() {
        let q = [[0.1, 0.2], [0.9, 0.1], [0.8, 0.95], [0.15, 0.8]];
        let m = square_to_quad(&q);
        for (uv, want) in [([0.0, 0.0], q[0]), ([1.0, 0.0], q[1]), ([1.0, 1.0], q[2]), ([0.0, 1.0], q[3])] {
            let p = apply(&m, uv);
            assert!((p[0] - want[0]).abs() < 1e-4 && (p[1] - want[1]).abs() < 1e-4, "{uv:?} -> {p:?}, want {want:?}");
        }
        // back: output point -> slice-local (u, v)
        let inv = invert3(&m).unwrap();
        let uv = apply(&inv, apply(&m, [0.3, 0.7]));
        assert!((uv[0] - 0.3).abs() < 1e-4 && (uv[1] - 0.7).abs() < 1e-4);
        // the full square is the identity
        let id = square_to_quad(&Slice::full("x").output);
        assert_eq!(apply(&id, [0.25, 0.5]), [0.25, 0.5]);
    }

    #[test]
    fn preset_names() {
        let mut p = Presets { dir: PathBuf::from("/nonexistent"), list: vec![], changed_at: None, on_disk: vec![] };
        let a = p.free_name("mapping");
        p.add(Mapping::new(&a));
        assert_eq!(a, "mapping");
        assert_eq!(p.free_name("mapping"), "mapping 2");
        assert!(p.rename("mapping", "club").is_ok());
        p.add(Mapping::new("stage"));
        assert!(p.rename("club", "stage").is_err());
        assert_eq!(file_name("a/b:c"), "a_b_c.json");
    }
}
