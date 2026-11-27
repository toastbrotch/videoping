# Mock of the Anthropic Messages API for the ai tab self-test: usage: ai-mock-live.py <port> <log.jsonl>
import json, sys
from http.server import BaseHTTPRequestHandler, HTTPServer
LOG = sys.argv[2]
class H(BaseHTTPRequestHandler):
    def log_message(self, *a): pass
    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["content-length"])))
        msgs = body["messages"]
        last = msgs[-1]
        info = {"path": self.path, "version": self.headers.get("anthropic-version"), "key": self.headers.get("x-api-key"),
                "beta": self.headers.get("anthropic-beta"), "model": body["model"], "effort": body.get("output_config"),
                "tools": len(body["tools"]), "n_msgs": len(msgs), "cache": body.get("cache_control"), "fallbacks": body.get("fallbacks")}
        if isinstance(last["content"], list) and last["content"] and last["content"][0].get("type") == "tool_result":
            info["tool_results"] = [(b["tool_use_id"], [c["type"] for c in b["content"]], b["content"][0]["text"][:60]) for b in last["content"]]
            info["thinking_kept"] = msgs[-2]["content"][0]["type"]
            out = {"content": [{"type": "text", "text": "Done: brightness **50 %**, the output looks darker."}], "stop_reason": "end_turn",
                   "usage": {"input_tokens": 50, "cache_read_input_tokens": 1000, "output_tokens": 20}}
        else:
            out = {"content": [{"type": "thinking", "thinking": "", "signature": "sig"},
                               {"type": "text", "text": "Dimming."},
                               {"type": "tool_use", "id": "tu1", "name": "set", "input": {"address": "/master/brightness", "value": 0.5}},
                               {"type": "tool_use", "id": "tu2", "name": "frame", "input": {"width": 320}}],
                   "stop_reason": "tool_use", "usage": {"input_tokens": 1200, "output_tokens": 40}}
        open(LOG, "a").write(json.dumps(info) + "\n")
        data = json.dumps(out).encode()
        self.send_response(200); self.send_header("content-type", "application/json"); self.send_header("content-length", str(len(data))); self.end_headers()
        self.wfile.write(data)
HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
