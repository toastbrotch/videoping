# Mock of the Anthropic Messages API for the shader editor ai self-test: usage: ai-mock-shader.py <port> <log.jsonl>
import json, sys, re
from http.server import BaseHTTPRequestHandler, HTTPServer
LOG = sys.argv[2]
def tool(i, name, inp): return {"type": "tool_use", "id": f"t{i}", "name": name, "input": inp}
class H(BaseHTTPRequestHandler):
    def log_message(self, *a): pass
    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["content-length"])))
        msgs = body["messages"]; n = len(msgs); last = msgs[-1]
        names = [t["name"] for t in body["tools"]]
        res = [b for b in last["content"] if isinstance(last["content"], list) and b.get("type") == "tool_result"] if isinstance(last["content"], list) else []
        info = {"n": n, "tools": names, "results": [[c.get("type") for c in r["content"]] + [r["content"][0].get("text", "")[:120]] for r in res]}
        if n == 1:
            out = [tool(1, "get_shader", {})]
        elif n == 3:
            text = res[0]["content"][0]["text"].split("\n\n", 1)[1]
            info["got_len"] = len(text)
            body_text = text.replace('"INPUTS": [', '"INPUTS": [\n    { "NAME": "uInvert", "LABEL": "Invert", "TYPE": "float", "MIN": 0.0, "MAX": 1.0, "DEFAULT": 0.0 },', 1)
            self.server.good = body_text.replace("gl_FragColor = vec4(col, 1.0);", "gl_FragColor = vec4(mix(col, 1.0 - col, uInvert), 1.0);")
            broken = self.server.good.replace("1.0 - col, uInvert", "1.0 - colr, uInvert")
            out = [tool(2, "set_shader", {"text": broken, "note": "invert slider"})]
        elif n == 5:
            out = [{"type": "text", "text": "Typo, fixing."}, tool(3, "set_shader", {"text": self.server.good, "note": "fix"}), tool(4, "look", {"width": 320})]
        else:
            out = [{"type": "text", "text": "Added an **Invert** slider (0..1)."}]
        open(LOG, "a").write(json.dumps(info) + "\n")
        stop = "tool_use" if any(b["type"] == "tool_use" for b in out) else "end_turn"
        data = json.dumps({"content": out, "stop_reason": stop, "usage": {"input_tokens": 100, "output_tokens": 50}}).encode()
        self.send_response(200); self.send_header("content-type", "application/json"); self.send_header("content-length", str(len(data))); self.end_headers()
        self.wfile.write(data)
HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
