#![allow(dead_code)]
//! NDI loopback: send test frames, discover the source, receive it.
//! cargo run --example ndi_loop_test -- [/path/to/ndi/lib]
#[path = "../src/ndi.rs"]
mod ndi;
use std::time::{Duration, Instant};
fn main() {
    let dir = std::env::args().nth(1).unwrap_or_default();
    let sender = ndi::NdiSender::new("videoping-loop", 30, &dir).expect("ndi sender");
    let lib = ndi::lib(&dir).unwrap();
    println!("lib: {}", lib.path);
    let finder = ndi::NdiFinder::new(lib.clone());
    let start = Instant::now();
    let mut rx: Option<ndi::NdiReceiver> = None;
    let mut got = 0;
    let mut checked = false;
    while start.elapsed() < Duration::from_secs(12) {
        // send a 320x180 frame: left half red, right half = frame counter in green
        let (w, h) = (320, 180);
        let mut b = sender.buffer((w * h * 4) as usize);
        for y in 0..h { for x in 0..w { let i = ((y * w + x) * 4) as usize;
            b[i..i + 4].copy_from_slice(&if x < w / 2 { [255, 0, 0, 255] } else { [0, 200, 0, 255] }); } }
        sender.send(ndi::FrameMsg { w, h, data: b });
        if rx.is_none() {
            if let Some(src) = finder.sources().into_iter().find(|s| s.contains("videoping-loop")) {
                println!("found source {src:?} after {:.1}s", start.elapsed().as_secs_f32());
                rx = Some(ndi::NdiReceiver::new(lib.clone(), &src).unwrap());
            }
        }
        if let Some(r) = rx.as_mut() {
            if let Some((fw, fh, px_l, px_r)) = r.take_frame(|f| {
                let l = &f.data[(f.h as usize / 2) * f.stride as usize + 10 * 4..][..4];
                let rr = &f.data[(f.h as usize / 2) * f.stride as usize + (f.w as usize - 10) * 4..][..4];
                (f.w, f.h, [l[0], l[1], l[2], l[3]], [rr[0], rr[1], rr[2], rr[3]])
            }) {
                got += 1;
                if !checked { checked = true; println!("first frame {fw}x{fh} left px {px_l:?} right px {px_r:?}"); }
            }
            if got >= 30 { break; }
        }
        std::thread::sleep(Duration::from_millis(33));
    }
    println!("received {got} frames, status {:?}", rx.as_ref().and_then(|r| r.status()));
    assert!(got >= 30, "loopback failed");
}
