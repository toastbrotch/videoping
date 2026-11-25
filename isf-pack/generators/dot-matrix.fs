/*{
  "DESCRIPTION": "An LED wall of round dots with glow. 14 patterns: waves, noise, rings, bars, plasma, lines, diagonals, scanner, spiral, checker, rain, sparkle, radar, squares. Unlit dots stay faintly visible.",
  "CREDIT": "videoping shader pack, written by Claude - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "led", "retro", "grid"],
  "INPUTS": [
    { "NAME": "pattern", "LABEL": "Pattern", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13], "LABELS": ["waves", "noise", "rings", "bars", "plasma", "lines", "diagonals", "scanner", "spiral", "checker", "rain", "sparkle", "radar", "squares"] },
    { "NAME": "columns", "LABEL": "Dots across", "TYPE": "float", "DEFAULT": 48.0, "MIN": 8.0, "MAX": 160.0 },
    { "NAME": "dotSize", "LABEL": "Dot size", "TYPE": "float", "DEFAULT": 0.7, "MIN": 0.2, "MAX": 1.0 },
    { "NAME": "speed", "LABEL": "Speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 4.0 },
    { "NAME": "scale", "LABEL": "Pattern scale", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.2, "MAX": 4.0 },
    { "NAME": "levels", "LABEL": "Brightness steps", "TYPE": "float", "DEFAULT": 6.0, "MIN": 2.0, "MAX": 16.0 },
    { "NAME": "glow", "LABEL": "Glow", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.5 },
    { "NAME": "offLevel", "LABEL": "Unlit dots", "TYPE": "float", "DEFAULT": 0.08, "MIN": 0.0, "MAX": 0.4 },
    { "NAME": "ledColor", "LABEL": "LED color", "TYPE": "color", "DEFAULT": [1.0, 0.55, 0.1, 1.0] },
    { "NAME": "hotColor", "LABEL": "Hot color", "TYPE": "color", "DEFAULT": [1.0, 0.95, 0.8, 1.0] },
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

float hash(vec2 p) { p = fract(p * vec2(123.34, 456.21)); p += dot(p, p + 45.32); return fract(p.x * p.y); }

float noise(vec2 p) {
    vec2 i = floor(p), f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
}

float field(vec2 p) {
    float t = TIME * speed;
    p *= scale;
    if (pattern == 0) return 0.5 + 0.5 * sin(p.x * 4.0 + sin(p.y * 3.0 + t) * 1.5 - t * 2.0);
    if (pattern == 1) return noise(p * 3.0 + vec2(t * 0.7, -t * 0.4)) * 0.7 + noise(p * 7.0 - t) * 0.3;
    if (pattern == 2) return 0.5 + 0.5 * sin(length(p) * 10.0 - t * 3.0);
    if (pattern == 3) {
        float bar = floor((p.x + 2.0) * 4.0);
        float hgt = 0.5 + 0.45 * sin(t * (1.0 + hash(vec2(bar, 1.0)) * 2.0) + bar);
        return step(p.y + 0.5, hgt) * (0.6 + 0.4 * (p.y + 0.5) / max(hgt, 0.01));
    }
    if (pattern == 4) return 0.5 + 0.25 * (sin(p.x * 5.0 + t) + sin(p.y * 4.0 - t * 1.3) + sin((p.x + p.y) * 3.0 + t * 0.7) + sin(length(p) * 6.0 - t)) * 0.5;
    // lines: bright horizontal lines scrolling up, each with a short fading tail
    if (pattern == 5) return pow(fract(p.y * 3.0 - t * 0.5), 6.0);
    // diagonals: stripes moving across at 45 degrees
    if (pattern == 6) return smoothstep(0.55, 0.95, 0.5 + 0.5 * sin((p.x + p.y) * 9.0 - t * 3.0));
    // scanner: one column sweeping left and right with a trail
    if (pattern == 7) {
        float x = 0.85 * sin(t * 1.5);
        float dir = cos(t * 1.5) > 0.0 ? 1.0 : -1.0;
        float d = (x - p.x) * dir;      // > 0 behind the beam
        return d < -0.02 ? 0.0 : exp(-max(d, 0.0) * 6.0);
    }
    // spiral: arms turning around the center
    if (pattern == 8) {
        float a = atan(p.y, p.x);
        return smoothstep(0.4, 0.9, 0.5 + 0.5 * sin(a * 3.0 + length(p) * 14.0 - t * 3.0));
    }
    // checker: a checkerboard that flips in time
    if (pattern == 9) {
        vec2 c = floor(p * 4.0);
        return mod(c.x + c.y + floor(t * 2.0), 2.0);
    }
    // rain: drops falling down the columns at their own speed, with tails
    if (pattern == 10) {
        float col = floor(p.x * 30.0);
        float sp = 0.4 + hash(vec2(col, 3.1)) * 0.9;
        float y = fract(-p.y * 0.6 - t * sp * 0.5 + hash(vec2(col, 7.7)));
        return pow(y, 8.0) * step(0.3, hash(vec2(col, 1.9)));
    }
    // sparkle: random dots flash and fade
    if (pattern == 11) {
        vec2 c = floor(p * 40.0);
        float ph = hash(c) + t * (0.3 + hash(c + 4.0) * 0.7);
        return pow(1.0 - fract(ph), 12.0);
    }
    // radar: a sweeping beam with an afterglow and a few blips
    if (pattern == 12) {
        float a = mod(atan(p.y, p.x) - t * 1.5, 6.2831853);
        float beam = exp(-a * 1.2) * step(length(p), 0.95);
        vec2 c = floor(p * 12.0);
        float blip = step(0.97, hash(c)) * exp(-a * 0.8);
        float ring = smoothstep(0.42, 0.5, abs(fract(length(p) * 3.0) - 0.5)) * 0.3;
        return max(max(beam, blip), ring * step(length(p), 0.95));
    }
    // squares: square outlines growing from the center (a tunnel)
    vec2 q = abs(p);
    return smoothstep(0.6, 1.0, 0.5 + 0.5 * sin(max(q.x, q.y) * 14.0 - t * 4.0));
}

void main() {
    float aspect = RENDERSIZE.x / RENDERSIZE.y;
    float cols = floor(columns + 0.5);
    float pitch = RENDERSIZE.x / cols;
    vec2 g = gl_FragCoord.xy / pitch;
    vec2 id = floor(g);
    vec2 f = fract(g) - 0.5;
    // the pattern is sampled once per dot, at its center
    vec2 p = ((id + 0.5) * pitch - 0.5 * RENDERSIZE) / RENDERSIZE.y;
    float v = clamp(field(p), 0.0, 1.0);
    float steps = floor(levels + 0.5);
    v = floor(v * steps + 0.5) / steps;
    float r = length(f);
    float dotm = 1.0 - smoothstep(dotSize * 0.5 - 0.06, dotSize * 0.5, r);
    float halo = exp(-r * 6.0) * glow * v;
    float level = max(v, offLevel);
    vec3 led = mix(ledColor.rgb, hotColor.rgb, smoothstep(0.7, 1.0, v));
    // a tiny highlight makes the dots look like lenses
    float lens = 1.0 - smoothstep(0.0, dotSize * 0.25, length(f - vec2(-0.12, 0.12) * dotSize));
    vec3 col = bgColor.rgb + led * halo * 0.6;
    col = mix(col, led * level * (1.0 + lens * 0.25 * v), dotm);
    gl_FragColor = keyed(col, bgColor.rgb);
}
