//! AI chat client for the "ai" tab: one request to an LLM provider, blocking, run in a worker
//! thread. The tools are the MCP tools (mcp.rs), answered by the app (ai_tab.rs).
//!
//! Providers:
//! - Anthropic Messages API (raw HTTP, API version 2026-10-21): adaptive thinking on the current
//!   models (effort setting), the assistant content is passed back unchanged (thinking blocks),
//!   tool results may carry images, top-level prompt caching, server-side refusal fallback on
//!   the models that have it
//! - OpenAI-compatible chat completions (OpenAI, Ollama, LM Studio, ...): tools as functions;
//!   images of tool results follow as a user message (tool messages are text only)

use serde_json::{json, Value};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

pub const ANTHROPIC_URL: &str = "https://api.anthropic.com";
pub const OPENAI_URL: &str = "https://api.openai.com/v1";
pub const ANTHROPIC_MODELS: [&str; 4] = ["claude-opus-5-5", "claude-sonnet-5-5", "claude-haiku-5-5", "claude-fable-5-1"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Provider {
    Anthropic,
    OpenAi,
}

impl Provider {
    pub fn from_str(s: &str) -> Provider {
        if s == "openai" { Provider::OpenAi } else { Provider::Anthropic }
    }
}

/// What one request needs (copied from the config).
#[derive(Clone, Debug)]
pub struct Settings {
    pub provider: Provider,
    pub model: String,
    pub key: String,
    /// empty = the provider's default
    pub url: String,
    /// anthropic: low / medium / high
    pub effort: String,
}

#[derive(Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: Value,
}

#[derive(Debug, Default)]
pub struct Reply {
    pub text: Vec<String>,
    pub calls: Vec<ToolCall>,
    /// the assistant message to append to the conversation, exactly as the provider sent it
    pub message: Option<Value>,
    pub stop: String,
    /// the provider declined (safety); the explanation if there is one
    pub refusal: Option<String>,
    /// (input tokens incl. cached, output tokens)
    pub usage: (u64, u64),
}

/// A tool answer to send back.
#[derive(Clone, Debug)]
pub struct ToolOut {
    pub id: String,
    pub text: String,
    pub is_error: bool,
    pub png: Option<Vec<u8>>,
}

/// The tools in the provider's format (from the MCP tool list).
pub fn tools(p: Provider) -> Value {
    let list = crate::mcp::tools();
    let v: Vec<Value> = list
        .as_array()
        .map(|a| {
            a.iter()
                .map(|t| match p {
                    Provider::Anthropic => json!({ "name": t["name"], "description": t["description"], "input_schema": t["inputSchema"] }),
                    Provider::OpenAi => json!({ "type": "function", "function": { "name": t["name"], "description": t["description"], "parameters": t["inputSchema"] } }),
                })
                .collect()
        })
        .unwrap_or_default();
    Value::Array(v)
}

/// Append a user text: merged into a trailing user message (after an error or a stop the
/// conversation can end with one), so the roles keep alternating.
pub fn push_user_text(p: Provider, msgs: &mut Vec<Value>, text: &str) {
    if p == Provider::Anthropic {
        if let Some(last) = msgs.last_mut().filter(|m| m["role"] == "user") {
            let block = json!({ "type": "text", "text": text });
            match &mut last["content"] {
                Value::Array(a) => a.push(block),
                c => {
                    let old = c.take();
                    *c = json!([{ "type": "text", "text": old.as_str().unwrap_or("") }, block]);
                }
            }
            return;
        }
    }
    msgs.push(json!({ "role": "user", "content": text }));
}

