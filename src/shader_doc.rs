//! Shader documentation: a markdown file next to the shader (`chaos-feedback.fs` +
//! `chaos-feedback.md`). Read-only; re-read when the file changes.
//!
//! What is taken from it (features.md "shader documentation"):
//! - the whole text (long tables as lists), shown rendered in a window ("?" on the effect / generator)
//! - the first paragraph after the `# title`: hover text in the finder
//! - the slider table `| **Label** | range | default | what it does |`: hover text of the
//!   parameter whose isf LABEL (or name) is the bold label
//! - a `> ⚠ ...` line: a grey ⚠ next to the shader name, the warning on hover

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Default)]
pub struct Doc {
    pub title: String,
    /// the first paragraph (plain text)
    pub summary: String,
    /// the text as shown in the window: tables with long text cells as lists (table cells
    /// do not wrap in the viewer)
    pub view: String,
    /// parameter label -> description (plain text, with range / default)
    pub params: HashMap<String, String>,
    /// the warnings (plain text)
    pub warnings: Vec<String>,
}

impl Doc {
    pub fn param(&self, label: &str, name: &str) -> Option<&str> {
        self.params.get(label).or_else(|| self.params.get(name)).map(|s| s.as_str())
    }
    pub fn warning(&self) -> Option<String> {
        (!self.warnings.is_empty()).then(|| self.warnings.join("\n\n"))
    }
}

/// The .md file that belongs to a shader.
pub fn md_path(shader: &Path) -> PathBuf {
    shader.with_extension("md")
}

/// Markdown inline marks away: **bold**, *italic*, `code`, [text](link) -> text.
pub fn plain(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' | '`' => {}
            '_' if out.ends_with(' ') || out.is_empty() || chars.peek().is_none_or(|n| n.is_whitespace() || n.is_ascii_punctuation()) => {}
            '[' => {
                // [text](link) -> text
                let mut text = String::new();
                let mut closed = false;
                for c in chars.by_ref() {
                    if c == ']' {
                        closed = true;
                        break;
                    }
                    text.push(c);
                }
                out.push_str(&text);
                if closed && chars.peek() == Some(&'(') {
                    for c in chars.by_ref() {
                        if c == ')' {
                            break;
                        }
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out.trim().to_string()
}

fn cells(line: &str) -> Vec<String> {
    let t = line.trim().trim_start_matches('|').trim_end_matches('|');
    t.split('|').map(|c| c.trim().to_string()).collect()
}

/// Tables whose rows have a long cell (> 50 characters) become lists:
/// `- **Label** · range · default — what it does` (header names before the middle cells).
pub fn view_text(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<String> = vec![];
    let mut i = 0;
    while i < lines.len() {
        let is_row = |l: &str| l.trim_start().starts_with('|');
        if !is_row(lines[i]) {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        }
        let start = i;
        while i < lines.len() && is_row(lines[i]) {
            i += 1;
        }
        let rows: Vec<Vec<String>> = lines[start..i].iter().map(|l| cells(l)).collect();
        let is_sep = |r: &Vec<String>| r.iter().all(|x| x.chars().all(|c| c == '-' || c == ':' || c == ' '));
        let long = rows.iter().any(|r| r.iter().any(|c| c.chars().count() > 50));
        if !long || rows.len() < 2 || !is_sep(&rows[1]) {
            out.extend(lines[start..i].iter().map(|l| l.to_string()));
            continue;
        }
        let head = &rows[0];
        for r in rows.iter().skip(2) {
            if r.is_empty() {
                continue;
            }
            let first = if r[0].starts_with("**") { r[0].clone() } else { format!("**{}**", r[0]) };
            let mut line = format!("- {first}");
            let n = r.len();
            for (k, c) in r.iter().enumerate().take(n.saturating_sub(1)).skip(1) {
                if c.is_empty() {
                    continue;
                }
                let h = head.get(k).map(|h| plain(h).to_lowercase()).unwrap_or_default();
                line += &if h.is_empty() || h == "range" { format!(" · {c}") } else { format!(" · {h} {c}") };
            }
            if n > 1 {
                line += &format!(" — {}", r[n - 1]);
            }
            out.push(line);
        }
    }
    out.join("\n")
}

pub fn parse(md: &Path, text: &str) -> Doc {
    let _ = md;
    let mut d = Doc { view: view_text(text), ..Default::default() };
    let mut header: Vec<String> = vec![];
    let mut para: Vec<String> = vec![];
    let mut seen_title = false;
    for line in text.lines() {
        let t = line.trim();
        if let Some(h) = t.strip_prefix("# ") {
            if d.title.is_empty() {
                d.title = plain(h);
                seen_title = true;
                continue;
            }
        }
        // the first paragraph after the title
        if d.summary.is_empty() && (seen_title || d.title.is_empty()) {
            let block = t.is_empty() || t.starts_with('#') || t.starts_with('|') || t.starts_with('>') || t.starts_with("```");
            if block {
                if !para.is_empty() {
                    d.summary = plain(&para.join(" "));
                }
            } else {
                para.push(t.to_string());
            }
        }
        if let Some(q) = t.strip_prefix('>') {
            if q.contains('⚠') {
                d.warnings.push(plain(q.replace("⚠️", "").replace('⚠', "").trim()));
            }
            continue;
        }
        if t.starts_with('|') {
            let c = cells(t);
            if c.iter().all(|x| x.chars().all(|ch| ch == '-' || ch == ':' || ch == ' ')) {
                continue; // |---|---|
            }
            // a parameter row: the first cell bold
            let first = &c[0];
            if first.starts_with("**") && first.ends_with("**") && first.len() > 4 && c.len() >= 2 {
                let label = first[2..first.len() - 2].trim().to_string();
                let desc = plain(c.last().unwrap());
                // the cells between: range, default ... with their column names
                let extra: Vec<String> = c[1..c.len() - 1]
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| !v.is_empty())
                    .map(|(i, v)| match header.get(i + 1) {
                        Some(h) if !h.is_empty() => format!("{}: {}", plain(h).to_lowercase(), plain(v)),
                        _ => plain(v),
                    })
                    .collect();
                let full = if extra.is_empty() { desc } else { format!("{desc}\n({})", extra.join(", ")) };
                d.params.entry(label).or_insert(full);
            } else {
                header = c;
            }
        } else if !t.is_empty() {
            header.clear();
        }
    }
    if d.summary.is_empty() && !para.is_empty() {
        d.summary = plain(&para.join(" "));
    }
    d
}

