//! Undo / redo: one history per tab (see features.md "undo"). A history keeps copies of the
//! tab's state. The app shows it the current state after every finished action (mouse released,
//! a midi knob still for a moment, a text field left); when that differs from the last state it
//! saw, the last state becomes an undo step. Undo puts a copy back; the renderer keeps everything
//! whose id survives (a clip plays on, effects keep their buffers).

use serde_json::Value;

/// How many steps a history keeps (older ones are dropped).
pub const LIMIT: usize = 100;

pub struct History<T: Clone> {
    undo: Vec<(T, String)>,
    redo: Vec<(T, String)>,
    /// the last state seen, its key (hash of what counts), and its comparable form
    cur: Option<(T, u64, Value)>,
}

impl<T: Clone> Default for History<T> {
    fn default() -> Self {
        History { undo: vec![], redo: vec![], cur: None }
    }
}

impl<T: Clone> History<T> {
    /// The current state after an action: a change becomes an undo step, labeled by `label`
    /// (from the comparable forms before / after). Returns the label of a new step.
    pub fn observe(&mut self, now: &T, key: u64, value: impl FnOnce() -> Value, label: impl FnOnce(&Value, &Value) -> String) -> Option<String> {
        match &self.cur {
            None => {
                self.cur = Some((now.clone(), key, value()));
                None
            }
            Some((_, k, _)) if *k == key => None,
            Some(_) => {
                let v = value();
                let (prev, _, pv) = self.cur.take().unwrap();
                let l = label(&pv, &v);
                self.undo.push((prev, l.clone()));
                if self.undo.len() > LIMIT {
                    self.undo.remove(0);
                }
                self.redo.clear();
                self.cur = Some((now.clone(), key, v));
                Some(l)
            }
        }
    }

    /// Take the current state as it is, without an undo step (e.g. after a bpm tap).
    pub fn absorb(&mut self, now: &T, key: u64, value: Value) {
        self.cur = Some((now.clone(), key, value));
    }

    /// The last state seen.
    pub fn current(&self) -> Option<&T> {
        self.cur.as_ref().map(|(t, _, _)| t)
    }

    /// What undo / redo would do (for the buttons' hover).
    pub fn undo_label(&self) -> Option<&str> {
        self.undo.last().map(|(_, l)| l.as_str())
    }

    pub fn redo_label(&self) -> Option<&str> {
        self.redo.last().map(|(_, l)| l.as_str())
    }

    /// Undo: the state to put back and what is undone. `now` (the current state) goes to redo.
    pub fn undo(&mut self, now: &T) -> Option<(T, String)> {
        let (prev, label) = self.undo.pop()?;
        self.redo.push((now.clone(), label.clone()));
        self.cur = None;
        Some((prev, label))
    }

    /// Redo: the state to put back and what is redone. `now` goes back to undo.
    pub fn redo(&mut self, now: &T) -> Option<(T, String)> {
        let (next, label) = self.redo.pop()?;
        self.undo.push((now.clone(), label.clone()));
        self.cur = None;
        Some((next, label))
    }

    /// After putting a state back: that state is the current one (no step).
    pub fn restored(&mut self, now: &T, key: u64, value: Value) {
        self.cur = Some((now.clone(), key, value));
    }
}

/// Hash of a comparable form (its JSON text).
pub fn key_of(v: &Value) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    v.to_string().hash(&mut h);
    h.finish()
}

/// The first place where two comparable forms differ, as a path of keys / indexes.
pub fn first_difference(a: &Value, b: &Value) -> Vec<String> {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            let mut keys: Vec<&String> = x.keys().chain(y.keys()).collect();
            keys.sort();
            keys.dedup();
            for k in keys {
                match (x.get(k), y.get(k)) {
                    (Some(p), Some(q)) if p == q => {}
                    (Some(p), Some(q)) => {
                        let mut r = vec![k.clone()];
                        r.extend(first_difference(p, q));
                        return r;
                    }
                    _ => return vec![k.clone()],
                }
            }
            vec![]
        }
        (Value::Array(x), Value::Array(y)) => {
            for i in 0..x.len().max(y.len()) {
                match (x.get(i), y.get(i)) {
                    (Some(p), Some(q)) if p == q => {}
                    (Some(p), Some(q)) => {
                        let mut r = vec![i.to_string()];
                        r.extend(first_difference(p, q));
                        return r;
                    }
                    _ => return vec![i.to_string()],
                }
            }
            vec![]
        }
        _ => vec![],
    }
}