/// The tool answers as messages (all answers of one turn together).
pub fn push_tool_results(p: Provider, msgs: &mut Vec<Value>, outs: &[ToolOut]) {
    use base64::Engine;
    let b64 = |b: &[u8]| base64::engine::general_purpose::STANDARD.encode(b);
    match p {
        Provider::Anthropic => {
            let blocks: Vec<Value> = outs
                .iter()
                .map(|o| {
                    // (the API refuses empty text blocks)
                    let text = if o.text.trim().is_empty() { if o.png.is_some() { "the image" } else { "done" } } else { o.text.as_str() };
                    let mut content = vec![json!({ "type": "text", "text": text })];
                    if let Some(png) = &o.png {
                        content.push(json!({ "type": "image", "source": { "type": "base64", "media_type": "image/png", "data": b64(png) } }));
                    }
                    json!({ "type": "tool_result", "tool_use_id": o.id, "content": content, "is_error": o.is_error })
                })
                .collect();
            msgs.push(json!({ "role": "user", "content": blocks }));
        }
        Provider::OpenAi => {
            for o in outs {
                let text = if o.is_error { format!("error: {}", o.text) } else if o.text.trim().is_empty() { "done".into() } else { o.text.clone() };
                msgs.push(json!({ "role": "tool", "tool_call_id": o.id, "content": text }));
            }
            let images: Vec<Value> = outs
                .iter()
                .filter_map(|o| o.png.as_ref())
                .map(|png| json!({ "type": "image_url", "image_url": { "url": format!("data:image/png;base64,{}", b64(png)) } }))
                .collect();
            if !images.is_empty() {
                let mut content = vec![json!({ "type": "text", "text": "the image(s) from the tool call above" })];
                content.extend(images);
                msgs.push(json!({ "role": "user", "content": content }));
            }
        }
    }
}

/// Current models: adaptive thinking + effort (older ones reject effort).
fn has_effort(model: &str) -> bool {
    let m = model.trim_start_matches("anthropic.");
    ["claude-opus-5", "claude-sonnet-5", "claude-haiku-5", "claude-fable-5", "claude-mythos-5", "claude-opus-4-6", "claude-opus-4-7", "claude-opus-4-8", "claude-sonnet-4-6"]
        .iter()
        .any(|p| m.starts_with(p))
}

/// Models with server-side refusal fallback ("default" routing).
fn has_fallback(model: &str) -> bool {
    ["claude-opus-5-5", "claude-opus-5", "claude-fable-5-1", "claude-sonnet-5-5"].contains(&model)
}

pub fn anthropic_body(s: &Settings, system: &str, msgs: &[Value]) -> Value {
    let mut body = json!({
        "model": s.model,
        "max_tokens": 16000,
        "system": system,
        "tools": tools(Provider::Anthropic),
        "messages": msgs,
        // automatic prompt caching: tools + system + history are the same prefix every round
        "cache_control": { "type": "ephemeral" },
    });
    if has_effort(&s.model) {
        body["output_config"] = json!({ "effort": s.effort });
    }
    if has_fallback(&s.model) {
        body["fallbacks"] = json!("default");
    }
    body
}

pub fn openai_body(s: &Settings, system: &str, msgs: &[Value]) -> Value {
    let mut all = vec![json!({ "role": "system", "content": system })];
    all.extend(msgs.iter().cloned());
    json!({ "model": s.model, "messages": all, "tools": tools(Provider::OpenAi) })
}

fn http_error(status: u16, body: &str) -> String {
    let msg = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v["error"]["message"].as_str().map(String::from).or_else(|| v["error"].as_str().map(String::from)))
        .unwrap_or_else(|| body.chars().take(300).collect());
    match status {
        401 | 403 => format!("{status}: key not accepted ({msg})"),
        429 => format!("{status}: rate limit or quota ({msg})"),
        _ => format!("{status}: {msg}"),
    }
}

