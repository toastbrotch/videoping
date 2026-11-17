#[path = "../src/ndi.rs"]
mod ndi;
fn main() {
    let s = ndi::NdiSender::new("videoping-test", 60).expect("ndi");
    eprintln!("created");
    for i in 0..120 {
        let mut b = s.buffer(1920 * 1080 * 4);
        for (k, px) in b.chunks_mut(4).enumerate() { px[0] = (k % 256) as u8; px[1] = i as u8; px[2] = 0; px[3] = 255; }
        s.send(ndi::FrameMsg { w: 1920, h: 1080, data: b });
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    eprintln!("sent");
    drop(s);
    eprintln!("dropped");
}
