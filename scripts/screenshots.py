#!/usr/bin/env python3
"""Make all screenshots of the website and the manual (website/img/*.png) from self-tests.

usage: scripts/screenshots.py [--recording <mp4>] [--query arrow]

Sources in the pictures: a generator from the pack (kaleido flower), an NDI source (a second
self-test videoping sends the pack's endless corridor as "corridor"), your last recording (the newest
mp4 in ~/Videos/videoping, or --recording) and the first videopong.net clip of the search (--query,
default "arrow"; needs the internet). Every self-test runs in its own temporary folder, never with your
session, config or autosave; the ai pictures use the mocks scripts/ai-mock-*.py (no key, nothing is
billed). The NDI library has to be installed (see install.md). The window has to come out 3000x1818
(the crops are made for that). Build first: cargo build --release
"""
import glob, json, os, shutil, subprocess, sys, tempfile, time, urllib.request
from PIL import Image

args = sys.argv[1:]
if args and args[0] in ("-h", "--help"):
    sys.exit(__doc__)
def opt(name, default):
    return args[args.index(name) + 1] if name in args else default
recordings = sorted(glob.glob(os.path.expanduser("~/Videos/videoping/*.mp4")), key=os.path.getmtime)
recording = os.path.abspath(opt("--recording", recordings[-1] if recordings else ""))
query = opt("--query", "arrow")
if not os.path.isfile(recording):
    sys.exit("no recording: pass --recording <mp4>")
repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
out = os.path.join(repo, "website", "img")
exe = os.path.join(repo, "target", "release", "videoping")
pack = os.path.join(repo, "isf-pack")
W, H = 3000, 1818
port0 = 7800 + os.getpid() % 100


