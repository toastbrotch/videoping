/*{
  "DESCRIPTION": "Drifting, pulsing cells with glowing edges. From stained glass to a living tissue to a pure wire net.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "organic", "pattern"],
  "INPUTS": [
    { "NAME": "cells", "LABEL": "Cells", "TYPE": "float", "DEFAULT": 8.0, "MIN": 0.5, "MAX": 30.0 },
    { "NAME": "jitter", "LABEL": "Jitter", "TYPE": "float", "DEFAULT": 0.9, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "drift", "LABEL": "Drift", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "edge", "LABEL": "Edge width", "TYPE": "float", "DEFAULT": 0.04, "MIN": 0.005, "MAX": 0.3 },
    { "NAME": "glow", "LABEL": "Edge glow", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "fill", "LABEL": "Cell fill", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "pulse", "LABEL": "Pulse", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "pulseSpeed", "LABEL": "Pulse speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 5.0 },
    { "NAME": "edgeColor", "LABEL": "Edge color", "TYPE": "color", "DEFAULT": [0.95, 0.75, 0.95, 1.0] },
    { "NAME": "cellColorA", "LABEL": "Cell color A", "TYPE": "color", "DEFAULT": [0.11, 0.41, 0.6, 1.0] },
    { "NAME": "cellColorB", "LABEL": "Cell color B", "TYPE": "color", "DEFAULT": [0.55, 0.29, 0.53, 1.0] },
    { "NAME": "bgColor", "LABEL": "Background", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
    { "NAME": "bgAlpha", "LABEL": "Background (0 = transparent)", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

// background transparency ("Background" slider < 1): straight alpha, so in the mixer (mix type A) the
// layer below shows through where only background is
vec4 bgOver(vec4 b, vec4 a) {
    float oa = a.a + b.a * (1.0 - a.a);
    return vec4((a.rgb * a.a + b.rgb * b.a * (1.0 - a.a)) / max(oa, 1e-4), oa);
}
// col = the finished pixel, bg = the background color, fgA = how much of the pixel is not background
vec4 withBackground(vec3 col, vec3 bg, float fgA) {
    if (bgAlpha >= 0.999) return vec4(col, 1.0);
    fgA = clamp(fgA, 0.0, 1.0);
    vec3 fg = fgA > 1e-4 ? (col - bg * (1.0 - fgA)) / fgA : col;
    return bgOver(vec4(bg, bgAlpha), vec4(clamp(fg, 0.0, 1.0), fgA));
}
// for light on the background: how much brighter than the background a pixel is
vec4 keyed(vec3 col, vec3 bg) {
    vec3 d = max(col - bg, 0.0);
    return withBackground(col, bg, max(d.r, max(d.g, d.b)));
}

vec2 hash2(vec2 p) {
    p = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
    return fract(sin(p) * 43758.5453);
}

void main() {
    vec2 p = (gl_FragCoord.xy - 0.5 * RENDERSIZE) / RENDERSIZE.y * cells;
    vec2 ip = floor(p), fp = fract(p);
    float t = TIME * drift;
    float d1 = 8.0, d2 = 8.0;
    vec2 best = vec2(0.0), bestR = vec2(0.0);
    // nearest point
    for (int y = -1; y <= 1; y++)
    for (int x = -1; x <= 1; x++) {
        vec2 g = vec2(float(x), float(y));
        vec2 h = hash2(ip + g);
        vec2 o = 0.5 + jitter * 0.5 * sin(t + 6.2831 * h);
        vec2 r = g + o - fp;
        float d = dot(r, r);
        if (d < d1) { d1 = d; best = ip + g; bestR = r; }
    }
    // distance to the nearest edge (exact for voronoi)
    float e = 8.0;
    for (int y = -2; y <= 2; y++)
    for (int x = -2; x <= 2; x++) {
        vec2 g = vec2(float(x), float(y));
        vec2 cell = floor(p) + g;
        if (cell == best) continue;
        vec2 h = hash2(cell);
        vec2 o = 0.5 + jitter * 0.5 * sin(t + 6.2831 * h);
        vec2 r = g + o - fp;
        e = min(e, dot(0.5 * (bestR + r), normalize(r - bestR)));
    }
    vec2 hb = hash2(best + 3.7);
    float beat = 0.5 + 0.5 * sin(TIME * pulseSpeed * 3.0 + hb.x * 6.2831);
    float body = (1.0 - pulse) + pulse * beat;
    vec3 cellCol = mix(cellColorA.rgb, cellColorB.rgb, hb.y) * body;
    // a soft core in the middle of each cell
    cellCol *= mix(0.55, 1.15, smoothstep(0.0, 0.8, e));
    float px = cells / RENDERSIZE.y;
    float line = 1.0 - smoothstep(edge * 0.5, edge * 0.5 + px * 1.5, e);
    float halo = glow * exp(-e / max(edge * 2.0, 1e-3)) * 0.5;
    vec3 col = mix(bgColor.rgb, cellCol, fill);
    col += edgeColor.rgb * halo;
    col = mix(col, edgeColor.rgb, line);
    gl_FragColor = keyed(col, bgColor.rgb);
}
