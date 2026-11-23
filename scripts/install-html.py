#!/usr/bin/env python3
"""install.md -> website/install.html (the install guide as a page of the website).

install.md stays the one source (readable in a terminal and in the repo); run this after every
change to it:  python3 scripts/install-html.py

Covers the markdown install.md uses: "#" / "===" headings, paragraphs, "-" and "1." lists
(nested by indentation, with code blocks inside), fenced code, tables, **bold**, *italic*,
`code`, [links](url), bare https urls and raw <img> lines. Paths that point into website/ are
made relative to the page; other repo files link one level up.
"""
import html
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "install.md"
DST = ROOT / "website" / "install.html"


def fix_path(url: str) -> str:
    if re.match(r"^[a-z]+:", url) or url.startswith("#"):
        return url
    if url.startswith("website/"):
        return url[len("website/"):]
    return "../" + url


def inline(t: str) -> str:
    # code spans first (their content is not formatted)
    parts = re.split(r"(`[^`]+`)", t)
    out = []
    for p in parts:
        if p.startswith("`") and p.endswith("`") and len(p) > 1:
            out.append("<code>" + html.escape(p[1:-1]) + "</code>")
            continue
        p = html.escape(p, quote=False)
        p = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", lambda m: f'<a href="{html.escape(fix_path(m.group(2)))}">{m.group(1)}</a>', p)
        p = re.sub(r"(?<![\"=>])(https?://[^\s<)]+)", r'<a href="\1">\1</a>', p)
        p = re.sub(r"\*\*(.+?)\*\*", r"<b>\1</b>", p)
        p = re.sub(r"(?<![*\w])\*(?!\s)(.+?)(?<!\s)\*(?![*\w])", r"<i>\1</i>", p)
        out.append(p)
    return "".join(out)


