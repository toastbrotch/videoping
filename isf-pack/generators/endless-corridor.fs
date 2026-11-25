/*{
  "DESCRIPTION": "Flying through an endless corridor of frames, twisting as it goes, fading into fog. Five looks: shaded, wireframe, outline, dots, x-ray. Crazy slider adds warping, pulsing frames, kaleidoscope, camera roll and rainbow colors.",
  "CREDIT": "videoping shader pack, written by Claude, guided by VJ mr.shintla of videopong.net - CC BY-NC-SA 4.0",
  "ISFVSN": "2",
  "CATEGORIES": ["generator", "3d", "tunnel"],
  "INPUTS": [
    { "NAME": "style", "LABEL": "Style", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2, 3, 4], "LABELS": ["shaded", "wireframe", "outline", "dots", "x-ray"] },
    { "NAME": "shape", "LABEL": "Frame shape", "TYPE": "long", "DEFAULT": 0, "VALUES": [0, 1, 2], "LABELS": ["square", "round", "hexagon"] },
    { "NAME": "speed", "LABEL": "Speed", "TYPE": "float", "DEFAULT": 3.0, "MIN": -6.0, "MAX": 6.0 },
    { "NAME": "twist", "LABEL": "Twist", "TYPE": "float", "DEFAULT": 0.35, "MIN": -1.0, "MAX": 1.0 },
    { "NAME": "spacing", "LABEL": "Frame spacing", "TYPE": "float", "DEFAULT": 0.8, "MIN": 0.3, "MAX": 3.0 },
    { "NAME": "thickness", "LABEL": "Frame thickness", "TYPE": "float", "DEFAULT": 0.12, "MIN": 0.02, "MAX": 0.4 },
    { "NAME": "sway", "LABEL": "Sway", "TYPE": "float", "DEFAULT": 0.9, "MIN": 0.0, "MAX": 1.5 },
    { "NAME": "fog", "LABEL": "Fog", "TYPE": "float", "DEFAULT": 0.1, "MIN": 0.0, "MAX": 0.5 },
    { "NAME": "lineWidth", "LABEL": "Line width (px)", "TYPE": "float", "DEFAULT": 1.2, "MIN": 0.5, "MAX": 5.0 },
    { "NAME": "wireDensity", "LABEL": "Wire density", "TYPE": "float", "DEFAULT": 8.0, "MIN": 1.0, "MAX": 30.0 },
    { "NAME": "fill", "LABEL": "Fill under lines", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "color1", "LABEL": "Color", "TYPE": "color", "DEFAULT": [0.95, 0.92, 0.85, 1.0] },
    { "NAME": "color2", "LABEL": "Shadow color", "TYPE": "color", "DEFAULT": [0.55, 0.29, 0.53, 1.0] },
    { "NAME": "bgColor", "LABEL": "Background / fog", "TYPE": "color", "DEFAULT": [0.0, 0.0, 0.0, 1.0] },
    { "NAME": "crazy", "LABEL": "Crazy", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "kaleido", "LABEL": "Kaleidoscope", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "rainbow", "LABEL": "Rainbow", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 },
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

mat2 rot2(float a) { float c = cos(a), s = sin(a); return mat2(c, s, -s, c); }

vec3 hue(float h) { return 0.5 + 0.5 * cos(6.2832 * (h + vec3(0.0, 0.33, 0.67))); }

float camZ() { return TIME * speed + crazy * sin(TIME * 0.7) * 2.0; }
vec2 path(float z) {
    return sway * vec2(sin(z * 0.21) + crazy * 0.6 * sin(z * 0.53 + TIME), cos(z * 0.17) * 0.6 + crazy * 0.5 * cos(z * 0.41 - TIME * 0.8));
}
float twistAt(float z) { return twist * (1.0 + crazy * 2.0 * sin(TIME * 0.31 + z * 0.05)); }

float frame2d(vec2 p, float z) {
    // per-frame pulsing size, wobble
    float pulse = 1.0 + crazy * 0.35 * sin(z * 1.7 - TIME * 4.0);
    p /= pulse;
    p += crazy * 0.15 * sin(p.yx * 4.0 + TIME * 2.0);
    int sh = shape;
    if (crazy > 0.5) sh = int(mod(floor(z / spacing + 0.5) + 3000.0, 3.0));
    float r;
    vec2 q = abs(p);
    if (sh == 1) r = abs(length(p) - 1.0);
    else if (sh == 2) r = abs(max(q.x * 0.866 + q.y * 0.5, q.y) - 1.0);
    else r = abs(max(q.x, q.y) - 1.0);
    return r * pulse;
}

float scene(vec3 p) {
    p.xy -= path(p.z);
    p.xy = rot2(p.z * twistAt(p.z)) * p.xy;
    float z = mod(p.z + spacing * 0.5, spacing) - spacing * 0.5;
    float ring = frame2d(p.xy, p.z) - thickness;
    return max(ring, abs(z) - thickness) * 0.8;
}

vec3 normalAt(vec3 p) {
    vec2 e = vec2(0.002, 0.0);
    return normalize(vec3(scene(p + e.xyy) - scene(p - e.xyy), scene(p + e.yxy) - scene(p - e.yxy), scene(p + e.yyx) - scene(p - e.yyx)));
}

void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * RENDERSIZE) / RENDERSIZE.y;
    // camera roll + kaleidoscope fold
    uv = rot2(crazy * (TIME * 0.4 + sin(TIME * 0.9) * 1.5)) * uv;
    if (kaleido > 0.0) {
        float n = 3.0 + floor(kaleido * 5.0);
        float a = atan(uv.y, uv.x), r = length(uv);
        float seg = 6.2832 / n;
        float af = abs(mod(a, seg) - seg * 0.5);
        uv = mix(uv, r * vec2(cos(af), sin(af)), kaleido);
    }
    // lens warp
    uv *= 1.0 + crazy * 0.4 * dot(uv, uv) * sin(TIME * 1.3);

    float cz = camZ();
    vec3 ro = vec3(path(cz), cz);
    vec3 ahead = vec3(path(cz + 1.0), cz + 1.0);
    vec3 fw = normalize(ahead - ro);
    vec3 rt = normalize(cross(vec3(0.0, 1.0, 0.0), fw));
    vec3 up = cross(fw, rt);
    float fov = 1.3 - crazy * 0.5 * sin(TIME * 0.5);
    vec3 rd = normalize(fw * fov + uv.x * rt + uv.y * up);

    float t = 0.0, minD = 1e5, tMin = 0.0, xray = 0.0;
    bool hit = false;
    for (int i = 0; i < 100; i++) {
        vec3 p = ro + rd * t;
        float d = scene(p);
        if (d < minD) { minD = d; tMin = t; }
        if (style == 4) {
            xray += exp(-abs(d) * 20.0) * 0.04 * exp(-t * fog);
            t += max(abs(d) * 0.7, 0.02);
        } else {
            if (d < 0.001 * (1.0 + t)) { hit = true; break; }
            t += d * 0.9;
        }
        if (t > 30.0) break;
    }
    vec3 p = ro + rd * t;
    vec3 n = hit ? normalAt(p) : vec3(0.0);
    vec3 lp = p; lp.xy -= path(p.z); lp.xy = rot2(p.z * twistAt(p.z)) * lp.xy;
    vec3 q = lp * wireDensity * 0.5;

    // rainbow colors flowing along the corridor
    vec3 rb = hue(p.z * 0.08 - TIME * 0.3 + atan(lp.y, lp.x) * 0.16);
    vec3 c1 = mix(color1.rgb, rb, rainbow);
    vec3 c2 = mix(color2.rgb, hue(p.z * 0.08 - TIME * 0.3 + 0.5) * 0.6, rainbow);

    float fade = exp(-t * fog);
    vec3 l = normalize(-rd + vec3(0.3, 0.5, 0.0));
    float diff = max(dot(n, l), 0.0);
    vec3 shaded = mix(c2, c1, diff);
    float lum = dot(shaded, vec3(0.3, 0.5, 0.2)) * fade;

    vec3 fq = fwidth(q);
    float fn = length(fwidth(n));
    float ft = fwidth(t) / max(t, 1e-3);
    float ndv = hit ? max(dot(n, -rd), 0.0) : 1.0;
    float rimLine = hit ? 1.0 - smoothstep(0.0, max(fwidth(ndv), 1e-3) * lineWidth, ndv) : 0.0;

    vec3 bg = bgColor.rgb + crazy * rainbow * 0.12 * hue(TIME * 0.1 + length(uv)) * (0.5 + 0.5 * sin(length(uv) * 20.0 - TIME * 3.0));
    vec3 col = bg;
    float fgA = 0.0;      // how much of the pixel is the corridor (fog counts as background)
    if (style == 0) {
        if (hit) col = mix(bg, shaded, fade);
        fgA = hit ? fade : 0.0;
    } else if (style == 1) {
        vec3 g = abs(fract(q - 0.5) - 0.5) / max(fq, vec3(1e-4));
        float line = 1.0 - smoothstep(lineWidth * 0.5, lineWidth * 0.5 + 1.0, min(min(g.x, g.y), g.z));
        if (hit) col = mix(bg, mix(bg, shaded, fill) + c1 * line, fade);
        fgA = hit ? max(fill, line) * fade : 0.0;
    } else if (style == 2) {
        float edge = hit ? smoothstep(0.06, 0.16, fn) : 0.0;
        edge = max(edge, hit ? smoothstep(0.03, 0.08, ft) : 0.0);
        edge = max(edge, rimLine);
        if (hit) col = mix(bg, shaded, fill * fade);
        col = mix(col, c1, edge * fade);
        fgA = max(edge, hit ? fill : 0.0) * fade;
    } else if (style == 3) {
        float cell = 4.0 + lineWidth * 3.0;
        vec2 c = mod(gl_FragCoord.xy, cell) - cell * 0.5;
        float r = cell * 0.5 * sqrt(clamp(lum, 0.0, 1.0)) * 1.1;
        float dotm = hit ? 1.0 - smoothstep(r - 0.8, r + 0.8, length(c)) : 0.0;
        if (hit) col = mix(bg, shaded, fill * fade);
        col = mix(col, c1, dotm);
        fgA = max(dotm, hit ? fill * fade : 0.0);
    } else {
        col = bg + c1 * xray;
    }
    // glow from near misses
    col += c1 * crazy * 0.25 * exp(-minD * 12.0) * exp(-tMin * fog);
    gl_FragColor = style == 4 ? keyed(col, bgColor.rgb) : withBackground(col, bgColor.rgb, fgA);
}