// ------------------------------------------------------------------ the live tab

/// The live tab's session as it counts for undo: without where you look (page, selected slot /
/// layer) and without what moves by itself (the value of a running animation).
pub fn live_value(s: &crate::state::Session) -> Value {
    let mut v = serde_json::to_value(s).unwrap_or(Value::Null);
    if let Value::Object(m) = &mut v {
        for k in ["page", "selected_instance", "selected_layer", "next_id"] {
            m.remove(k);
        }
    }
    strip_running(&mut v);
    v
}

/// Animated parameters: a running one's value / phase / direction change by themselves.
fn strip_running(v: &mut Value) {
    match v {
        Value::Object(m) => {
            if let Some(Value::Array(chs)) = m.get_mut("Anim") {
                for ch in chs.iter_mut() {
                    if let Value::Object(c) = ch {
                        let running = c.get("mode").and_then(|x| x.as_str()) != Some("Pause");
                        c.remove("phase");
                        c.remove("dir");
                        if running {
                            c.remove("value");
                        }
                    }
                }
            }
            for x in m.values_mut() {
                strip_running(x);
            }
        }
        Value::Array(a) => a.iter_mut().for_each(strip_running),
        _ => {}
    }
}

/// What changed, in words: "slot 9 (testclip) · effect 1 (pixelate) · size" (parameter names as
/// they are saved; `live_label_with` turns them into the sliders' labels).
#[cfg(test)]
pub fn live_label(before: &Value, after: &Value) -> String {
    live_label_with(before, after, &|_, _| None)
}

/// Like live_label; `label(shader path, parameter name)` gives a parameter's slider label.
pub fn live_label_with(before: &Value, after: &Value, label: &dyn Fn(&str, &str) -> Option<String>) -> String {
    let path = first_difference(before, after);
    let at = |v: &Value, n: usize| -> Value {
        let mut x = v.clone();
        for p in path.iter().take(n) {
            x = match &x {
                Value::Object(m) => m.get(p).cloned().unwrap_or(Value::Null),
                Value::Array(a) => p.parse::<usize>().ok().and_then(|i| a.get(i).cloned()).unwrap_or(Value::Null),
                _ => Value::Null,
            };
        }
        x
    };
    // the after state names things (or the before state, when it was removed)
    let named = |n: usize, field: &str| -> Option<String> {
        [after, before].iter().find_map(|v| at(v, n).get(field).and_then(|x| x.as_str()).map(String::from))
    };
    let stem = |n: usize| -> Option<String> {
        [after, before].iter().find_map(|v| {
            at(v, n).get("path").and_then(|x| x.as_str()).map(|p| std::path::Path::new(p).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default())
        })
    };
    let full_path = |n: usize| -> Option<String> { [after, before].iter().find_map(|v| at(v, n).get("path").and_then(|x| x.as_str()).map(String::from)) };
    let mut shader: Option<String> = None;
    let mut parts: Vec<String> = vec![];
    let mut i = 0;
    while i < path.len() {
        let p = path[i].as_str();
        let next = path.get(i + 1).map(|s| s.as_str());
        match (p, next) {
            ("slots", Some(n)) => {
                let k: usize = n.parse().unwrap_or(0);
                parts.push(match named(i + 2, "name") {
                    Some(name) => format!("slot {} ({name})", k + 1),
                    None => format!("slot {}", k + 1),
                });
                i += 2;
            }
            ("layers", Some(n)) => {
                parts.push(format!("layer {}", n.parse::<usize>().unwrap_or(0) + 1));
                i += 2;
            }
            ("effects", Some(n)) | ("output_effects", Some(n)) => {
                let k: usize = n.parse().unwrap_or(0);
                let what = if p == "output_effects" { "output effect" } else { "effect" };
                shader = full_path(i + 2);
                parts.push(match stem(i + 2) {
                    Some(s) => format!("{what} {} ({s})", k + 1),
                    None => format!("{what} {}", k + 1),
                });
                i += 2;
            }
            ("params", Some(_)) => {
                if let Some(n) = named(i + 2, "name") {
                    parts.push(shader.as_deref().and_then(|s| label(s, &n)).unwrap_or(n));
                }
                break;
            }
            ("Generator", _) => {
                shader = full_path(i + 1);
                i += 1;
            }
            ("source", _) | ("Clip", _) | ("Ndi", _) => i += 1,
            ("transport", _) => {
                parts.push("in / out, speed".into());
                break;
            }
            ("mix", _) => {
                parts.push("mix".into());
                break;
            }
            ("master", _) => {
                parts.push("master".into());
                break;
            }
            ("bpm", _) => {
                parts.push("bpm".into());
                break;
            }
            ("output", _) => {
                parts.push("output options".into());
                break;
            }
            ("finder_roots", _) | ("finder_favs", _) => {
                parts.push("finder".into());
                break;
            }
            ("active", _) => {
                parts.push("on / off".into());
                break;
            }
            ("instance", _) => {
                parts.push("loaded".into());
                break;
            }
            _ => break,
        }
    }
    if parts.is_empty() { "a change".into() } else { parts.join(" · ") }
}

