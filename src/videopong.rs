//! videopong.net clips in the finder (features.md "videopong"): the public JSON API (no key),
//! search / random, thumbnails, download of the public preview mp4 into a visible cache folder.
//! All network work runs in background threads; the ui polls.

use serde_json::Value;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const API: &str = "https://www.videopong.net/api";
pub const PAGE: usize = 50;

#[derive(Clone, Debug, PartialEq)]
pub struct Clip {
    pub id: String,
    pub title: String,
    pub duration: String,
    pub resolution: String,
    pub license: String,
    pub author: String,
    pub thumb: String,
    pub preview: String,
    pub page: String,
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(60))).http_status_as_error(true).build().into()
}

fn get_json(url: &str) -> Result<Value, String> {
    let mut r = agent().get(url).header("user-agent", concat!("videoping/", env!("CARGO_PKG_VERSION"))).call().map_err(|e| format!("videopong: {e}"))?;
    r.body_mut().read_json::<Value>().map_err(|e| format!("videopong: {e}"))
}

/// Clips of an API answer: data = [[clip, user], ...]; count = all hits.
pub fn parse(v: &Value) -> Result<(Vec<Clip>, usize), String> {
    if let Some(e) = v["status"]["error"].as_i64().filter(|e| *e != 0) {
        return Err(format!("videopong error {e}: {}", v["status"]["errorstring"].as_str().unwrap_or("")));
    }
    let s = |x: &Value| match x {
        Value::String(t) => t.clone(),
        Value::Null => String::new(),
        o => o.to_string(),
    };
    let mut out = vec![];
    for pair in v["data"].as_array().into_iter().flatten() {
        let (c, u) = match pair {
            Value::Array(a) => (a.first().cloned().unwrap_or(Value::Null), a.get(1).cloned().unwrap_or(Value::Null)),
            o => (o.clone(), Value::Null),
        };
        let clip = Clip {
            id: s(&c["id"]),
            title: s(&c["title"]),
            duration: s(&c["duration"]),
            resolution: s(&c["resolution"]),
            license: s(&c["license"]),
            author: s(&u["name"]),
            thumb: s(&c["url_thumb"]),
            preview: s(&c["url_preview_mp4"]),
            page: s(&c["url_videopong"]),
        };
        if !clip.id.is_empty() && !clip.preview.is_empty() {
            out.push(clip);
        }
    }
    let count = v["count"].as_str().and_then(|c| c.parse().ok()).or_else(|| v["count"].as_u64().map(|c| c as usize)).unwrap_or(out.len());
    Ok((out, count))
}

/// The id may only be letters and digits (it goes into file names).
fn safe_id(id: &str) -> String {
    id.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

pub fn clip_dir() -> PathBuf {
    if let Ok(d) = std::env::var("VIDEOPING_SELFTEST") {
        return PathBuf::from(d).join("videopong");
    }
    dirs::video_dir().or_else(|| dirs::home_dir().map(|h| h.join("Videos"))).unwrap_or_else(|| PathBuf::from(".")).join("videopong")
}

fn thumb_dir() -> PathBuf {
    dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("videoping").join("videopong-thumbs")
}

/// The local file of a clip: "<title> [<id>].mp4".
pub fn clip_path(c: &Clip) -> PathBuf {
    let title: String = c.title.chars().map(|ch| if ch.is_alphanumeric() || " -_.,()".contains(ch) { ch } else { '_' }).collect();
    let title = title.trim().chars().take(60).collect::<String>();
    clip_dir().join(format!("{} [{}].mp4", if title.is_empty() { "clip" } else { &title }, safe_id(&c.id)))
}

pub fn is_cached(c: &Clip) -> bool {
    std::fs::metadata(clip_path(c)).is_ok_and(|m| m.len() > 0)
}

/// What to do when a download is done.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum After {
    Preview,
    FirstFree,
    Slot(usize),
}

