/*{
  "DESCRIPTION": "A blooming flower of mirrored noise petals in polar coordinates, breathing and turning.",
  "CREDIT": "videoping shader pack, written by Claude - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "kaleidoscope", "organic"],
  "INPUTS": [
    { "NAME": "petals", "LABEL": "Petals", "TYPE": "float", "DEFAULT": 8.0, "MIN": 3.0, "MAX": 24.0 },
    { "NAME": "symmetry", "LABEL": "Symmetry", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1], "LABELS": ["mirrored", "rotated"] },
    { "NAME": "rotation", "LABEL": "Rotation", "TYPE": "float", "DEFAULT": 0.1, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "zoom", "LABEL": "Zoom", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.3, "MAX": 3.0 },
    { "NAME": "bloom", "LABEL": "Bloom speed", "TYPE": "float", "DEFAULT": 0.3, "MIN": -2.0, "MAX": 2.0 },
    { "NAME": "detail", "LABEL": "Detail", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "rings", "LABEL": "Rings", "TYPE": "float", "DEFAULT": 0.4, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "sharpness", "LABEL": "Sharp edges", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "crisp", "LABEL": "Crisp (fine detail + edge lines)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "dirt", "LABEL": "Dirt (grain, specks, scratches)", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "colorIn", "LABEL": "Inner color", "TYPE": "color", "DEFAULT": [0.98, 0.85, 0.6, 1.0] },
    { "NAME": "colorOut", "LABEL": "Outer color", "TYPE": "color", "DEFAULT": [0.55, 0.29, 0.53, 1.0] },
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

float fbm(vec2 p, int oct) {
    float v = 0.0, a = 0.5;
    for (int i = 0; i < 6; i++) { if (i >= oct) break; v += a * noise(p); p = p * 2.03 + 5.1; a *= 0.5; }
    return v;
}

void main() {
    vec2 p = (gl_FragCoord.xy - 0.5 * RENDERSIZE) / RENDERSIZE.y / zoom;
    float r = length(p);
    float a = atan(p.y, p.x) + TIME * rotation;
    float n = floor(petals + 0.5);
    float seg = 6.2831853 / n;
    float sa = mod(a, seg);
    // mirrored: every second petal reflected; rotated: all the same way
    if (symmetry == 0) sa = abs(sa - seg * 0.5);
    float lr = log(r + 1e-4);
    vec2 q = vec2(sa * 2.0, lr * 1.5 - TIME * bloom);
    // crisp adds the finest octaves
    int oct = 2 + int(detail * 4.0 + crisp * 2.0);
    float f = fbm(q * 2.0, oct);
    // petal outline: wider in the middle of a segment
    float shape = cos(sa / seg * 3.14159 * (symmetry == 0 ? 2.0 : 1.0)) * 0.5 + 0.5;
    float x = f * 0.8 + shape * 0.5 - r * 0.35;
    // sharp: the soft petal edge becomes a hard cut, the color steps into bands
    float w = mix(0.25, 0.004, sharpness);
    float petal = smoothstep(0.45 - w, 0.45 + w, x);
    float band = mix(1.0, 0.7 + 0.3 * sin(lr * 18.0 - TIME * bloom * 6.0 + f * 4.0), rings);
    float shade = smoothstep(0.9, 0.0, r) * f * 1.6;
    float steps = mix(64.0, 4.0, sharpness * sharpness);
    shade = mix(shade, floor(shade * steps + 0.5) / steps, sharpness);
    vec3 col = mix(colorOut.rgb, colorIn.rgb, shade);
    col = mix(bgColor.rgb, col * band, petal);
    // crisp: thin bright lines along the petal edges and the inner folds
    float fw = max(fwidth(x), 1e-4);
    float edgeLine = 1.0 - smoothstep(0.0, 1.5, abs(x - 0.45) / fw);
    float fold = 1.0 - smoothstep(0.0, 1.2, abs(fract(f * 6.0) - 0.5) / max(fwidth(f * 6.0), 1e-4));
    col += mix(colorIn.rgb, vec3(1.0), 0.5) * crisp * (edgeLine + fold * 0.5 * petal);
    // a small bright center
    col += colorIn.rgb * exp(-r * 18.0) * 0.8;
    // dirt: film grain, dark and light specks, fine radial scratches (they turn with the flower)
    if (dirt > 0.0) {
        vec2 px = gl_FragCoord.xy;
        float grain = hash(px + fract(TIME * 7.3) * 113.0) - 0.5;
        float speck = step(1.0 - dirt * 0.004, hash(floor(px / 2.0) + floor(TIME * 12.0) * 3.7));
        float dark = step(1.0 - dirt * 0.01, hash(floor(px / 3.0) * 1.3 + floor(TIME * 9.0)));
        float scratch = step(0.985, noise(vec2(a * 40.0 / 6.2831 * float(n), floor(TIME * 3.0)))) * smoothstep(0.1, 0.4, r);
        col += grain * dirt * 0.35;
        col = mix(col, vec3(1.0), speck * 0.8);
        col *= 1.0 - dark * 0.8;
        col = mix(col, col * 0.4 + colorIn.rgb * 0.3, scratch * dirt);
        // uneven, a little faded: like an old print
        col *= mix(1.0, 0.75 + 0.5 * noise(px / 180.0 + TIME * 0.05), dirt * 0.6);
    }
    gl_FragColor = keyed(max(col, 0.0), bgColor.rgb);
}