// ------------------------------------------------------------------ projection mapping, midi

/// What changed in a mapping preset: "slice 2 (right box)", "mask 1 (booth)".
pub fn map_label(before: &Value, after: &Value) -> String {
    let path = first_difference(before, after);
    let name = |list: &str, i: usize| -> Option<String> {
        [after, before].iter().find_map(|v| v.get(list).and_then(|l| l.get(i)).and_then(|x| x.get("name")).and_then(|n| n.as_str()).map(String::from))
    };
    match (path.first().map(|s| s.as_str()), path.get(1).and_then(|n| n.parse::<usize>().ok())) {
        (Some("slices"), Some(i)) => name("slices", i).map(|n| format!("slice {} ({n})", i + 1)).unwrap_or(format!("slice {}", i + 1)),
        (Some("masks"), Some(i)) => name("masks", i).map(|n| format!("mask {} ({n})", i + 1)).unwrap_or(format!("mask {}", i + 1)),
        _ => "mapping".into(),
    }
}

/// What changed in a midi profile: "binding 3 (/layer/1/mix)", "orientation".
pub fn midi_label(before: &Value, after: &Value) -> String {
    let path = first_difference(before, after);
    match (path.first().map(|s| s.as_str()), path.get(1).and_then(|n| n.parse::<usize>().ok())) {
        (Some("bindings"), Some(i)) => {
            let addr = [after, before].iter().find_map(|v| v.get("bindings").and_then(|l| l.get(i)).and_then(|b| b.get("addr")).and_then(|a| a.as_str()).map(String::from));
            addr.map(|a| format!("binding {} ({a})", i + 1)).unwrap_or(format!("binding {}", i + 1))
        }
        (Some("bindings"), None) => "bindings".into(),
        (Some("grid"), _) => "orientation".into(),
        _ => "profile".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn history_steps_undo_redo() {
        let mut h: History<i32> = History::default();
        let v = |x: i32| json!(x);
        assert_eq!(h.observe(&1, 1, || v(1), |_, _| "x".into()), None, "the first state is no step");
        assert_eq!(h.observe(&1, 1, || v(1), |_, _| "x".into()), None, "unchanged: no step");
        assert_eq!(h.observe(&2, 2, || v(2), |_, _| "to 2".into()).as_deref(), Some("to 2"));
        h.observe(&3, 3, || v(3), |_, _| "to 3".into());
        assert_eq!(h.undo_label(), Some("to 3"));
        let (back, l) = h.undo(&3).unwrap();
        assert_eq!((back, l.as_str()), (2, "to 3"));
        h.restored(&2, 2, v(2));
        assert_eq!(h.redo_label(), Some("to 3"));
        let (fwd, _) = h.redo(&2).unwrap();
        assert_eq!(fwd, 3);
        h.restored(&3, 3, v(3));
        // a new change after an undo drops the redo steps
        h.undo(&3);
        h.restored(&2, 2, v(2));
        h.observe(&5, 5, || v(5), |_, _| "to 5".into());
        assert!(h.redo_label().is_none());
        // absorb: no step
        h.absorb(&6, 6, v(6));
        assert_eq!(h.undo_label(), Some("to 5"));
        // the limit
        let mut h: History<i32> = History::default();
        for i in 0..(LIMIT as i32 + 10) {
            h.observe(&i, i as u64, || v(i), |_, _| "s".into());
        }
        assert_eq!(h.undo.len(), LIMIT);
    }

    #[test]
    fn running_animations_and_navigation_do_not_count() {
        let mut s = crate::state::Session::default();
        let a = key_of(&live_value(&s));
        s.page = 1;
        s.selected_layer = 2;
        assert_eq!(key_of(&live_value(&s)), a, "where you look is no change");
        s.bpm = 128.0;
        assert_ne!(key_of(&live_value(&s)), a, "the bpm is");
        // a running animation's value moves by itself; a paused one's value is a change
        let mut v = json!({ "x": { "Anim": [{ "value": 0.2, "mode": "Forward", "phase": 0.1, "dir": 1.0, "lo": 0.0 }] } });
        strip_running(&mut v);
        assert_eq!(v, json!({ "x": { "Anim": [{ "mode": "Forward", "lo": 0.0 }] } }));
        let mut v = json!({ "x": { "Anim": [{ "value": 0.2, "mode": "Pause", "phase": 0.1, "dir": 1.0 }] } });
        strip_running(&mut v);
        assert_eq!(v, json!({ "x": { "Anim": [{ "value": 0.2, "mode": "Pause" }] } }));
    }

    #[test]
    fn labels_say_what_changed() {
        let before = json!({ "slots": [null, { "name": "testclip", "effects": [{ "path": "/x/pixelate.fs", "params": [{ "name": "size", "value": 1 }] }] }] });
        let after = json!({ "slots": [null, { "name": "testclip", "effects": [{ "path": "/x/pixelate.fs", "params": [{ "name": "size", "value": 2 }] }] }] });
        assert_eq!(live_label(&before, &after), "slot 2 (testclip) · effect 1 (pixelate) · size");
        assert_eq!(live_label(&json!({ "bpm": 120 }), &json!({ "bpm": 128 })), "bpm");
        assert_eq!(live_label(&json!({ "layers": [{ "mix": 1 }] }), &json!({ "layers": [{ "mix": 2 }] })), "layer 1 · mix");
        let before = json!({ "slots": [null, null] });
        let after = json!({ "slots": [null, { "name": "hex-pulse" }] });
        assert_eq!(live_label(&before, &after), "slot 2 (hex-pulse)");
        assert_eq!(live_label(&json!({ "output_effects": [] }), &json!({ "output_effects": [{ "path": "/a/trails.fs" }] })), "output effect 1 (trails)");
        assert_eq!(map_label(&json!({ "slices": [{ "name": "a", "x": 1 }] }), &json!({ "slices": [{ "name": "a", "x": 2 }] })), "slice 1 (a)");
        assert_eq!(midi_label(&json!({ "bindings": [{ "addr": "/bpm/tap", "mode": "Trigger" }] }), &json!({ "bindings": [{ "addr": "/bpm/tap", "mode": "Toggle" }] })), "binding 1 (/bpm/tap)");
    }
}