struct Entry {
    checked: Instant,
    mtime: Option<SystemTime>,
    doc: Option<Rc<Doc>>,
}

/// The docs of all shaders seen, re-read when the .md changes (checked at most once a second).
#[derive(Default)]
pub struct Docs {
    map: HashMap<PathBuf, Entry>,
}

impl Docs {
    pub fn get(&mut self, shader: &Path) -> Option<Rc<Doc>> {
        let now = Instant::now();
        if let Some(e) = self.map.get(shader) {
            if now.duration_since(e.checked) < Duration::from_secs(1) {
                return e.doc.clone();
            }
        }
        let md = md_path(shader);
        let mtime = std::fs::metadata(&md).and_then(|m| m.modified()).ok();
        if let Some(e) = self.map.get_mut(shader) {
            e.checked = now;
            if e.mtime == mtime {
                return e.doc.clone();
            }
        }
        let doc = mtime.and_then(|_| std::fs::read_to_string(&md).ok()).map(|t| Rc::new(parse(&md, &t)));
        self.map.insert(shader.to_path_buf(), Entry { checked: now, mtime, doc: doc.clone() });
        doc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_doc() {
        let md = "# Chaos Feedback\n\n`chaos.fs` is an ISF effect: a **black-and-white** loop, see [videoskillet](https://x.y) (MIT).\nSecond line.\n\n1. list\n\n> ⚠️ **Flashing:** with *Haerte* high it flips.\n\n## Sliders\n\n| Slider | Range | Default | What it does |\n|---|---|---|---|\n| **Feedback-Gain (<0 = invers)** | -2 – 2 | -1.05 | Gain of the *loop*. |\n| **Reset** | on/off | off | Empties it. |\n\n| Value | Name |\n|---|---|\n| 1 | Mono |\n";
        let d = parse(Path::new("a.md"), md);
        assert_eq!(d.title, "Chaos Feedback");
        assert_eq!(d.summary, "chaos.fs is an ISF effect: a black-and-white loop, see videoskillet (MIT). Second line.");
        assert_eq!(d.param("Feedback-Gain (<0 = invers)", "x"), Some("Gain of the loop.\n(range: -2 – 2, default: -1.05)"));
        assert_eq!(d.param("nope", "Reset"), Some("Empties it.\n(range: on/off, default: off)"));
        assert_eq!(d.params.len(), 2);
        assert_eq!(d.warning().as_deref(), Some("Flashing: with Haerte high it flips."));
        // the slider table (long cells) as a list, the short table stays
        let v = view_text("| Slider | Range | Default | What it does |\n|---|---|---|---|\n| **Gain** | -2 – 2 | -1.05 | Gain of the loop, negative inverts every generation. |\n\n| Value | Name |\n|---|---|\n| 1 | Mono |");
        assert_eq!(v, "- **Gain** · -2 – 2 · default -1.05 — Gain of the loop, negative inverts every generation.\n\n| Value | Name |\n|---|---|\n| 1 | Mono |");
    }

    #[test]
    fn plain_keeps_snake_case() {
        assert_eq!(plain("uses my_param and _x_"), "uses my_param and x");
    }
}