pub struct Download {
    pub done: Arc<AtomicU64>,
    pub total: Arc<AtomicU64>,
    rx: Receiver<Result<PathBuf, String>>,
    pub after: After,
}

/// What a finished background job reports to the ui.
pub enum Event {
    /// a download is done: the clip, its file (or the error), what to do with it
    Downloaded(Clip, Result<PathBuf, String>, After),
}

#[derive(Default)]
pub struct Videopong {
    pub query: String,
    pub results: Vec<Clip>,
    pub total: usize,
    pub error: Option<String>,
    /// the search that is running: (more = append), receiver
    searching: Option<(bool, Receiver<Result<(Vec<Clip>, usize), String>>)>,
    last_query: Option<String>,
    /// thumbnails: id -> decoded rgba (taken by the ui once and turned into a texture)
    pub thumbs: Arc<Mutex<HashMap<String, Option<(u32, u32, Vec<u8>)>>>>,
    thumb_tx: Option<Sender<Clip>>,
    pub downloads: HashMap<String, Download>,
    clips: HashMap<String, Clip>,
}

impl Videopong {
    pub fn busy(&self) -> bool {
        self.searching.is_some()
    }

    /// Search (empty query: random clips); `more` = the next page of the last search.
    pub fn search(&mut self, more: bool) {
        if self.searching.is_some() {
            return;
        }
        let q = if more { self.last_query.clone().unwrap_or_default() } else { self.query.trim().to_string() };
        let offset = if more { self.results.len() } else { 0 };
        let url = if q.is_empty() {
            format!("{API}/clip/random/{PAGE}")
        } else {
            // the query is one path segment
            let enc: String = q.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect();
            format!("{API}/clip/search/{enc}/{offset}/{PAGE}")
        };
        self.last_query = Some(q);
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let _ = tx.send(get_json(&url).and_then(|v| parse(&v)));
        });
        self.searching = Some((more, rx));
        self.error = None;
    }

    fn thumb_workers(&mut self) -> Sender<Clip> {
        if let Some(t) = &self.thumb_tx {
            return t.clone();
        }
        let (tx, rx) = channel::<Clip>();
        let rx = Arc::new(Mutex::new(rx));
        for _ in 0..4 {
            let rx = rx.clone();
            let thumbs = self.thumbs.clone();
            std::thread::spawn(move || loop {
                let Ok(c) = rx.lock().map_err(|_| ()).and_then(|r| r.recv().map_err(|_| ())) else { return };
                let img = load_thumb(&c);
                if let Ok(mut t) = thumbs.lock() {
                    t.insert(c.id.clone(), img);
                }
            });
        }
        self.thumb_tx = Some(tx.clone());
        tx
    }

    /// Once per frame: finished searches; finished downloads are returned to the app.
    pub fn poll(&mut self) -> Vec<Event> {
        if let Some((more, rx)) = &self.searching {
            if let Ok(res) = rx.try_recv() {
                let more = *more;
                self.searching = None;
                match res {
                    Ok((clips, total)) => {
                        if !more {
                            self.results.clear();
                        }
                        let tx = self.thumb_workers();
                        for c in &clips {
                            if !self.results.iter().any(|r| r.id == c.id) {
                                if let Ok(t) = self.thumbs.lock() {
                                    if !t.contains_key(&c.id) {
                                        let _ = tx.send(c.clone());
                                    }
                                }
                                self.results.push(c.clone());
                            }
                        }
                        self.total = total.max(self.results.len());
                    }
                    Err(e) => self.error = Some(e),
                }
            }
        }
        let mut ev = vec![];
        let done: Vec<String> = self.downloads.iter().filter_map(|(id, d)| d.rx.try_recv().ok().map(|r| (id.clone(), r))).map(|(id, r)| {
            let c = self.clips.get(&id).cloned();
            let after = self.downloads.get(&id).map(|d| d.after).unwrap_or(After::Preview);
            if let Some(c) = c {
                ev.push(Event::Downloaded(c, r, after));
            }
            id
        }).collect();
        for id in done {
            self.downloads.remove(&id);
        }
        ev
    }

    /// The clip's local file: at once when cached (Some), else the download starts (None).
    pub fn fetch(&mut self, c: &Clip, after: After) -> Option<PathBuf> {
        if is_cached(c) {
            return Some(clip_path(c));
        }
        if let Some(d) = self.downloads.get_mut(&c.id) {
            d.after = after;
            return None;
        }
        let done = Arc::new(AtomicU64::new(0));
        let total = Arc::new(AtomicU64::new(0));
        let (tx, rx) = channel();
        let (c2, d2, t2) = (c.clone(), done.clone(), total.clone());
        std::thread::spawn(move || {
            let _ = tx.send(download(&c2, &d2, &t2));
        });
        self.clips.insert(c.id.clone(), c.clone());
        self.downloads.insert(c.id.clone(), Download { done, total, rx, after });
        None
    }
}

