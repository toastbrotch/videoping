/*{
  "DESCRIPTION": "A honeycomb grid lit by waves that ripple out from moving points. Cells light up, hold and fade, like a hex LED floor.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "pattern", "grid"],
  "INPUTS": [
    { "NAME": "hexSize", "LABEL": "Hex size", "TYPE": "float", "DEFAULT": 0.06, "MIN": 0.015, "MAX": 0.25 },
    { "NAME": "gap", "LABEL": "Gap", "TYPE": "float", "DEFAULT": 0.12, "MIN": 0.0, "MAX": 0.6 },
    { "NAME": "sources", "LABEL": "Wave sources", "TYPE": "float", "DEFAULT": 3.0, "MIN": 1.0, "MAX": 6.0 },
    { "NAME": "waveSpeed", "LABEL": "Wave speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 4.0 },
    { "NAME": "waveLength", "LABEL": "Wave length", "TYPE": "float", "DEFAULT": 0.35, "MIN": 0.05, "MAX": 1.5 },
    { "NAME": "sharp", "LABEL": "Wave sharpness", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "wander", "LABEL": "Sources wander", "TYPE": "float", "DEFAULT": 0.2, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "outline", "LABEL": "Outline", "TYPE": "float", "DEFAULT": 0.25, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "colorA", "LABEL": "Color A", "TYPE": "color", "DEFAULT": [0.22, 0.56, 0.24, 1.0] },
    { "NAME": "colorB", "LABEL": "Color B", "TYPE": "color", "DEFAULT": [0.95, 0.95, 0.6, 1.0] },
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

// nearest hex center (pointy top) and the offset to it
vec4 hexCell(vec2 p) {
    vec2 s = vec2(1.0, 1.7320508);
    vec4 c = floor(vec4(p, p - vec2(0.5, 1.0)) / s.xyxy) + 0.5;
    vec4 h = vec4(p - c.xy * s, p - (c.zw + 0.5) * s);
    return dot(h.xy, h.xy) < dot(h.zw, h.zw) ? vec4(h.xy, c.xy) : vec4(h.zw, c.zw + 0.5);
}

float hexDist(vec2 p) {
    p = abs(p);
    return max(dot(p, normalize(vec2(1.0, 1.7320508))), p.x);
}

float hash(vec2 p) { p = fract(p * vec2(123.34, 456.21)); p += dot(p, p + 45.32); return fract(p.x * p.y); }

void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE) / RENDERSIZE.y;
    vec4 h = hexCell(uv / hexSize);
    vec2 local = h.xy;
    vec2 center = h.zw * vec2(1.0, 1.7320508) * hexSize;   // the cell center: the whole cell lights as one
    float energy = 0.0;
    int n = int(sources + 0.5);
    for (int i = 0; i < 6; i++) {
        if (i >= n) break;
        float fi = float(i);
        vec2 src = vec2(0.6 * sin(fi * 2.4 + TIME * wander * (0.3 + 0.1 * fi)), 0.35 * cos(fi * 1.7 + TIME * wander * (0.4 + 0.07 * fi)));
        float d = length(center - src);
        float ph = d / waveLength - TIME * waveSpeed + fi * 0.37;
        float w = 0.5 + 0.5 * cos(ph * 6.2831853);
        w = pow(w, mix(1.0, 12.0, sharp));
        energy += w * exp(-d * 1.2);
    }
    energy = clamp(energy, 0.0, 1.2);
    float flicker = 0.85 + 0.15 * hash(h.zw + floor(TIME * 8.0));
    float hd = hexDist(local);
    float cellMask = 1.0 - smoothstep(0.5 - gap * 0.5 - 0.02, 0.5 - gap * 0.5, hd);
    float edge = smoothstep(0.5 - gap * 0.5 - 0.08, 0.5 - gap * 0.5 - 0.02, hd) * cellMask;
    vec3 lit = mix(colorA.rgb, colorB.rgb, smoothstep(0.4, 1.0, energy)) * energy * flicker;
    vec3 col = bgColor.rgb;
    col = mix(col, bgColor.rgb + lit, cellMask);
    col += colorA.rgb * edge * outline * 0.6;
    gl_FragColor = keyed(col, bgColor.rgb);
}
