#!/usr/bin/env python3
"""Make all screenshots of the website and the manual (website/img/*.png) from self-tests.

usage: scripts/screenshots.py <testclip.mp4> [--no-videopong]

Runs a self-test videoping per picture (its own folder each time, never your session, config or
autosave), with the repository's shaders, the given clip and the ai mocks (scripts/ai-mock-*.py,
no API key, nothing is billed); the videopong.net picture needs the internet. The window has to
come out 3000x1818 (the crops are made for that); build first: cargo build --release
"""
import os, shutil, subprocess, sys, tempfile, time
from PIL import Image

args = sys.argv[1:]
if not args:
    sys.exit(__doc__)
clip = os.path.abspath(args[0])
videopong = "--no-videopong" not in args
repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
out = os.path.join(repo, "website", "img")
exe = os.path.join(repo, "target", "release", "videoping")
W, H = 3000, 1818


def shot(name, env, mock=None):
    d = tempfile.mkdtemp(prefix=f"vp-shot-{name}-")
    shutil.copy(clip, os.path.join(d, "testclip.mp4"))
    server = None
    if mock:
        port = str(7800 + os.getpid() % 100)
        server = subprocess.Popen([sys.executable, os.path.join(repo, "scripts", mock), port, os.path.join(d, "mock.jsonl")])
        time.sleep(1)
        env = dict(env, VIDEOPING_SELFTEST_AI=f"http://127.0.0.1:{port}")
    try:
        subprocess.run([exe], env=dict(os.environ, VIDEOPING_SELFTEST=d, **env), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=120)
    finally:
        if server:
            server.terminate()
    f = os.path.join(d, f"ui_{W}x{H}.rgba")
    if not os.path.exists(f):
        sys.exit(f"{name}: no {W}x{H} screenshot (other screen size? see {d})")
    print("took", name)
    return Image.frombytes("RGBA", (W, H), open(f, "rb").read()).convert("RGB")


def pal(im, n=160):
    return im.quantize(colors=n, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)


def save(im, name):
    im.save(os.path.join(out, name), optimize=True)
    print("wrote", name, im.size)


# the live tab: the pack's hex pulse over the clip, the clip with pixelate (a row of choice buttons)
im = shot("live", {"VIDEOPING_SELFTEST_EFFECT": os.path.join(repo, "isf-pack/effects/pixelate.fs")})
save(pal(im.resize((1500, 909), Image.LANCZOS)), "overview.png")
bar = Image.new("RGB", (2000, 36), im.getpixel((1000, 18)))  # top bar without its empty middle
bar.paste(im.crop((0, 0, 720, 36)), (0, 0))
bar.paste(im.crop((1740, 0, 3000, 36)), (740, 0))
save(bar, "topbar.png")
save(im.crop((0, 36, 390, 36 + 648)), "finder.png")
save(im.crop((392, 36, 392 + 620, 36 + 1440)), "prepare.png")
save(pal(im.crop((1022, 36, 1022 + 890, 36 + 1700))), "detail.png")
save(pal(im.crop((1920, 36, 1920 + 1080, 36 + 1100))), "mixer.png")

im = shot("midi", {"VIDEOPING_SELFTEST_MIDI": "1", "VIDEOPING_SELFTEST_TAB": "midi"})
save(pal(im.crop((0, 0, W, 1000)).resize((1500, 500), Image.LANCZOS)), "midi.png")

im = shot("mapping", {"VIDEOPING_SELFTEST_MAP": "grid", "VIDEOPING_SELFTEST_TAB": "mapping"})
save(pal(im.crop((0, 0, 1800, H)).resize((900, 909), Image.LANCZOS)), "mapping.png")

im = shot("editor", {"VIDEOPING_SELFTEST_TAB": "shader", "VIDEOPING_SELFTEST_EDIT": os.path.join(repo, "isf-pack/effects/invert.fs")}, "ai-mock-shader.py")
save(pal(im.resize((1500, 909), Image.LANCZOS), 200), "editor.png")

im = shot("ai", {"VIDEOPING_SELFTEST_TAB": "ai"}, "ai-mock-live.py")
save(pal(im.crop((0, 0, W, 640)).resize((1500, 320), Image.LANCZOS), 200), "ai.png")

if videopong:
    im = shot("videopong", {"VIDEOPING_SELFTEST_VP": "tunnel", "VIDEOPING_SELFTEST_STAY": "22"})
    save(im.crop((0, 36, 390, 36 + 1060)), "videopong.png")
