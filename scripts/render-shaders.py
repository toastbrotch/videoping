#!/usr/bin/env python3
"""Render ISF shaders in a real videoping and save what they look like.

usage: scripts/render-shaders.py <outdir> <shader.fs>... [--set P=value ...] [--wait seconds] [--width px]

Starts a self-test videoping (MCP on, its own folder, never your session) with the shaders' folders
linked into its finder, loads each shader into slot 20, applies the --set values (P = parameter
number as the ui counts it, e.g. 1=2 picks the second style), saves two frames as PNG, prints the
GPU time of the frame and any compile errors, then stops videoping. Build first: cargo build --release
"""
import base64, json, os, subprocess, sys, tempfile, time, urllib.request

args = sys.argv[1:]
if not args:
    sys.exit(__doc__)
out = os.path.abspath(args.pop(0))
sets, files, wait, width = [], [], 1.5, 480
while args:
    a = args.pop(0)
    if a == "--set":
        sets.append(args.pop(0))
    elif a == "--wait":
        wait = float(args.pop(0))
    elif a == "--width":
        width = int(args.pop(0))
    else:
        files.append(os.path.abspath(a))
os.makedirs(out, exist_ok=True)
repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
tmp = tempfile.mkdtemp(prefix="vp-render-")
for i, d in enumerate(sorted({os.path.dirname(f) for f in files})):
    os.symlink(d, os.path.join(tmp, f"shaders{i}"))
port = 7480 + os.getpid() % 100
env = dict(os.environ, VIDEOPING_SELFTEST=tmp, VIDEOPING_SELFTEST_MCP=str(port), VIDEOPING_SELFTEST_STAY="600",
           VIDEOPING_SELFTEST_CLIP="/nonexistent")
proc = subprocess.Popen([os.path.join(repo, "target/release/videoping")], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
URL = f"http://127.0.0.1:{port}/mcp"


def call(name, a):
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": name, "arguments": a}}).encode()
    req = urllib.request.Request(URL, body, {"Content-Type": "application/json"})
    return json.load(urllib.request.urlopen(req, timeout=30))["result"]


try:
    for _ in range(120):
        try:
            call("state", {})
            break
        except Exception:
            time.sleep(0.25)
    time.sleep(8)  # the self-test's own mouse script clears the session in its first seconds
    for f in files:
        name = os.path.splitext(os.path.basename(f))[0]
        r = call("load_file", {"path": f, "slot": 20})
        if r.get("isError"):
            print(name, r["content"][0]["text"])
            continue
        call("set", {"address": "/slot/20/select"})
        for s in sets:
            k, v = s.split("=", 1)
            call("set", {"address": f"/slot/20/param/{k}", "value": float(v)})
        suffix = "".join("-" + s.replace("=", "") for s in sets)
        for n in (1, 2):
            time.sleep(wait)
            c = call("frame", {"source": "preview", "width": width})["content"][0]
            if c.get("type") == "image":
                open(os.path.join(out, f"{name}{suffix}-{n}.png"), "wb").write(base64.b64decode(c["data"]))
        gpu = next((l for l in call("state", {})["content"][0]["text"].splitlines() if l.startswith("output")), "")
        print(f"{name}{suffix}: {gpu[gpu.find('GPU'):gpu.find(' per frame')] if 'GPU' in gpu else '?'} (whole frame)")
    print(call("shader_errors", {})["content"][0]["text"])
finally:
    proc.terminate()