/// One request, blocking.
pub fn request(s: &Settings, system: &str, msgs: &[Value]) -> Result<Reply, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(300))).http_status_as_error(false).build().into();
    let base = s.url.trim().trim_end_matches('/');
    let (url, body) = match s.provider {
        Provider::Anthropic => (format!("{}/v1/messages", if base.is_empty() { ANTHROPIC_URL } else { base }), anthropic_body(s, system, msgs)),
        Provider::OpenAi => (format!("{}/chat/completions", if base.is_empty() { OPENAI_URL } else { base }), openai_body(s, system, msgs)),
    };
    let mut req = agent.post(&url).header("content-type", "application/json");
    match s.provider {
        Provider::Anthropic => {
            req = req.header("x-api-key", s.key.trim()).header("anthropic-version", "2026-10-21");
            if has_fallback(&s.model) {
                req = req.header("anthropic-beta", "server-side-fallback-2026-07-01");
            }
        }
        Provider::OpenAi => {
            if !s.key.trim().is_empty() {
                req = req.header("authorization", format!("Bearer {}", s.key.trim()));
            }
        }
    }
    let mut resp = req.send_json(&body).map_err(|e| format!("{url}: {e}"))?;
    let status = resp.status().as_u16();
    let text = resp.body_mut().with_config().limit(64 * 1024 * 1024).read_to_string().map_err(|e| e.to_string())?;
    if status >= 400 {
        return Err(http_error(status, &text));
    }
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("answer is not JSON: {e}"))?;
    Ok(match s.provider {
        Provider::Anthropic => parse_anthropic(&v),
        Provider::OpenAi => parse_openai(&v),
    })
}

pub fn parse_anthropic(v: &Value) -> Reply {
    let mut r = Reply { stop: v["stop_reason"].as_str().unwrap_or("").to_string(), ..Default::default() };
    let content = v["content"].as_array().cloned().unwrap_or_default();
    for b in &content {
        match b["type"].as_str() {
            Some("text") => {
                if let Some(t) = b["text"].as_str().filter(|t| !t.trim().is_empty()) {
                    r.text.push(t.to_string());
                }
            }
            Some("tool_use") => r.calls.push(ToolCall {
                id: b["id"].as_str().unwrap_or("").to_string(),
                name: b["name"].as_str().unwrap_or("").to_string(),
                input: b["input"].clone(),
            }),
            _ => {}
        }
    }
    if r.stop == "refusal" {
        r.refusal = Some(v["stop_details"]["explanation"].as_str().unwrap_or("").to_string());
        r.calls.clear();
    } else {
        // the whole content back unchanged (thinking blocks, fallback blocks)
        r.message = Some(json!({ "role": "assistant", "content": content }));
    }
    let u = &v["usage"];
    let n = |k: &str| u[k].as_u64().unwrap_or(0);
    r.usage = (n("input_tokens") + n("cache_read_input_tokens") + n("cache_creation_input_tokens"), n("output_tokens"));
    r
}

pub fn parse_openai(v: &Value) -> Reply {
    let choice = &v["choices"][0];
    let msg = &choice["message"];
    let mut r = Reply { stop: choice["finish_reason"].as_str().unwrap_or("").to_string(), ..Default::default() };
    if let Some(t) = msg["content"].as_str().filter(|t| !t.trim().is_empty()) {
        r.text.push(t.to_string());
    }
    let mut calls_json = vec![];
    for c in msg["tool_calls"].as_array().cloned().unwrap_or_default() {
        let args = c["function"]["arguments"].clone();
        // arguments are a JSON string (some servers send an object)
        let input = match &args {
            Value::String(s) => serde_json::from_str(s).unwrap_or(json!({})),
            Value::Object(_) => args.clone(),
            _ => json!({}),
        };
        let id = c["id"].as_str().map(String::from).unwrap_or_else(|| format!("call_{}", calls_json.len()));
        let name = c["function"]["name"].as_str().unwrap_or("").to_string();
        calls_json.push(json!({ "id": id, "type": "function", "function": { "name": name, "arguments": input.to_string() } }));
        r.calls.push(ToolCall { id, name, input });
    }
    let mut m = json!({ "role": "assistant", "content": msg["content"].as_str().unwrap_or("") });
    if !calls_json.is_empty() {
        m["tool_calls"] = Value::Array(calls_json);
    }
    r.message = Some(m);
    r.usage = (v["usage"]["prompt_tokens"].as_u64().unwrap_or(0), v["usage"]["completion_tokens"].as_u64().unwrap_or(0));
    r
}

