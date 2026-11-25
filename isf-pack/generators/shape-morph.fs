/*{
  "DESCRIPTION": "Outlines that morph circle to square to triangle to star and back, repeated as rings that travel outwards.",
  "CREDIT": "videoping shader pack, written by Claude - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "shapes", "lines"],
  "INPUTS": [
    { "NAME": "morph", "LABEL": "Shape (0 circle - 1 square - 2 triangle - 3 star)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 3.0 },
    { "NAME": "autoMorph", "LABEL": "Auto morph speed", "TYPE": "float", "DEFAULT": 0.2, "MIN": 0.0, "MAX": 2.0 },
    { "NAME": "rings", "LABEL": "Rings", "TYPE": "float", "DEFAULT": 8.0, "MIN": 1.0, "MAX": 30.0 },
    { "NAME": "travel", "LABEL": "Ring travel", "TYPE": "float", "DEFAULT": 0.3, "MIN": -2.0, "MAX": 2.0 },
    { "NAME": "lag", "LABEL": "Morph lag per ring", "TYPE": "float", "DEFAULT": 0.15, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "rotation", "LABEL": "Rotation", "TYPE": "float", "DEFAULT": 0.1, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "twistRings", "LABEL": "Twist per ring", "TYPE": "float", "DEFAULT": 0.1, "MIN": -0.5, "MAX": 0.5 },
    { "NAME": "lineWidth", "LABEL": "Line width (px)", "TYPE": "float", "DEFAULT": 2.0, "MIN": 0.5, "MAX": 12.0 },
    { "NAME": "starPoints", "LABEL": "Star points", "TYPE": "float", "DEFAULT": 5.0, "MIN": 3.0, "MAX": 12.0 },
    { "NAME": "colorIn", "LABEL": "Inner color", "TYPE": "color", "DEFAULT": [0.95, 0.92, 0.85, 1.0] },
    { "NAME": "colorOut", "LABEL": "Outer color", "TYPE": "color", "DEFAULT": [0.11, 0.41, 0.6, 1.0] },
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

float sdCircle(vec2 p) { return length(p) - 1.0; }
float sdSquare(vec2 p) { vec2 d = abs(p) - vec2(0.85); return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0); }
float sdTriangle(vec2 p) {
    p.y += 0.25;
    const float k = 1.7320508;
    p.x = abs(p.x) - 1.1;
    p.y = p.y + 1.1 / k;
    if (p.x + k * p.y > 0.0) p = vec2(p.x - k * p.y, -k * p.x - p.y) / 2.0;
    p.x -= clamp(p.x, -2.2, 0.0);
    return -length(p) * sign(p.y);
}
float sdStar(vec2 p, float n) {
    float an = 3.14159265 / n;
    float en = 3.14159265 / (n * 0.4 + 1.0);
    vec2 acs = vec2(cos(an), sin(an));
    vec2 ecs = vec2(cos(en), sin(en));
    float bn = mod(atan(p.x, p.y), 2.0 * an) - an;
    p = length(p) * vec2(cos(bn), abs(sin(bn)));
    p -= 1.1 * acs;
    p += ecs * clamp(-dot(p, ecs), 0.0, 1.1 * acs.y / ecs.y);
    return length(p) * sign(p.x);
}

float shapeN(vec2 p, float k) {
    if (k < 0.5) return sdCircle(p);
    if (k < 1.5) return sdSquare(p);
    if (k < 2.5) return sdTriangle(p);
    return sdStar(p, floor(starPoints + 0.5));
}

float shapeAt(vec2 p, float m) {
    m = mod(m, 4.0);                 // 3 -> 4 blends the star back into the circle
    float k = floor(m);
    return mix(shapeN(p, k), shapeN(p, mod(k + 1.0, 4.0)), smoothstep(0.0, 1.0, m - k));
}

void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE) / RENDERSIZE.y;
    float px = 1.0 / RENDERSIZE.y;
    float n = floor(rings + 0.5);
    float m0 = morph + TIME * autoMorph;
    vec3 col = bgColor.rgb;
    float maxR = 0.75;
    // each ring is the same shape at its own size, slightly behind in the morph
    for (int i = 0; i < 30; i++) {
        if (float(i) >= n) break;
        float k = fract((float(i) + TIME * travel) / n);
        float r = mix(0.02, maxR * 1.2, k);
        float rot = TIME * rotation + k * twistRings * 6.2831;
        vec2 q = mat2(cos(rot), sin(rot), -sin(rot), cos(rot)) * uv / r;
        float d = abs(shapeAt(q, m0 - k * lag * n * 0.25)) * r;
        float line = 1.0 - smoothstep(lineWidth * 0.5 * px, (lineWidth * 0.5 + 1.0) * px, d);
        float fade = smoothstep(0.0, 0.08, k) * smoothstep(1.0, 0.8, k);
        col = mix(col, mix(colorIn.rgb, colorOut.rgb, k), line * fade);
    }
    gl_FragColor = keyed(col, bgColor.rgb);
}