def start(name, env, session=None, mock=None):
    """A self-test videoping in its own folder; returns (process, folder, mock server)."""
    d = tempfile.mkdtemp(prefix=f"vp-shot-{name}-")
    env = dict(env)
    env.setdefault("VIDEOPING_SELFTEST_CLIP", "/nonexistent")
    if session is not None:
        p = os.path.join(d, "session.json")
        json.dump(session, open(p, "w"), indent=1)
        env["VIDEOPING_SELFTEST_SESSION"] = p
    server = None
    if mock:
        port = str(port0 + 50)
        server = subprocess.Popen([sys.executable, os.path.join(repo, "scripts", mock), port, os.path.join(d, "mock.jsonl")])
        time.sleep(1)
        env["VIDEOPING_SELFTEST_AI"] = f"http://127.0.0.1:{port}"
    proc = subprocess.Popen([exe], env=dict(os.environ, VIDEOPING_SELFTEST=d, **env), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return proc, d, server


def shot(name, env, session=None, mock=None):
    proc, d, server = start(name, env, session, mock)
    try:
        proc.wait(timeout=120)
    finally:
        if server:
            server.terminate()
    f = os.path.join(d, f"ui_{W}x{H}.rgba")
    if not os.path.exists(f):
        sys.exit(f"{name}: no {W}x{H} screenshot (other screen size? see {d})")
    print("took", name)
    return Image.frombytes("RGBA", (W, H), open(f, "rb").read()).convert("RGB"), d


def pal(im, n=160):
    return im.quantize(colors=n, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)


def save(im, name):
    im.save(os.path.join(out, name), optimize=True)
    print("wrote", name, im.size)


def anim(v, lo, hi):
    return {"Anim": [{"value": v, "lo": lo, "hi": hi, "mode": "Pause", "loop_mode": "Loop", "speed": {"bpm_mode": False, "free": 0.25, "beats": 4.0}, "phase": 0.0, "dir": 1.0}]}


OUTPUT = {"width": 1920, "height": 1080, "window_open": False, "fullscreen": False, "ndi_enabled": False, "ndi_name": "videoping", "ndi_lib_dir": "",
          "fps": 60, "screen": 0, "prefer_nvidia": False, "mapping": "", "mapping_on": False, "map_ndi": False, "map_rec": False}
MIX = lambda t="Alpha", a=1.0: {"mix_type": t, "amount": a, "rgb": [True, True, True]}
TRANSPORT = {"mode": "Forward", "loop_mode": "Loop", "speed": {"bpm_mode": False, "free": 1.0, "beats": 4.0}, "in_pt": 0.0, "out_pt": 1.0}


def session(slots, layers, selected, roots):
    full = slots + [None] * (32 - len(slots))
    return {"slots": full, "page": 0, "selected_instance": selected, "layers": layers, "selected_layer": 1, "bpm": 124.0, "output_effects": [],
            "master": {"brightness": 1.0}, "output": dict(OUTPUT), "finder_roots": roots, "finder_favs": [], "next_id": 100}


# 1. videopong.net: the search, its finder picture and the first clip (downloaded into the folder)
im, vpdir = shot("videopong", {"VIDEOPING_SELFTEST_VP": query, "VIDEOPING_SELFTEST_STAY": "22"})
save(im.crop((0, 36, 390, 36 + 1060)), "videopong.png")
vpclips = [f for f in glob.glob(os.path.join(vpdir, "**", "*.mp4"), recursive=True)]
if not vpclips:
    sys.exit("videopong: no clip downloaded")
vpclip = vpclips[0]
print("videopong clip:", os.path.basename(vpclip))

# 2. the NDI source: a second videoping sends the endless corridor as "corridor"
sender_session = session([{"id": 1, "name": "corridor", "source": {"Generator": {"path": f"{pack}/generators/endless-corridor.fs", "params": []}}, "effects": [], "seed": 0.3}],
                         [{"instance": 1, "active": True, "mix": MIX()}, {"instance": None, "active": True, "mix": MIX()}, {"instance": None, "active": True, "mix": MIX()}], None, [pack])
sender_session["output"].update({"ndi_enabled": True, "ndi_name": "corridor"})
sender, _, _ = start("ndi-sender", {"VIDEOPING_SELFTEST_STAY": "300", "VIDEOPING_SELFTEST_SHOT_AT": "290"}, sender_session)
try:
    # its full network name ("MACHINE (corridor)"), from a probe with MCP
    mport = str(port0 + 60)
    probe, _, _ = start("ndi-probe", {"VIDEOPING_SELFTEST_MCP": mport, "VIDEOPING_SELFTEST_STAY": "40"})
    ndi = None
    for _ in range(60):
        time.sleep(0.5)
        try:
            req = urllib.request.Request(f"http://127.0.0.1:{mport}/mcp", json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "list_files", "arguments": {"kind": "ndi"}}}).encode(), {"Content-Type": "application/json"})
            text = json.load(urllib.request.urlopen(req, timeout=5))["result"]["content"][0]["text"]
            ndi = next((l.split(" | ", 1)[1] for l in text.splitlines() if l.startswith("ndi | ") and "(corridor)" in l), None)
            if ndi:
                break
        except Exception:
            pass
    probe.terminate()
    if not ndi:
        sys.exit("NDI: the sender was not found (NDI library installed?)")
    print("ndi source:", ndi)

    # 3. the session of the pictures; the finder shows only these clips (a folder of links to them)
    media = tempfile.mkdtemp(prefix="vp-shot-media-")
    rec_link = os.path.join(media, os.path.basename(recording))
    vp_link = os.path.join(media, os.path.basename(vpclip))
    os.symlink(recording, rec_link)
    os.symlink(vpclip, vp_link)
    recording, vpclip = rec_link, vp_link
    slots = [
        {"id": 1, "name": "kaleido flower", "source": {"Generator": {"path": f"{pack}/generators/kaleido-flower.fs", "params": [{"name": "bgAlpha", "value": anim(0.0, 0.0, 1.0)}]}}, "effects": [], "seed": 0.5},
        {"id": 2, "name": "corridor (NDI)", "source": {"Ndi": {"source": ndi}}, "effects": [], "seed": 0.5},
        {"id": 3, "name": os.path.splitext(os.path.basename(recording))[0], "source": {"Clip": {"path": recording, "transport": dict(TRANSPORT)}}, "effects": [], "seed": 0.5},
        {"id": 4, "name": os.path.splitext(os.path.basename(vpclip))[0], "source": {"Clip": {"path": vpclip, "transport": dict(TRANSPORT)}},
         "effects": [{"uid": 10, "path": f"{pack}/effects/pixelate.fs", "active": True, "mix": MIX(), "params": [{"name": "shape", "value": anim(1.0, 0.0, 4.0)}], "seed": 0.5, "open": True},
                     {"uid": 11, "path": f"{pack}/effects/rgb-split.fs", "active": True, "mix": MIX(), "params": [], "seed": 0.5, "open": False}], "seed": 0.5},
    ]
    layers = [{"instance": 1, "active": True, "mix": MIX()}, {"instance": 2, "active": True, "mix": MIX("LumaKey")}, {"instance": 3, "active": True, "mix": MIX()}]
    main = session(slots, layers, 4, [pack, media])

    im, _ = shot("live", {"VIDEOPING_SELFTEST_SHOT_AT": "9"}, main)
    save(pal(im.resize((1500, 909), Image.LANCZOS)), "overview.png")
    bar = Image.new("RGB", (2000, 36), im.getpixel((1000, 18)))  # top bar without its empty middle
    bar.paste(im.crop((0, 0, 720, 36)), (0, 0))
    bar.paste(im.crop((1740, 0, 3000, 36)), (740, 0))
    save(bar, "topbar.png")
    save(im.crop((0, 36, 390, 36 + 648)), "finder.png")
    save(im.crop((392, 36, 392 + 620, 36 + 1440)), "prepare.png")
    save(pal(im.crop((1022, 36, 1022 + 890, 36 + 1700))), "detail.png")
    save(pal(im.crop((1920, 36, 1920 + 1080, 36 + 1100))), "mixer.png")

    im, _ = shot("editor", {"VIDEOPING_SELFTEST_TAB": "shader", "VIDEOPING_SELFTEST_SHOT_AT": "9", "VIDEOPING_SELFTEST_EDIT": f"{pack}/effects/invert.fs"}, main, "ai-mock-shader.py")
    save(pal(im.resize((1500, 909), Image.LANCZOS), 200), "editor.png")

    im, _ = shot("ai", {"VIDEOPING_SELFTEST_TAB": "ai", "VIDEOPING_SELFTEST_SHOT_AT": "9"}, main, "ai-mock-live.py")
    save(pal(im.crop((0, 0, W, 640)).resize((1500, 320), Image.LANCZOS), 200), "ai.png")
finally:
    sender.terminate()

# 4. the tabs without video: the built-in test session
im, _ = shot("midi", {"VIDEOPING_SELFTEST_MIDI": "1", "VIDEOPING_SELFTEST_TAB": "midi"})
save(pal(im.crop((0, 0, W, 1000)).resize((1500, 500), Image.LANCZOS)), "midi.png")
im, _ = shot("mapping", {"VIDEOPING_SELFTEST_MAP": "grid", "VIDEOPING_SELFTEST_TAB": "mapping"})
save(pal(im.crop((0, 0, 1800, H)).resize((900, 909), Image.LANCZOS)), "mapping.png")