/// The request in a worker thread; the answer arrives on the channel.
pub fn spawn(s: Settings, system: String, msgs: Vec<Value>) -> Receiver<Result<Reply, String>> {
    let (tx, rx) = channel();
    let _ = std::thread::Builder::new().name("ai".into()).spawn(move || {
        let _ = tx.send(request(&s, &system, &msgs));
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anthropic_roundtrip() {
        let v = json!({
            "content": [
                { "type": "thinking", "thinking": "", "signature": "x" },
                { "type": "text", "text": "Dimming." },
                { "type": "tool_use", "id": "t1", "name": "set", "input": { "address": "/master/brightness", "value": 0.5 } }
            ],
            "stop_reason": "tool_use",
            "usage": { "input_tokens": 10, "cache_read_input_tokens": 90, "output_tokens": 5 }
        });
        let r = parse_anthropic(&v);
        assert_eq!(r.text, vec!["Dimming."]);
        assert_eq!(r.calls[0].name, "set");
        assert_eq!(r.usage, (100, 5));
        // the thinking block goes back unchanged
        assert_eq!(r.message.as_ref().unwrap()["content"][0]["type"], "thinking");
        let mut msgs = vec![json!({ "role": "user", "content": "dim it" }), r.message.unwrap()];
        push_tool_results(Provider::Anthropic, &mut msgs, &[ToolOut { id: "t1".into(), text: "".into(), is_error: false, png: Some(vec![1, 2]) }]);
        assert_eq!(msgs[2]["content"][0]["type"], "tool_result");
        assert_eq!(msgs[2]["content"][0]["content"][0]["text"], "the image");
        assert_eq!(msgs[2]["content"][0]["content"][1]["type"], "image");
        // a new user text after the tool results is merged into that user message
        push_user_text(Provider::Anthropic, &mut msgs, "thanks");
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[2]["content"][1]["text"], "thanks");
        let s = Settings { provider: Provider::Anthropic, model: "claude-opus-5-5".into(), key: String::new(), url: String::new(), effort: "low".into() };
        let b = anthropic_body(&s, "sys", &msgs);
        assert_eq!(b["output_config"]["effort"], "low");
        assert_eq!(b["fallbacks"], "default");
        assert_eq!(b["tools"][0]["name"], "state");
        // refusal: no message to append, no calls
        let r = parse_anthropic(&json!({ "content": [], "stop_reason": "refusal", "stop_details": { "explanation": "no" } }));
        assert!(r.message.is_none() && r.refusal.as_deref() == Some("no"));
    }

    #[test]
    fn openai_roundtrip() {
        let v = json!({ "choices": [{ "finish_reason": "tool_calls", "message": { "role": "assistant", "content": null,
            "tool_calls": [{ "id": "c1", "type": "function", "function": { "name": "frame", "arguments": "{\"width\": 320}" } }] } }],
            "usage": { "prompt_tokens": 7, "completion_tokens": 3 } });
        let r = parse_openai(&v);
        assert_eq!(r.calls[0].input["width"], 320);
        let mut msgs = vec![r.message.unwrap()];
        push_tool_results(Provider::OpenAi, &mut msgs, &[ToolOut { id: "c1".into(), text: "image".into(), is_error: false, png: Some(vec![0]) }]);
        assert_eq!(msgs[1]["role"], "tool");
        assert_eq!(msgs[2]["content"][1]["type"], "image_url");
        assert_eq!(tools(Provider::OpenAi)[0]["function"]["name"], "state");
    }
}
