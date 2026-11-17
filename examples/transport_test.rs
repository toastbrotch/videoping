#![allow(dead_code)]
#[path = "../src/state.rs"]
mod state;
#[path = "../src/video.rs"]
mod video;
use state::*;
fn run(name: &str, clip: &str, t: &ClipTransport, secs: f32) {
    let mut p = video::Player::new(std::path::Path::new(clip)).unwrap();
    let mut rng = Rng::new();
    let start = std::time::Instant::now();
    let mut log = vec![];
    let mut frames = 0;
    while start.elapsed().as_secs_f32() < secs {
        p.update(t, 120.0, false, &mut rng);
        if p.take_frame(|_| ()).is_some() { frames += 1; }
        let e = start.elapsed().as_secs_f32();
        if log.last().map_or(true, |(le, _): &(f32, f64)| e - le > 0.5) { log.push((e, p.position)); }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    let s: Vec<String> = log.iter().map(|(e, pos)| format!("{e:.1}s:{pos:.2}")).collect();
    println!("{name:10} frames={frames:3} err={:?} | {}", p.error, s.join(" "));
}
fn main() {
    video::init().unwrap();
    let clip = std::env::args().nth(1).unwrap();
    let mut t = ClipTransport::default();
    run("forward", &clip, &t, 3.0);
    t.mode = PlayMode::Backward;
    run("backward", &clip, &t, 3.0);
    t.mode = PlayMode::Forward; t.loop_mode = LoopMode::Bounce; t.in_pt = 0.25; t.out_pt = 0.5; t.speed.free = 2.0;
    run("bounce2x", &clip, &t, 4.0);
    t.loop_mode = LoopMode::Loop; t.speed.bpm_mode = true; t.speed.beats = 4.0; // 2 s for the 2 s range -> rate 1.0
    run("bpm4beats", &clip, &t, 4.0);
    once_then(&clip, LoopMode::Loop);
    once_then(&clip, LoopMode::Bounce);
}

/// play "once" to the end (stops), then switch to `next`: playback must continue
fn once_then(clip: &str, next: LoopMode) {
    let mut p = video::Player::new(std::path::Path::new(clip)).unwrap();
    let mut rng = Rng::new();
    let mut t = ClipTransport { in_pt: 0.5, out_pt: 0.75, loop_mode: LoopMode::Once, ..Default::default() };
    t.speed.free = 2.0;
    let tick = |p: &mut video::Player, t: &ClipTransport, secs: f32, rng: &mut Rng| {
        let s = std::time::Instant::now();
        while s.elapsed().as_secs_f32() < secs {
            p.update(t, 120.0, false, rng);
            p.take_frame(|_| ());
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
        p.position
    };
    let a = tick(&mut p, &t, 2.5, &mut rng); // range 4..6 s at 2x -> done after ~1 s, then held
    let b = tick(&mut p, &t, 0.5, &mut rng);
    t.loop_mode = next;
    let c = tick(&mut p, &t, 0.6, &mut rng);
    println!("once->{:<6} held at {a:.2}/{b:.2} (should stay ~6.0), after switch {c:.2} (should have moved)", next.label());
}
