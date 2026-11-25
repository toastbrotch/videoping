/*{
  "DESCRIPTION": "Animated noise drawn as topographic height lines, like a hiking map. Every fifth line is an index line.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "lines", "organic"],
  "INPUTS": [
    { "NAME": "lines", "LABEL": "Lines", "TYPE": "float", "DEFAULT": 16.0, "MIN": 3.0, "MAX": 60.0 },
    { "NAME": "lineWidth", "LABEL": "Line width", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.3, "MAX": 5.0 },
    { "NAME": "scale", "LABEL": "Scale", "TYPE": "float", "DEFAULT": 2.0, "MIN": 0.5, "MAX": 8.0 },
    { "NAME": "speed", "LABEL": "Speed", "TYPE": "float", "DEFAULT": 0.15, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "warp", "LABEL": "Warp", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "flow", "LABEL": "Flow (lines move)", "TYPE": "float", "DEFAULT": 0.1, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "fill", "LABEL": "Height shading", "TYPE": "float", "DEFAULT": 0.2, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "lineColor", "LABEL": "Line color", "TYPE": "color", "DEFAULT": [0.95, 0.9, 0.8, 1.0] },
    { "NAME": "lowColor", "LABEL": "Low color", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
    { "NAME": "highColor", "LABEL": "High color", "TYPE": "color", "DEFAULT": [0.35, 0.18, 0.33, 1.0] },
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

float hash(vec2 p) { p = fract(p * vec2(123.34, 456.21)); p += dot(p, p + 45.32); return fract(p.x * p.y); }

float noise(vec2 p) {
    vec2 i = floor(p), f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
}

float fbm(vec2 p) {
    float v = 0.0, a = 0.5;
    mat2 r = mat2(0.8, 0.6, -0.6, 0.8);
    for (int i = 0; i < 5; i++) { v += a * noise(p); p = r * p * 2.03 + 11.7; a *= 0.5; }
    return v;
}

void main() {
    vec2 p = (gl_FragCoord.xy - 0.5 * RENDERSIZE) / RENDERSIZE.y * scale;
    float t = TIME * speed;
    vec2 q = vec2(fbm(p + vec2(0.0, t)), fbm(p + vec2(5.2, 1.3) - vec2(t * 0.7, 0.0)));
    float h = fbm(p + warp * q + vec2(t * 0.3, -t * 0.2));
    h = clamp((h - 0.15) / 0.7, 0.0, 1.0);

    float v = h * lines - TIME * flow;
    float fw = max(fwidth(v), 1e-4);
    float d = abs(fract(v - 0.5) - 0.5) / fw;
    // every fifth line is a thicker index line
    float idx = abs(fract((v - 0.5) / 5.0) - 0.5) * 5.0 / fw;
    float line = 1.0 - smoothstep(lineWidth * 0.5, lineWidth * 0.5 + 1.0, d);
    float index = 1.0 - smoothstep(lineWidth, lineWidth + 1.0, idx);
    line = max(line, index);

    vec3 ground = mix(lowColor.rgb, highColor.rgb, h);
    vec3 base = mix(lowColor.rgb, ground, fill * 2.0 > 1.0 ? 1.0 : fill * 2.0);
    vec3 col = mix(base, lineColor.rgb, line * mix(0.75, 1.0, index));
    gl_FragColor = keyed(col, lowColor.rgb);
}