def slug(t: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", t.lower()).strip("-")


def convert(md: str):
    lines = md.split("\n")
    out = []
    toc = []
    title = "install videoping"
    i = 0
    stack = []  # open lists: (indent, tag)

    def close_lists(to_indent=-1):
        while stack and stack[-1][0] > to_indent:
            out.append("</li></" + stack.pop()[1] + ">")

    para = []

    def flush_para():
        if para:
            out.append("<p>" + inline(" ".join(x.strip() for x in para)) + "</p>")
            para.clear()

    while i < len(lines):
        l = lines[i]
        s = l.strip()
        ind = len(l) - len(l.lstrip(" "))
        # fenced code (also inside list items)
        if s.startswith("```"):
            flush_para()
            if stack and ind < stack[-1][0]:
                close_lists(ind - 1)
            code = []
            i += 1
            while i < len(lines) and not lines[i].strip().startswith("```"):
                code.append(lines[i][ind:] if lines[i][:ind].strip() == "" else lines[i])
                i += 1
            out.append("<pre><code>" + html.escape("\n".join(code)) + "</code></pre>")
            i += 1
            continue
        # raw html (the logo)
        if s.startswith("<img"):
            flush_para()
            src = re.search(r'src="([^"]+)"', s)
            if src:
                out.append(f'<img class="logo" src="{fix_path(src.group(1))}" alt="videoping">')
            i += 1
            continue
        # "title\n=====" heading
        if i + 1 < len(lines) and re.match(r"^=+$", lines[i + 1].strip()) and s:
            flush_para()
            close_lists()
            title = s
            out.append(f"<h1>{inline(s)}</h1>")
            i += 2
            continue
        m = re.match(r"^(#{1,3})\s+(.*)$", s)
        if m and ind == 0:
            flush_para()
            close_lists()
            level = len(m.group(1)) + 1
            text = m.group(2)
            sid = slug(text)
            if level == 2:
                toc.append((sid, text))
            out.append(f'<h{level} id="{sid}">{inline(text)}</h{level}>')
            i += 1
            continue
        # tables
        if s.startswith("|") and i + 1 < len(lines) and re.match(r"^\|[\s:|-]+\|$", lines[i + 1].strip()):
            flush_para()
            close_lists()
            row = lambda r: [c.strip() for c in r.strip().strip("|").split("|")]
            head = row(s)
            out.append("<table><tr>" + "".join(f"<th>{inline(c)}</th>" for c in head) + "</tr>")
            i += 2
            while i < len(lines) and lines[i].strip().startswith("|"):
                out.append("<tr>" + "".join(f"<td>{inline(c)}</td>" for c in row(lines[i])) + "</tr>")
                i += 1
            out.append("</table>")
            continue
        # list items
        m = re.match(r"^(\s*)(-|\d+\.)\s+(.*)$", l)
        if m:
            flush_para()
            ind = len(m.group(1))
            tag = "ul" if m.group(2) == "-" else "ol"
            if stack and ind < stack[-1][0]:
                close_lists(ind)
            if stack and stack[-1][0] == ind:
                out.append("</li><li>")
            else:
                out.append(f"<{tag}><li>")
                stack.append((ind, tag))
            item = [m.group(3)]
            i += 1
            # continuation lines of this item (more indented, not a new item / block)
            while i < len(lines):
                n = lines[i]
                ns = n.strip()
                nind = len(n) - len(n.lstrip(" "))
                if ns and nind > ind and not re.match(r"^\s*(-|\d+\.)\s+", n) and not ns.startswith("```"):
                    item.append(ns)
                    i += 1
                else:
                    break
            out.append(inline(" ".join(item)))
            continue
        if not s:
            flush_para()
            # a blank line ends a list unless the next content is indented under it
            j = i + 1
            while j < len(lines) and not lines[j].strip():
                j += 1
            if stack and j < len(lines):
                nind = len(lines[j]) - len(lines[j].lstrip(" "))
                if nind <= stack[0][0] and not re.match(r"^\s*(-|\d+\.)\s+", lines[j]):
                    close_lists()
            i += 1
            continue
        if stack and ind <= stack[0][0]:
            close_lists()
        para.append(l)
        i += 1
    flush_para()
    close_lists()
    return title, toc, "\n".join(out)


PAGE = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>install videoping</title>
<meta name="description" content="How to install videoping on Linux: requirements, NDI, build, launcher, NVIDIA on hybrid laptops.">
<link rel="icon" href="logo/videoping-v2-icon.svg">
<!-- generated from install.md by scripts/install-html.py: edit install.md, then run the script -->
<style>
:root {{ --bg:#141414; --card:#1e1e23; --line:#2a2a31; --text:#e9e7ec; --muted:#9b98a3; --accent:#8b4a87; --accent2:#c999c5;
  --l1:#1b6898; --l2:#388e3c; --l3:#c8640a; }}
* {{ box-sizing: border-box; }}
body {{ margin:0; background:var(--bg); color:var(--text); font:16.5px/1.65 system-ui,-apple-system,"Segoe UI",Ubuntu,Cantarell,"DejaVu Sans",sans-serif; }}
a {{ color:var(--accent2); }}
.stripes {{ display:flex; height:4px; }} .stripes i {{ flex:1; }}
.stripes i:nth-child(1) {{ background:var(--l1); }} .stripes i:nth-child(2) {{ background:var(--l2); }} .stripes i:nth-child(3) {{ background:var(--l3); }}
nav.top {{ border-bottom:1px solid var(--line); }}
nav.top .in {{ max-width:900px; margin:0 auto; padding:0 20px; display:flex; gap:20px; align-items:center; height:56px; }}
nav.top .in a {{ color:var(--muted); text-decoration:none; }} nav.top .in a:hover {{ color:var(--text); }}
nav.top .in .brand {{ color:var(--text); font-weight:700; margin-right:auto; display:flex; gap:8px; align-items:center; }}
nav.top .in .brand img {{ width:24px; height:24px; }}
main {{ max-width:900px; margin:0 auto; padding:30px 20px 80px; }}
img.logo {{ width:min(420px,80%); display:block; margin:10px 0 20px; }}
h1 {{ font-size:38px; margin:0 0 10px; letter-spacing:-.5px; }}
h2 {{ font-size:26px; margin:46px 0 10px; padding-top:18px; border-top:1px solid var(--line); }}
h3 {{ font-size:19px; margin:28px 0 8px; }}
.toc {{ background:var(--card); border:1px solid var(--line); border-radius:10px; padding:14px 20px; margin:20px 0; }}
.toc b {{ color:var(--muted); font-size:13px; text-transform:uppercase; letter-spacing:1px; }}
.toc ol {{ margin:6px 0 0; }}
code {{ font-family:ui-monospace,"DejaVu Sans Mono",Menlo,monospace; font-size:.92em; background:#25252b; padding:1px 5px; border-radius:4px; }}
pre {{ background:#0b0b0c; border:1px solid var(--line); border-radius:8px; padding:14px 16px; overflow-x:auto; scrollbar-color:#3a3a40 #0b0b0c; }}
pre code {{ background:none; padding:0; font-size:14px; color:#d6d3da; }}
li {{ margin:5px 0; }}
table {{ border-collapse:collapse; margin:14px 0; }}
th, td {{ border:1px solid var(--line); padding:7px 12px; text-align:left; }}
th {{ background:var(--card); }}
.alpha {{ border:1px solid rgba(240,195,106,.4); background:rgba(240,195,106,.06); color:#f0c36a; border-radius:8px; padding:10px 14px; font-size:15px; }}
@media (max-width:560px) {{ nav.top .in a:not(.brand) {{ display:none; }} h1 {{ font-size:30px; }} }}
</style>
</head>
<body>
<div class="stripes"><i></i><i></i><i></i></div>
<nav class="top"><div class="in">
  <a class="brand" href="index.html"><img src="logo/videoping-v2-icon.svg" alt="">videoping</a>
  <a href="index.html">home</a><a href="manual.html">manual</a><a href="install.html">install</a>
</div></nav>
<main>
{body_top}
<p class="alpha">videoping is an early alpha, vibecoded with Claude: expect rough edges.</p>
<div class="toc"><b>contents</b><ol>{toc}</ol></div>
{body_rest}
</main>
</body>
</html>
"""


def main():
    title, toc, body = convert(SRC.read_text())
    # the toc goes after the intro (before the first numbered section)
    k = body.find("<h2")
    top, rest = (body[:k], body[k:]) if k >= 0 else (body, "")
    toc_html = "".join(f'<li><a href="#{sid}">{inline(re.sub(r"^[0-9]+\.\s*", "", t))}</a></li>' for sid, t in toc)
    DST.write_text(PAGE.format(body_top=top, toc=toc_html, body_rest=rest))
    print(f"wrote {DST.relative_to(ROOT)}")


if __name__ == "__main__":
    sys.exit(main())
