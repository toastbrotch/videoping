/*{
  "DESCRIPTION": "Two or three line or ring patterns drifting over each other: the interference makes large, slowly moving moire waves.",
  "CREDIT": "videoping shader pack, written by Claude - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "lines", "op-art"],
  "INPUTS": [
    { "NAME": "pattern", "LABEL": "Pattern", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2], "LABELS": ["rings", "lines", "rings + lines"] },
    { "NAME": "layers", "LABEL": "Layers", "TYPE": "long", "DEFAULT": 2, "VALUES": [2, 3], "LABELS": ["2", "3"] },
    { "NAME": "frequency", "LABEL": "Frequency", "TYPE": "float", "DEFAULT": 120.0, "MIN": 20.0, "MAX": 300.0 },
    { "NAME": "drift", "LABEL": "Drift", "TYPE": "float", "DEFAULT": 0.25, "MIN": 0.0, "MAX": 1.5 },
    { "NAME": "spread", "LABEL": "Spread", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "thickness", "LABEL": "Line thickness", "TYPE": "float", "DEFAULT": 0.45, "MIN": 0.1, "MAX": 0.9 },
    { "NAME": "colorA", "LABEL": "Color A", "TYPE": "color", "DEFAULT": [0.96, 0.95, 0.92, 1.0] },
    { "NAME": "colorB", "LABEL": "Color B", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
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

// one transparent grating: 1 = clear, 0 = a dark line
float grating(vec2 p, int k, float t) {
    float fk = float(k);
    // every layer has its own slowly moving center / angle
    vec2 c = spread * vec2(sin(t * (0.7 + 0.31 * fk) + fk * 2.1), cos(t * (0.5 + 0.23 * fk) + fk * 1.3));
    float a = t * (0.13 + 0.07 * fk) * (mod(fk, 2.0) < 1.0 ? 1.0 : -1.0) + fk * 1.0471;
    vec2 dir = vec2(cos(a), sin(a));
    float rings = length(p - c);
    float lines = dot(p - c, dir);
    float v = pattern == 0 ? rings : pattern == 1 ? lines : (k == 1 ? lines : rings);
    float x = v * frequency / 6.2831853;
    float d = abs(fract(x) - 0.5) * 2.0;           // 0 at the line center, 1 between lines
    float aa = fwidth(x) * 2.0;
    return smoothstep(thickness - aa, thickness + aa, d);
}

void main() {
    vec2 p = (gl_FragCoord.xy - 0.5 * RENDERSIZE) / RENDERSIZE.y;
    float t = TIME * drift;
    // gratings laid over each other: light passes only where all are clear
    float m = grating(p, 0, t) * grating(p, 1, t);
    if (layers >= 3) m *= grating(p, 2, t);
    gl_FragColor = withBackground(mix(colorB.rgb, colorA.rgb, m), colorB.rgb, m);
}