fn download(c: &Clip, done: &AtomicU64, total: &AtomicU64) -> Result<PathBuf, String> {
    let path = clip_path(c);
    std::fs::create_dir_all(clip_dir()).map_err(|e| e.to_string())?;
    let part = path.with_extension("mp4.part");
    let mut r = agent().get(&c.preview).header("user-agent", concat!("videoping/", env!("CARGO_PKG_VERSION"))).call().map_err(|e| format!("videopong: {e}"))?;
    if let Some(n) = r.headers().get("content-length").and_then(|v| v.to_str().ok()).and_then(|v| v.parse().ok()) {
        total.store(n, Ordering::Relaxed);
    }
    let mut body = r.body_mut().with_config().limit(2 * 1024 * 1024 * 1024).reader();
    let mut f = std::fs::File::create(&part).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = body.read(&mut buf).map_err(|e| {
            let _ = std::fs::remove_file(&part);
            format!("videopong: {e}")
        })?;
        if n == 0 {
            break;
        }
        f.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        done.fetch_add(n as u64, Ordering::Relaxed);
    }
    drop(f);
    std::fs::rename(&part, &path).map_err(|e| e.to_string())?;
    Ok(path)
}

/// A thumbnail (cached on disk), decoded to rgba.
fn load_thumb(c: &Clip) -> Option<(u32, u32, Vec<u8>)> {
    let dir = thumb_dir();
    let file = dir.join(format!("{}.jpg", safe_id(&c.id)));
    let bytes = match std::fs::read(&file) {
        Ok(b) if !b.is_empty() => b,
        _ => {
            if c.thumb.is_empty() {
                return None;
            }
            let mut r = agent().get(&c.thumb).call().ok()?;
            let b = r.body_mut().read_to_vec().ok()?;
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(&file, &b);
            b
        }
    };
    let img = image::load_from_memory_with_format(&bytes, image::ImageFormat::Jpeg).ok()?.thumbnail(160, 120).to_rgba8();
    Some((img.width(), img.height(), img.into_raw()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_answer() {
        let v: Value = serde_json::from_str(
            r#"{"status":{"error":0,"errorstring":"ok"},"data":[[{"id":"0idc3ejap14","title":"swans/average 01","duration":"20","resolution":"1920x1080","license":"bysa","url_thumb":"https://x/t.jpg","url_preview_mp4":"https://x/m.mp4","url_videopong":"https://x/d"},{"name":"plush"}]],"count":"405"}"#,
        )
        .unwrap();
        let (c, n) = parse(&v).unwrap();
        assert_eq!(n, 405);
        assert_eq!((c[0].id.as_str(), c[0].author.as_str(), c[0].duration.as_str()), ("0idc3ejap14", "plush", "20"));
        assert!(clip_path(&c[0]).ends_with("swans_average 01 [0idc3ejap14].mp4"));
        let e: Value = serde_json::from_str(r#"{"status":{"error":3,"errorstring":"bad"}}"#).unwrap();
        assert!(parse(&e).is_err());
    }
}
