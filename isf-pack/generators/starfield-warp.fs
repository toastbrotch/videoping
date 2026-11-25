/*{
  "DESCRIPTION": "Stars rushing towards you. At low speed calm points, at high speed long warp streaks; or the stars become spinning wire forms: crosses, rings, squares, triangles, hexagons, sparkles, or all of them mixed.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "3d", "space"],
  "INPUTS": [
    { "NAME": "style", "LABEL": "Form", "TYPE": "long", "DEFAULT": 1, "VALUES": [0, 1, 2, 3, 4, 5, 6, 7, 8], "LABELS": ["dots", "streaks", "crosses", "rings", "squares", "triangles", "hexagons", "sparkles", "mixed"] },
    { "NAME": "formSize", "LABEL": "Form size", "TYPE": "float", "DEFAULT": 1.6, "MIN": 0.3, "MAX": 16.0 },
    { "NAME": "spin", "LABEL": "Form spin", "TYPE": "float", "DEFAULT": 0.5, "MIN": -3.0, "MAX": 3.0 },
    { "NAME": "speed", "LABEL": "Speed", "TYPE": "float", "DEFAULT": 1.0, "MIN": -2.0, "MAX": 8.0 },
    { "NAME": "density", "LABEL": "Density", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.01, "MAX": 1.0 },
    { "NAME": "streak", "LABEL": "Streak length", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "starSize", "LABEL": "Star size", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.3, "MAX": 4.0 },
    { "NAME": "roll", "LABEL": "Roll", "TYPE": "float", "DEFAULT": 0.05, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "center", "LABEL": "Center", "TYPE": "point2D", "DEFAULT": [0.5, 0.5], "MIN": [0.0, 0.0], "MAX": [1.0, 1.0] },
    { "NAME": "tint", "LABEL": "Color spread", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "starColor", "LABEL": "Star color", "TYPE": "color", "DEFAULT": [0.9, 0.95, 1.0, 1.0] },
    { "NAME": "tintColor", "LABEL": "Tint color", "TYPE": "color", "DEFAULT": [0.95, 0.6, 0.9, 1.0] },
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

// four random numbers per star from one hash
vec4 hash4(vec2 p) {
    vec4 q = fract(vec4(p.xyxy) * vec4(0.1031, 0.1030, 0.0973, 0.1099));
    q += dot(q, q.wzxy + 33.33);
    return fract((q.xxyz + q.yzzw) * q.zywx);
}

float segDist(vec2 p, vec2 a, vec2 b) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-8), 0.0, 1.0);
    return length(pa - ba * h);
}

// distance to the outline of a form (r: offset from the star, already turned; a: its radius)
float formDist(vec2 r, int form, float a) {
    if (form == 2) {                                   // cross
        vec2 q = abs(r);
        return max(min(max(q.x, q.y - a), max(q.y, q.x - a)), length(r) - a);
    }
    if (form == 3) return abs(length(r) - a);          // ring
    if (form == 4) { vec2 q = abs(r); return abs(max(q.x, q.y) - a * 0.8); }   // square
    if (form == 5) {                                   // triangle
        vec2 q = vec2(abs(r.x), r.y);
        return abs(max(q.x * 0.866 + q.y * 0.5, -q.y) - a * 0.5);
    }
    if (form == 6) { vec2 q = abs(r); return abs(max(q.x * 0.866 + q.y * 0.5, q.y) - a * 0.85); }  // hexagon
    // sparkle: a four-pointed star (thin rays that get thinner outwards)
    vec2 q = abs(r);
    float thin = min(q.x, q.y) - 0.15 * a * (1.0 - clamp(max(q.x, q.y) / a, 0.0, 1.0));
    return max(thin, length(r) - a);
}

void main() {
    vec2 c = center * RENDERSIZE;
    vec2 uv = (gl_FragCoord.xy - c) / RENDERSIZE.y;
    float ra = TIME * roll;
    uv = mat2(cos(ra), sin(ra), -sin(ra), cos(ra)) * uv;
    float px = 1.0 / RENDERSIZE.y;
    vec3 col = bgColor.rgb;
    float z0 = TIME * speed * 0.25;
    const int LAYERS = 5;
    // depth layers: each holds a grid of stars, moving from far to near
    for (int i = 0; i < LAYERS; i++) {
        float layer = fract(float(i) / float(LAYERS) + z0);
        float depth = mix(4.0, 0.08, layer);           // far .. near
        float fadeIn = smoothstep(0.0, 0.25, layer) * smoothstep(1.0, 0.85, layer);
        float layerId = floor(float(i) / float(LAYERS) + z0) * 13.0 + float(i);
        // big forms get a coarser star grid (fewer, bigger forms), so each still fits its cell
        float k = style >= 2 ? max(1.0, formSize / 1.6) : 1.0;
        float gs = depth * 6.0 / k;
        vec2 g = uv * gs;
        vec2 id = floor(g);
        // 4 cells: the own one and three neighbors. Streaks reach towards the center, so for dots /
        // streaks those are the ones towards the center; a form is at most half a cell wide, so for
        // forms the neighbors on the side of the pixel are enough.
        vec2 toC = style < 2 ? vec2(g.x > 0.0 ? -1.0 : 1.0, g.y > 0.0 ? -1.0 : 1.0)
                             : vec2(fract(g.x) < 0.5 ? -1.0 : 1.0, fract(g.y) < 0.5 ? -1.0 : 1.0);
        for (int k = 0; k < 4; k++) {
            vec2 off = vec2(k == 1 || k == 3 ? toC.x : 0.0, k >= 2 ? toC.y : 0.0);
            vec2 cell = id + off;
            vec4 r1 = hash4(cell + layerId * 7.31);
            float h = r1.x;
            if (h > density) continue;
            vec2 sp = cell + r1.yz;
            vec4 r2 = hash4(cell * 1.37 + layerId * 3.1 + 17.0);
            vec2 s = sp / gs;                            // star on screen
            // streak: where the star was a moment ago (further in)
            float dz = clamp(speed * 0.06 * streak, 0.0, 0.4);
            vec2 s0 = s * depth / (depth + dz * depth * 4.0);
            float size = starSize * px * (1.5 + 2.0 / depth) * 0.5;
            // far from this star (beyond its form, streak and glow): nothing to draw
            float reach = (style < 2 ? length(s - s0) : size * 6.0 * formSize) + size * 12.0;
            if (length(uv - s) > reach) continue;
            float d;
            int form = style == 8 ? 2 + int(r2.x * 5.999) : style;
            if (form == 0) d = length(uv - s);
            else if (form == 1) d = segDist(uv, s0, s);
            else {
                // every form turns on its own, at its own speed
                float a = r2.y * 6.2831 + TIME * spin * (0.5 + r2.z);
                vec2 r = mat2(cos(a), sin(a), -sin(a), cos(a)) * (uv - s);
                d = formDist(r, form, min(size * 6.0 * formSize, 0.45 / gs));
            }
            float b = (1.0 - smoothstep(size * 0.5, size * 0.5 + px * 1.5, d)) + exp(-d / (size * 2.5)) * 0.25;
            vec3 sc = mix(starColor.rgb, tintColor.rgb, tint * r1.w);
            col += sc * b * fadeIn;
        }
    }
    gl_FragColor = keyed(col, bgColor.rgb);
}
